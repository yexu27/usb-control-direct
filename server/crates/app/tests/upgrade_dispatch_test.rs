use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use protocol_gateway::post_send::{PostSendAction, PostSendActionExecutor};
use system_upgrade::{
    DebInspector, DebMetadata, PackageStager, PackageVerifier, SystemVersion, UpgradeCoordinator,
    UpgradeError, UpgradePreflight, UpgradePreflightRequest, UpgradeScheduler, UpgradeSourceReader,
    UpgradeSourceState, UpgradeStatus, UpgradeTask,
};
use tempfile::TempDir;
use usb_control_app::upgrade_dispatch::{
    UpgradeDispatch, UpgradeResultObserver, UpgradeStartAudit,
};
use usb_control_app::upgrade_scheduler::{SYSTEMCTL_PROGRAM, SYSTEMCTL_START_ARGS};

#[test]
fn send_failure_cancels_prepared_task_and_removes_payload() {
    let fixture = Fixture::new(false, false);
    let action = action();

    fixture.dispatch.cancel(&action);

    assert!(!fixture.root.join("current.json").exists());
    assert!(!fixture.root.join("staging").join(UPGRADE_ID).exists());
    assert_eq!(fixture.history().status, UpgradeStatus::Cancelled);
    assert!(fixture.events().is_empty());
}

#[test]
fn dispatch_observes_before_schedule() {
    let fixture = Fixture::new(false, false);

    fixture.dispatch.execute(action()).unwrap();
    fixture.dispatch.execute(action()).unwrap();

    assert_eq!(
        fixture.events(),
        vec![
            "audit:upgrade-test",
            "observe:upgrade-test",
            "schedule:upgrade-test"
        ]
    );
    assert_eq!(fixture.current().status, UpgradeStatus::Accepted);
}

#[test]
fn start_audit_failure_does_not_block_observer_or_scheduler() {
    let fixture = Fixture::new(true, false);

    fixture.dispatch.execute(action()).unwrap();

    assert_eq!(
        fixture.events(),
        vec![
            "audit:upgrade-test",
            "observe:upgrade-test",
            "schedule:upgrade-test"
        ]
    );
    assert_eq!(fixture.current().status, UpgradeStatus::Accepted);
}

#[test]
fn schedule_failure_is_observed_before_result_persist() {
    let fixture = Fixture::new(false, true);

    assert!(fixture.dispatch.execute(action()).is_err());

    assert!(!fixture.root.join("current.json").exists());
    assert_eq!(fixture.history().status, UpgradeStatus::ScheduleFailed);
    assert_eq!(
        fixture.events(),
        vec![
            "audit:upgrade-test",
            "observe:upgrade-test",
            "schedule:upgrade-test"
        ]
    );
}

#[test]
fn duplicate_post_send_action_observes_once() {
    let fixture = Fixture::new(false, false);

    fixture.dispatch.execute(action()).unwrap();
    fixture.dispatch.execute(action()).unwrap();

    assert_eq!(
        fixture
            .events()
            .iter()
            .filter(|event| event.as_str() == "observe:upgrade-test")
            .count(),
        1
    );
}

#[test]
fn systemd_scheduler_command_is_fixed_and_not_configurable() {
    assert_eq!(SYSTEMCTL_PROGRAM, "systemctl");
    assert_eq!(
        SYSTEMCTL_START_ARGS,
        ["start", "--no-block", "usb-control-updater.service"]
    );
}

const UPGRADE_ID: &str = "upgrade-test";

fn action() -> PostSendAction {
    PostSendAction::StartSystemUpgrade {
        upgrade_id: UPGRADE_ID.into(),
    }
}

struct Fixture {
    _temp: TempDir,
    root: std::path::PathBuf,
    dispatch: UpgradeDispatch,
    events: Arc<Mutex<Vec<String>>>,
}

impl Fixture {
    fn new(audit_fails: bool, scheduler_fails: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("upgrade");
        let events = Arc::new(Mutex::new(Vec::new()));
        let scheduler = Arc::new(RecordingScheduler {
            events: Arc::clone(&events),
            fails: scheduler_fails,
        });
        let coordinator = Arc::new(
            UpgradeCoordinator::new(
                root.clone(),
                PackageStager::new(root.clone(), 128 * 1024 * 1024),
                PackageVerifier::new(temp.path().join("keys"), Arc::new(UnusedInspector)),
                Arc::new(TestUpgradeSource),
                1,
                Arc::new(TestPreflight),
                scheduler,
            )
            .unwrap(),
        );
        let staging = root.join("staging").join(UPGRADE_ID);
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("package.bin"), b"prepared").unwrap();
        fs::write(
            root.join("current.json"),
            serde_json::to_vec(&prepared_task()).unwrap(),
        )
        .unwrap();
        let audit = Arc::new(RecordingAudit {
            events: Arc::clone(&events),
            fails: audit_fails,
        });
        let observer = Arc::new(RecordingObserver {
            events: Arc::clone(&events),
        });
        let dispatch = UpgradeDispatch::new(coordinator, audit, observer);
        Self {
            _temp: temp,
            root,
            dispatch,
            events,
        }
    }

    fn events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }

    fn current(&self) -> UpgradeTask {
        read_json(&self.root.join("current.json"))
    }

    fn history(&self) -> UpgradeTask {
        read_json(&self.root.join("history").join(format!("{UPGRADE_ID}.json")))
    }
}

struct RecordingObserver {
    events: Arc<Mutex<Vec<String>>>,
}

impl UpgradeResultObserver for RecordingObserver {
    fn observe(&self, task: UpgradeTask) {
        self.events
            .lock()
            .unwrap()
            .push(format!("observe:{}", task.upgrade_id));
    }
}

struct TestPreflight;

struct TestUpgradeSource;

impl UpgradeSourceReader for TestUpgradeSource {
    fn read(&self) -> Result<UpgradeSourceState, UpgradeError> {
        Ok(UpgradeSourceState {
            current_version: SystemVersion::parse("3.0.1")?,
            current_schema: 1,
        })
    }
}

impl UpgradePreflight for TestPreflight {
    fn check(&self, _request: &UpgradePreflightRequest) -> Result<(), UpgradeError> {
        Ok(())
    }
}

fn prepared_task() -> UpgradeTask {
    UpgradeTask {
        format_version: 1,
        upgrade_id: UPGRADE_ID.into(),
        status: UpgradeStatus::Prepared,
        username: "admin".into(),
        role: 0,
        source_ip: "192.0.2.10".into(),
        source_version: SystemVersion::parse("3.0.1").unwrap(),
        target_version: SystemVersion::parse("3.1.0").unwrap(),
        package_sha256: "a".repeat(64),
        created_at: 1,
        updated_at: 1,
    }
}

fn read_json(path: &Path) -> UpgradeTask {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

struct RecordingScheduler {
    events: Arc<Mutex<Vec<String>>>,
    fails: bool,
}

impl UpgradeScheduler for RecordingScheduler {
    fn start(&self, upgrade_id: &str) -> Result<(), UpgradeError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("schedule:{upgrade_id}"));
        if self.fails {
            Err(UpgradeError::State("injected scheduler failure".into()))
        } else {
            Ok(())
        }
    }
}

struct RecordingAudit {
    events: Arc<Mutex<Vec<String>>>,
    fails: bool,
}

impl UpgradeStartAudit for RecordingAudit {
    fn record_start(&self, task: &UpgradeTask) -> Result<(), String> {
        self.events
            .lock()
            .unwrap()
            .push(format!("audit:{}", task.upgrade_id));
        if self.fails {
            Err("injected audit failure".into())
        } else {
            Ok(())
        }
    }
}

struct UnusedInspector;

impl DebInspector for UnusedInspector {
    fn inspect(&self, _deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        Ok(DebMetadata {
            package: "usb-control".into(),
            version: SystemVersion::parse("3.1.0")?,
            architecture: "arm64".into(),
            expanded_size: 4096,
            files: BTreeSet::new(),
            tls_cert_sha256: String::new(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            migration_schema_to: 2,
            upgrade_signing_key_id: String::new(),
        })
    }
}
