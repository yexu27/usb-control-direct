mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

use system_upgrade::{
    DebInspector, DebMetadata, PackageStager, PackageVerifier, PrepareUpgradeRequest,
    SystemVersion, UpgradeCoordinator, UpgradeEnvironment, UpgradeError, UpgradePreflight,
    UpgradePreflightFailure, UpgradePreflightRequest, UpgradeResultStore, UpgradeScheduler,
    UpgradeStateLock, UpgradeStatus, UpgradeTask, UpgradeTaskStore,
};

use support::{sha256_hex, MatchingDebInspector, PackageFixture};

const MAX_PACKAGE_SIZE: u64 = 128 * 1024 * 1024;

#[test]
fn accepts_only_legal_state_transitions() {
    use UpgradeStatus::*;

    let legal = [
        (Validating, Rejected),
        (Validating, Prepared),
        (Prepared, Cancelled),
        (Prepared, Accepted),
        (Accepted, ScheduleFailed),
        (Accepted, Stopping),
        (Accepted, Failed),
        (Stopping, Installing),
        (Stopping, Failed),
        (Installing, Migrating),
        (Installing, Failed),
        (Migrating, Starting),
        (Migrating, Failed),
        (Starting, HealthChecking),
        (Starting, Failed),
        (HealthChecking, Committed),
        (HealthChecking, Failed),
    ];

    for (from, to) in legal {
        assert!(
            from.can_transition_to(to),
            "expected legal transition {from:?} -> {to:?}"
        );
    }

    let all = [
        Validating,
        Rejected,
        Prepared,
        Cancelled,
        Accepted,
        ScheduleFailed,
        Stopping,
        Installing,
        Migrating,
        Starting,
        HealthChecking,
        Committed,
        Failed,
    ];
    for from in all {
        for to in all {
            let expected = legal.contains(&(from, to));
            assert_eq!(
                from.can_transition_to(to),
                expected,
                "unexpected transition decision for {from:?} -> {to:?}"
            );
        }
    }
}

#[test]
fn failed_is_terminal_and_releases_current_slot() {
    let root = tempfile::tempdir().unwrap();
    let store = UpgradeTaskStore::new(root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(root.path()).unwrap();
    let task = accepted_task("upgrade-failed-terminal");
    store
        .create(
            &guard,
            &UpgradeTask {
                status: UpgradeStatus::Prepared,
                ..task.clone()
            },
        )
        .unwrap();
    store
        .transition(&guard, &task.upgrade_id, UpgradeStatus::Accepted, 101)
        .unwrap();

    let failed = store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Failed, 102)
        .unwrap();

    assert_eq!(failed.status, UpgradeStatus::Failed);
    assert!(store.current().unwrap().is_none());
    assert_eq!(
        store.history(&task.upgrade_id).unwrap().unwrap().status,
        UpgradeStatus::Failed
    );
}

#[test]
fn ensure_terminal_is_idempotent_for_same_task_and_status() {
    let root = tempfile::tempdir().unwrap();
    let store = UpgradeTaskStore::new(root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(root.path()).unwrap();
    let task = accepted_task("upgrade-idempotent-terminal");
    store
        .create(
            &guard,
            &UpgradeTask {
                status: UpgradeStatus::Prepared,
                ..task.clone()
            },
        )
        .unwrap();
    store
        .transition(&guard, &task.upgrade_id, UpgradeStatus::Accepted, 101)
        .unwrap();

    let first = store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Failed, 102)
        .unwrap();
    let second = store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Failed, 103)
        .unwrap();

    assert_eq!(first, second);
}

#[test]
fn ensure_terminal_rejects_different_status() {
    let root = tempfile::tempdir().unwrap();
    let store = UpgradeTaskStore::new(root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(root.path()).unwrap();
    let task = accepted_task("upgrade-terminal-mismatch");
    store
        .create(
            &guard,
            &UpgradeTask {
                status: UpgradeStatus::Prepared,
                ..task.clone()
            },
        )
        .unwrap();
    store
        .transition(&guard, &task.upgrade_id, UpgradeStatus::Accepted, 101)
        .unwrap();
    store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Failed, 102)
        .unwrap();

    assert!(store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::ScheduleFailed, 103,)
        .is_err());
}

#[test]
fn ensure_terminal_rejects_different_current_task() {
    let root = tempfile::tempdir().unwrap();
    let store = UpgradeTaskStore::new(root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(root.path()).unwrap();
    let task = accepted_task("upgrade-current-task");
    store
        .create(
            &guard,
            &UpgradeTask {
                status: UpgradeStatus::Prepared,
                ..task
            },
        )
        .unwrap();

    assert!(store
        .ensure_terminal(&guard, "upgrade-another-task", UpgradeStatus::Failed, 102)
        .is_err());
}

#[test]
fn task_history_keeps_latest_twenty_by_updated_at() {
    let root = tempfile::tempdir().unwrap();
    let store = UpgradeTaskStore::new(root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(root.path()).unwrap();
    for index in 0..22 {
        let task = accepted_task(&format!("upgrade-{index:02}"));
        store
            .create(
                &guard,
                &UpgradeTask {
                    status: UpgradeStatus::Prepared,
                    ..task.clone()
                },
            )
            .unwrap();
        store
            .transition(
                &guard,
                &task.upgrade_id,
                UpgradeStatus::Accepted,
                101 + index,
            )
            .unwrap();
        store
            .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Failed, 200 + index)
            .unwrap();
    }

    assert_eq!(
        fs::read_dir(root.path().join("history")).unwrap().count(),
        20
    );
    assert!(store.history("upgrade-00").unwrap().is_none());
    assert!(store.history("upgrade-21").unwrap().is_some());
}

#[test]
fn atomically_persists_prepared_task_and_payload() {
    let fixture = PackageFixture::valid();
    let package_digest = sha256_hex(&fixture.package_bytes);
    let root = fixture.root();
    let coordinator = coordinator(&fixture, Arc::new(RecordingScheduler::succeeds()));

    let prepared = coordinator
        .prepare(request(&fixture, "admin", "192.0.2.10"))
        .expect("valid signed package must be prepared");

    assert!(is_path_safe_id(&prepared.upgrade_id));
    assert_eq!(
        prepared.target_version,
        SystemVersion::parse("3.1.0").expect("valid target version")
    );

    let current_path = root.join("current.json");
    let current: UpgradeTask = read_json(&current_path);
    assert_eq!(current.format_version, 1);
    assert_eq!(current.upgrade_id, prepared.upgrade_id);
    assert_eq!(current.status, UpgradeStatus::Prepared);
    assert_eq!(current.username, "admin");
    assert_eq!(current.role, 1);
    assert_eq!(current.source_ip, "192.0.2.10");
    assert_eq!(
        current.source_version,
        SystemVersion::parse("3.0.1").expect("valid source version")
    );
    assert_eq!(current.target_version, prepared.target_version);
    assert_eq!(current.package_sha256, package_digest);
    assert!(current.created_at > 0);
    assert!(current.updated_at >= current.created_at);

    let staging = root.join("staging").join(&prepared.upgrade_id);
    assert_eq!(
        fs::read(staging.join("package.bin")).expect("read persisted package"),
        fixture.package_bytes
    );
    assert_mode(&root, 0o700);
    assert_mode(&root.join("staging"), 0o700);
    assert_mode(&staging, 0o700);
    assert_mode(&current_path, 0o600);
    assert_mode(&staging.join("package.bin"), 0o600);
    assert_no_temporary_entries(&root);
}

#[test]
fn prepare_runs_preflight_once_before_prepared() {
    let fixture = PackageFixture::valid();
    let events = Arc::new(Mutex::new(Vec::new()));
    let preflight = Arc::new(RecordingPreflight::before_publish(
        fixture.root(),
        Arc::clone(&events),
    ));
    let coordinator = coordinator_with_components(
        &fixture,
        Arc::new(RecordingScheduler::succeeds()),
        Arc::new(RecordingDebInspector {
            events: Arc::clone(&events),
        }),
        preflight.clone(),
    );

    coordinator
        .prepare(request(&fixture, "admin", "192.0.2.25"))
        .expect("consistent environment must prepare");

    assert_eq!(events.lock().unwrap().as_slice(), ["inspect", "preflight"]);
    let calls = preflight.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].package_size, fixture.package_bytes.len() as u64);
    assert_eq!(calls[0].deb_size, fixture.deb_bytes.len() as u64);
    assert_eq!(calls[0].expanded_size, 4096);
    assert_eq!(calls[0].source_version, version("3.0.1"));
    assert_eq!(calls[0].target_version, version("3.1.0"));
    assert_eq!(calls[0].schema_from, 1);
    assert_eq!(calls[0].schema_to, 2);
}

#[test]
fn preflight_failure_records_rejected_history_and_removes_staging() {
    let fixture = PackageFixture::valid();
    let root = fixture.root();
    let preflight = Arc::new(RecordingPreflight::fails(
        root.clone(),
        UpgradePreflightFailure::DpkgBusy,
    ));
    let coordinator = coordinator_with_components(
        &fixture,
        Arc::new(RecordingScheduler::succeeds()),
        Arc::new(MatchingDebInspector),
        preflight,
    );

    let error = coordinator
        .prepare(request(&fixture, "admin", "192.0.2.26"))
        .expect_err("busy dpkg must reject before prepared");

    assert!(matches!(
        error,
        UpgradeError::Preflight(UpgradePreflightFailure::DpkgBusy)
    ));
    assert!(!root.join("current.json").exists());
    assert_eq!(fs::read_dir(root.join("staging")).unwrap().count(), 0);
    let history = fs::read_dir(root.join("history"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(history.len(), 1);
    let rejected: UpgradeTask = read_json(&history[0].path());
    assert_eq!(rejected.status, UpgradeStatus::Rejected);
}

#[test]
fn rejects_second_non_terminal_task_as_busy() {
    let fixture = PackageFixture::valid();
    let root = fixture.root();
    let coordinator = coordinator(&fixture, Arc::new(RecordingScheduler::succeeds()));

    let first = coordinator
        .prepare(request(&fixture, "admin-a", "192.0.2.11"))
        .expect("first package must be prepared");
    let second = coordinator.prepare(request(&fixture, "admin-b", "192.0.2.12"));

    assert!(matches!(second, Err(UpgradeError::Busy)));
    let current: UpgradeTask = read_json(&root.join("current.json"));
    assert_eq!(current.upgrade_id, first.upgrade_id);
    assert_eq!(current.username, "admin-a");
    assert_eq!(current.status, UpgradeStatus::Prepared);
    let staging_ids = fs::read_dir(root.join("staging"))
        .expect("read staging directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("read staging entries");
    assert_eq!(staging_ids.len(), 1);
    assert_eq!(staging_ids[0].file_name(), first.upgrade_id.as_str());
}

#[test]
fn concurrent_coordinators_never_overwrite_current_task() {
    let fixture = PackageFixture::valid();
    let root = fixture.root();
    let first = coordinator(&fixture, Arc::new(RecordingScheduler::succeeds()));
    let second = coordinator(&fixture, Arc::new(RecordingScheduler::succeeds()));
    let first_request = request(&fixture, "admin-a", "192.0.2.21");
    let second_request = request(&fixture, "admin-b", "192.0.2.22");
    let barrier = Arc::new(Barrier::new(3));

    let first_barrier = barrier.clone();
    let first_thread = thread::spawn(move || {
        first_barrier.wait();
        first.prepare(first_request)
    });
    let second_barrier = barrier.clone();
    let second_thread = thread::spawn(move || {
        second_barrier.wait();
        second.prepare(second_request)
    });
    barrier.wait();

    let results = [
        first_thread.join().expect("first coordinator thread"),
        second_thread.join().expect("second coordinator thread"),
    ];
    let successful_ids = results
        .iter()
        .filter_map(|result| result.as_ref().ok())
        .map(|prepared| prepared.upgrade_id.as_str())
        .collect::<Vec<_>>();
    let busy_count = results
        .iter()
        .filter(|result| matches!(result, Err(UpgradeError::Busy)))
        .count();

    assert_eq!(successful_ids.len(), 1, "exactly one prepare may win");
    assert_eq!(busy_count, 1, "the losing admission must report Busy");
    let current: UpgradeTask = read_json(&root.join("current.json"));
    assert_eq!(current.upgrade_id, successful_ids[0]);
    assert_eq!(current.status, UpgradeStatus::Prepared);
    let staging_ids = fs::read_dir(root.join("staging"))
        .expect("read staging directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("read staging entries");
    assert_eq!(staging_ids.len(), 1, "losing staging must be removed");
    assert_eq!(staging_ids[0].file_name(), successful_ids[0]);
}

#[test]
fn verification_error_is_not_masked_when_rejection_cleanup_fails() {
    let fixture = PackageFixture::valid();
    let coordinator = coordinator_with_inspector(
        &fixture,
        Arc::new(RecordingScheduler::succeeds()),
        Arc::new(SabotagingDebInspector),
    );

    let result = coordinator.prepare(request(&fixture, "admin", "192.0.2.23"));

    assert!(matches!(result, Err(UpgradeError::ProductMismatch)));
    let history = fs::read_dir(fixture.root().join("history"))
        .expect("read rejection history")
        .collect::<Result<Vec<_>, _>>()
        .expect("read rejection entries");
    assert_eq!(history.len(), 1);
    let rejected: UpgradeTask = read_json(&history[0].path());
    assert_eq!(rejected.status, UpgradeStatus::Rejected);
}

#[test]
fn response_failure_cancels_task_and_removes_payload() {
    let fixture = PackageFixture::valid();
    let root = fixture.root();
    let coordinator = coordinator(&fixture, Arc::new(RecordingScheduler::succeeds()));
    let prepared = coordinator
        .prepare(request(&fixture, "admin", "192.0.2.13"))
        .expect("package must be prepared");

    coordinator
        .response_failed(&prepared.upgrade_id)
        .expect("response failure must cancel prepared task");

    assert!(!root.join("current.json").exists());
    assert!(!root.join("staging").join(&prepared.upgrade_id).exists());
    let cancelled: UpgradeTask = read_json(
        &root
            .join("history")
            .join(format!("{}.json", prepared.upgrade_id)),
    );
    assert_eq!(cancelled.status, UpgradeStatus::Cancelled);

    let next = coordinator
        .prepare(request(&fixture, "admin", "192.0.2.14"))
        .expect("terminal cancellation must release the busy slot");
    assert_ne!(next.upgrade_id, prepared.upgrade_id);
}

#[test]
fn response_failure_keeps_prepared_task_when_payload_cleanup_fails() {
    let fixture = PackageFixture::valid();
    let root = fixture.root();
    let coordinator = coordinator(&fixture, Arc::new(RecordingScheduler::succeeds()));
    let prepared = coordinator
        .prepare(request(&fixture, "admin", "192.0.2.24"))
        .expect("package must be prepared");
    let staging = root.join("staging").join(&prepared.upgrade_id);
    fs::remove_dir_all(&staging).expect("remove staged directory for failure simulation");
    fs::write(&staging, b"not a directory").expect("replace staged directory with a file");

    let first = coordinator.response_failed(&prepared.upgrade_id);

    assert!(matches!(first, Err(UpgradeError::Io(_))));
    let current: UpgradeTask = read_json(&root.join("current.json"));
    assert_eq!(current.upgrade_id, prepared.upgrade_id);
    assert_eq!(current.status, UpgradeStatus::Prepared);
    assert!(!root
        .join("history")
        .join(format!("{}.json", prepared.upgrade_id))
        .exists());

    fs::remove_file(&staging).expect("remove cleanup failure sentinel");
    fs::create_dir(&staging).expect("restore a removable task staging directory");
    coordinator
        .response_failed(&prepared.upgrade_id)
        .expect("response cleanup must be retryable");

    assert!(!root.join("current.json").exists());
    assert!(!staging.exists());
    let cancelled: UpgradeTask = read_json(
        &root
            .join("history")
            .join(format!("{}.json", prepared.upgrade_id)),
    );
    assert_eq!(cancelled.status, UpgradeStatus::Cancelled);
}

#[test]
fn scheduler_failure_records_schedule_failed_without_stopping_service() {
    let fixture = PackageFixture::valid();
    let root = fixture.root();
    let scheduler = Arc::new(RecordingScheduler::fails());
    let coordinator = coordinator(&fixture, scheduler.clone());
    let prepared = coordinator
        .prepare(request(&fixture, "admin", "192.0.2.15"))
        .expect("package must be prepared");

    let premature = coordinator.schedule(&prepared.upgrade_id);
    assert!(matches!(premature, Err(UpgradeError::State(_))));
    assert!(scheduler.calls().is_empty());

    let accepted = coordinator
        .accept_after_response(&prepared.upgrade_id)
        .expect("sent response must accept prepared task");
    assert_eq!(accepted.status, UpgradeStatus::Accepted);

    let schedule = coordinator.schedule(&prepared.upgrade_id);
    assert!(matches!(schedule, Err(UpgradeError::State(_))));
    assert_eq!(scheduler.calls(), vec![prepared.upgrade_id.clone()]);
    assert!(!root.join("current.json").exists());
    assert!(root.join("staging").join(&prepared.upgrade_id).exists());

    let failed: UpgradeTask = read_json(
        &root
            .join("history")
            .join(format!("{}.json", prepared.upgrade_id)),
    );
    assert_eq!(failed.status, UpgradeStatus::ScheduleFailed);
    let result = UpgradeResultStore::new(root)
        .unwrap()
        .get(&prepared.upgrade_id)
        .unwrap()
        .expect("schedule failure must persist an importable result");
    assert_eq!(result.status, UpgradeStatus::ScheduleFailed);
    assert!(result
        .original_error
        .as_deref()
        .unwrap()
        .contains("scheduler rejected start"));
    assert_eq!(result.effective_version, failed.source_version);
}

fn accepted_task(upgrade_id: &str) -> UpgradeTask {
    UpgradeTask {
        format_version: 1,
        upgrade_id: upgrade_id.into(),
        status: UpgradeStatus::Accepted,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        package_sha256: "a".repeat(64),
        created_at: 100,
        updated_at: 100,
    }
}

fn coordinator(
    fixture: &PackageFixture,
    scheduler: Arc<dyn UpgradeScheduler>,
) -> UpgradeCoordinator {
    coordinator_with_components(
        fixture,
        scheduler,
        Arc::new(MatchingDebInspector),
        Arc::new(RecordingPreflight::succeeds(
            fixture.root(),
            Arc::new(Mutex::new(Vec::new())),
        )),
    )
}

fn coordinator_with_inspector(
    fixture: &PackageFixture,
    scheduler: Arc<dyn UpgradeScheduler>,
    inspector: Arc<dyn DebInspector>,
) -> UpgradeCoordinator {
    coordinator_with_components(
        fixture,
        scheduler,
        inspector,
        Arc::new(RecordingPreflight::succeeds(
            fixture.root(),
            Arc::new(Mutex::new(Vec::new())),
        )),
    )
}

fn coordinator_with_components(
    fixture: &PackageFixture,
    scheduler: Arc<dyn UpgradeScheduler>,
    inspector: Arc<dyn DebInspector>,
    preflight: Arc<dyn UpgradePreflight>,
) -> UpgradeCoordinator {
    UpgradeCoordinator::new(
        fixture.root(),
        PackageStager::new(fixture.root(), MAX_PACKAGE_SIZE),
        PackageVerifier::new(fixture.key_dir(), inspector),
        UpgradeEnvironment {
            current_version: SystemVersion::parse("3.0.1").expect("valid current version"),
            current_schema: 1,
            supported_schema_max: 2,
            protocol_version: 1,
        },
        preflight,
        scheduler,
    )
    .expect("create upgrade coordinator")
}

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).expect("valid test version")
}

fn request(fixture: &PackageFixture, username: &str, source_ip: &str) -> PrepareUpgradeRequest {
    PrepareUpgradeRequest {
        package_bytes: fixture.package_bytes.clone(),
        client_target_version: "v3.1.0".to_string(),
        client_sha256: sha256_hex(&fixture.package_bytes),
        username: username.to_string(),
        role: 1,
        source_ip: source_ip.to_string(),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    serde_json::from_slice(&fs::read(path).expect("read JSON file")).expect("parse strict JSON")
}

fn is_path_safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn assert_mode(path: &Path, expected: u32) {
    let mode = fs::metadata(path)
        .expect("read metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode,
        expected,
        "unexpected permissions for {}",
        path.display()
    );
}

fn assert_no_temporary_entries(root: &Path) {
    fn visit(path: &Path) {
        for entry in fs::read_dir(path).expect("walk upgrade directory") {
            let entry = entry.expect("read upgrade directory entry");
            let path = entry.path();
            assert!(
                !entry.file_name().to_string_lossy().contains(".tmp"),
                "temporary entry left behind: {}",
                path.display()
            );
            if entry.file_type().expect("read file type").is_dir() {
                visit(&path);
            }
        }
    }

    visit(root);
}

struct RecordingScheduler {
    calls: Mutex<Vec<String>>,
    fail: bool,
}

struct RecordingDebInspector {
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl DebInspector for RecordingDebInspector {
    fn inspect(&self, path: &Path) -> Result<DebMetadata, UpgradeError> {
        self.events.lock().unwrap().push("inspect");
        MatchingDebInspector.inspect(path)
    }
}

struct RecordingPreflight {
    root: std::path::PathBuf,
    calls: Mutex<Vec<UpgradePreflightRequest>>,
    events: Arc<Mutex<Vec<&'static str>>>,
    failure: Option<UpgradePreflightFailure>,
    assert_no_current: bool,
}

impl RecordingPreflight {
    fn succeeds(root: std::path::PathBuf, events: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self {
            root,
            calls: Mutex::new(Vec::new()),
            events,
            failure: None,
            assert_no_current: false,
        }
    }

    fn before_publish(root: std::path::PathBuf, events: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self {
            root,
            calls: Mutex::new(Vec::new()),
            events,
            failure: None,
            assert_no_current: true,
        }
    }

    fn fails(root: std::path::PathBuf, failure: UpgradePreflightFailure) -> Self {
        Self {
            root,
            calls: Mutex::new(Vec::new()),
            events: Arc::new(Mutex::new(Vec::new())),
            failure: Some(failure),
            assert_no_current: false,
        }
    }
}

impl UpgradePreflight for RecordingPreflight {
    fn check(&self, request: &UpgradePreflightRequest) -> Result<(), UpgradeError> {
        if self.assert_no_current {
            assert!(
                !self.root.join("current.json").exists(),
                "preflight must run before prepared is published"
            );
        }
        self.events.lock().unwrap().push("preflight");
        self.calls.lock().unwrap().push(request.clone());
        match &self.failure {
            Some(failure) => Err(UpgradeError::Preflight(failure.clone())),
            None => Ok(()),
        }
    }
}

impl RecordingScheduler {
    fn succeeds() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            fail: false,
        }
    }

    fn fails() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            fail: true,
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("scheduler lock").clone()
    }
}

impl UpgradeScheduler for RecordingScheduler {
    fn start(&self, upgrade_id: &str) -> Result<(), UpgradeError> {
        self.calls
            .lock()
            .expect("scheduler lock")
            .push(upgrade_id.to_string());
        if self.fail {
            Err(UpgradeError::State("scheduler rejected start".into()))
        } else {
            Ok(())
        }
    }
}

struct SabotagingDebInspector;

impl DebInspector for SabotagingDebInspector {
    fn inspect(&self, deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        let staging = deb_path.parent().expect("staged DEB parent");
        fs::remove_dir_all(staging).expect("remove staging during verifier failure simulation");
        fs::write(staging, b"cleanup must fail").expect("replace staging directory with file");
        Err(UpgradeError::ProductMismatch)
    }
}
