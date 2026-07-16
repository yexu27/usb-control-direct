use std::sync::Arc;

use storage::Storage;
use storage_test_support::TestDb;
use system_upgrade::{
    ActiveRelease, ActiveReleaseStore, SystemVersion, UpgradeResult, UpgradeResultStore,
    UpgradeStateLock, UpgradeStatus, UpgradeTask, UpgradeTaskStore,
};
use usb_control_app::upgrade_result::{ImportDisposition, UpgradeResultImporter};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn active(
    online_upgrade_id: Option<&str>,
    version_value: &str,
    committed_at: i64,
) -> ActiveRelease {
    ActiveRelease {
        format_version: 1,
        version: version(version_value),
        schema_version: 1,
        committed_at,
        online_upgrade_id: online_upgrade_id.map(str::to_owned),
    }
}

fn task(upgrade_id: &str) -> UpgradeTask {
    UpgradeTask {
        format_version: 1,
        upgrade_id: upgrade_id.into(),
        status: UpgradeStatus::Prepared,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        package_sha256: "b".repeat(64),
        created_at: 100,
        updated_at: 100,
    }
}

fn result(upgrade_id: &str, status: UpgradeStatus) -> UpgradeResult {
    let committed = status == UpgradeStatus::Committed;
    UpgradeResult {
        format_version: 1,
        upgrade_id: upgrade_id.into(),
        status,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        effective_version: version(if committed { "3.0.2" } else { "3.0.1" }),
        failed_stage: (!committed).then(|| "installing".into()),
        original_error: (!committed).then(|| "候选版本安装失败".into()),
        finished_at: 210,
    }
}

struct Fixture {
    root: tempfile::TempDir,
    _db: TestDb,
    storage: Arc<Storage>,
    importer: UpgradeResultImporter,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let db = TestDb::new();
        let storage = Arc::new(Storage::open(db.path()).unwrap());
        storage.config_set("system_version", "unchanged").unwrap();
        let importer = UpgradeResultImporter::new(root.path().to_path_buf(), Arc::clone(&storage));
        Self {
            root,
            _db: db,
            storage,
            importer,
        }
    }

    fn releases(&self) -> ActiveReleaseStore {
        ActiveReleaseStore::new(self.root.path().to_path_buf()).unwrap()
    }

    fn tasks(&self) -> UpgradeTaskStore {
        UpgradeTaskStore::new(self.root.path().to_path_buf()).unwrap()
    }

    fn results(&self) -> UpgradeResultStore {
        UpgradeResultStore::new(self.root.path().to_path_buf()).unwrap()
    }

    fn lock(&self) -> UpgradeStateLock {
        UpgradeStateLock::acquire(self.root.path()).unwrap()
    }
}

#[test]
fn imports_failed_result_once_without_changing_system_version() {
    let fixture = Fixture::new();
    fixture
        .releases()
        .commit(&fixture.lock(), &active(None, "3.0.1", 50))
        .unwrap();
    let value = result("upgrade-failed", UpgradeStatus::Failed);

    assert_eq!(
        fixture.importer.import_result(&value).unwrap(),
        ImportDisposition::Imported
    );
    assert_eq!(
        fixture.importer.import_result(&value).unwrap(),
        ImportDisposition::AlreadyImported
    );

    let logs = fixture
        .storage
        .operation_log_query_by_time(0, i64::MAX)
        .unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].result, 1);
    assert_eq!(logs[0].fail_reason.as_deref(), Some("候选版本安装失败"));
    assert_eq!(
        logs[0].request_id.as_deref(),
        Some("system-upgrade:upgrade-failed:result")
    );
    assert_eq!(
        fixture
            .storage
            .config_get("system_version")
            .unwrap()
            .unwrap()
            .config_value
            .as_deref(),
        Some("unchanged")
    );
}

#[test]
fn committed_result_requires_matching_active_release() {
    let fixture = Fixture::new();
    fixture
        .releases()
        .commit(
            &fixture.lock(),
            &active(Some("another-upgrade"), "3.0.2", 200),
        )
        .unwrap();

    assert!(fixture
        .importer
        .import_result(&result("upgrade-committed", UpgradeStatus::Committed))
        .is_err());
    assert_eq!(fixture.storage.operation_log_count().unwrap(), 0);
}

#[tokio::test]
async fn observer_persists_failed_terminal_before_importing_log() {
    let fixture = Fixture::new();
    fixture
        .releases()
        .commit(&fixture.lock(), &active(None, "3.0.1", 50))
        .unwrap();
    let tasks = fixture.tasks();
    let guard = fixture.lock();
    let prepared = task("upgrade-failed-observed");
    tasks.create(&guard, &prepared).unwrap();
    let accepted = tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Accepted, 110)
        .unwrap();
    fixture
        .results()
        .write(&guard, &result(&prepared.upgrade_id, UpgradeStatus::Failed))
        .unwrap();
    drop(guard);

    fixture
        .importer
        .monitor_active_task(accepted)
        .await
        .unwrap();

    assert_eq!(
        tasks.history(&prepared.upgrade_id).unwrap().unwrap().status,
        UpgradeStatus::Failed
    );
    assert!(tasks.current().unwrap().is_none());
    assert_eq!(fixture.storage.operation_log_count().unwrap(), 1);
}

#[tokio::test]
async fn live_observer_imports_schedule_failed_while_service_is_running() {
    let fixture = Fixture::new();
    fixture
        .releases()
        .commit(&fixture.lock(), &active(None, "3.0.1", 50))
        .unwrap();
    let tasks = fixture.tasks();
    let guard = fixture.lock();
    let prepared = task("upgrade-schedule-failed");
    tasks.create(&guard, &prepared).unwrap();
    let accepted = tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Accepted, 110)
        .unwrap();
    fixture
        .results()
        .write(
            &guard,
            &result(&prepared.upgrade_id, UpgradeStatus::ScheduleFailed),
        )
        .unwrap();
    drop(guard);

    fixture
        .importer
        .monitor_active_task(accepted)
        .await
        .unwrap();

    assert_eq!(
        tasks.history(&prepared.upgrade_id).unwrap().unwrap().status,
        UpgradeStatus::ScheduleFailed
    );
    let logs = fixture
        .storage
        .operation_log_query_by_time(0, i64::MAX)
        .unwrap();
    assert_eq!(logs.len(), 1);
    assert!(logs[0]
        .detail
        .as_deref()
        .unwrap()
        .contains("schedule_failed"));
}

#[tokio::test]
async fn observer_reconstructs_committed_result_after_active_publish() {
    let fixture = Fixture::new();
    let tasks = fixture.tasks();
    let guard = fixture.lock();
    let prepared = task("upgrade-committed-observed");
    tasks.create(&guard, &prepared).unwrap();
    tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Accepted, 110)
        .unwrap();
    tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Stopping, 120)
        .unwrap();
    tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Installing, 130)
        .unwrap();
    tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Migrating, 140)
        .unwrap();
    tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Starting, 150)
        .unwrap();
    let health_checking = tasks
        .transition(
            &guard,
            &prepared.upgrade_id,
            UpgradeStatus::HealthChecking,
            160,
        )
        .unwrap();
    fixture
        .releases()
        .commit(&guard, &active(Some(&prepared.upgrade_id), "3.0.2", 200))
        .unwrap();
    drop(guard);

    fixture
        .importer
        .monitor_active_task(health_checking)
        .await
        .unwrap();

    let stored = fixture
        .results()
        .get(&prepared.upgrade_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, UpgradeStatus::Committed);
    assert_eq!(
        tasks.history(&prepared.upgrade_id).unwrap().unwrap().status,
        UpgradeStatus::Committed
    );
    assert_eq!(fixture.storage.operation_log_count().unwrap(), 1);
}

#[tokio::test]
async fn observer_does_not_scan_or_import_unrelated_results() {
    let fixture = Fixture::new();
    fixture
        .releases()
        .commit(&fixture.lock(), &active(None, "3.0.1", 50))
        .unwrap();
    let guard = fixture.lock();
    fixture
        .results()
        .write(&guard, &result("unrelated-upgrade", UpgradeStatus::Failed))
        .unwrap();
    let tasks = fixture.tasks();
    let prepared = task("observed-upgrade");
    tasks.create(&guard, &prepared).unwrap();
    let accepted = tasks
        .transition(&guard, &prepared.upgrade_id, UpgradeStatus::Accepted, 110)
        .unwrap();
    fixture
        .results()
        .write(&guard, &result(&prepared.upgrade_id, UpgradeStatus::Failed))
        .unwrap();
    drop(guard);

    fixture
        .importer
        .monitor_active_task(accepted)
        .await
        .unwrap();

    assert_eq!(fixture.storage.operation_log_count().unwrap(), 1);
    assert!(!fixture
        .root
        .path()
        .join("imports/unrelated-upgrade.done")
        .exists());
}
