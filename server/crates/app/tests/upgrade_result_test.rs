use std::sync::{Arc, Mutex};

use storage::model::OperationLogInsert;
use storage::{InsertOnceResult, Storage};
use storage_test_support::TestDb;
use system_upgrade::{
    ActiveRelease, ReleaseStateStore, SystemVersion, UpgradeResult, UpgradeStatus, UpgradeTask,
};
use usb_control_app::upgrade_result::{
    ImportDisposition, TokioUpgradeResultObserver, UpgradeImportStorage, UpgradeResultImporter,
    UpgradeResultObserver,
};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn active(upgrade_id: &str, version_value: &str) -> ActiveRelease {
    ActiveRelease {
        format_version: 1,
        upgrade_id: upgrade_id.into(),
        version: version(version_value),
        deb_sha256: "a".repeat(64),
        schema_version: 1,
        committed_at: 200,
    }
}

fn result(upgrade_id: &str, status: UpgradeStatus) -> UpgradeResult {
    UpgradeResult {
        format_version: 1,
        upgrade_id: upgrade_id.into(),
        status,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        effective_version: if status == UpgradeStatus::Committed {
            version("3.0.2")
        } else {
            version("3.0.1")
        },
        failed_stage: (status != UpgradeStatus::Committed).then(|| "installing".into()),
        original_error: (status != UpgradeStatus::Committed).then(|| "候选版本安装失败".into()),
        rollback_error: None,
        finished_at: 210,
    }
}

fn task(upgrade_id: &str, status: UpgradeStatus) -> UpgradeTask {
    UpgradeTask {
        format_version: 1,
        upgrade_id: upgrade_id.into(),
        status,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        package_sha256: "b".repeat(64),
        created_at: 100,
        updated_at: 190,
    }
}

fn arrange() -> (
    tempfile::TempDir,
    TestDb,
    Arc<Storage>,
    UpgradeResultImporter,
) {
    let root = tempfile::tempdir().unwrap();
    let db = TestDb::new();
    let storage = Arc::new(Storage::open(db.path()).unwrap());
    storage.config_set("system_version", "3.0.1").unwrap();
    let importer = UpgradeResultImporter::new(root.path().to_path_buf(), Arc::clone(&storage));
    (root, db, storage, importer)
}

fn write_active_and_result(root: &std::path::Path, value: &UpgradeResult) {
    let store = ReleaseStateStore::new(root.to_path_buf()).unwrap();
    store
        .commit_active_release(&active(
            &value.upgrade_id,
            &value.effective_version.to_string(),
        ))
        .unwrap();
    store.write_result(value).unwrap();
}

#[test]
fn imports_committed_result_once_and_syncs_version() {
    let (root, _db, storage, importer) = arrange();
    let value = result("upgrade-committed", UpgradeStatus::Committed);
    write_active_and_result(root.path(), &value);

    assert_eq!(
        importer.import_result(&value).unwrap(),
        ImportDisposition::Imported
    );
    assert_eq!(
        importer.import_result(&value).unwrap(),
        ImportDisposition::AlreadyImported
    );
    assert_eq!(storage.operation_log_count().unwrap(), 1);
    assert_eq!(
        storage
            .config_get("system_version")
            .unwrap()
            .unwrap()
            .config_value
            .as_deref(),
        Some("3.0.2")
    );
    assert!(root.path().join("imports/upgrade-committed.done").is_file());
}

#[test]
fn imports_rolled_back_result_with_original_failure() {
    let (root, _db, storage, importer) = arrange();
    let value = result("upgrade-rolled-back", UpgradeStatus::RolledBack);
    write_active_and_result(root.path(), &value);
    importer.import_result(&value).unwrap();

    let logs = storage.operation_log_query_by_time(0, i64::MAX).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].result, 1);
    assert_eq!(logs[0].fail_reason.as_deref(), Some("候选版本安装失败"));
    assert!(logs[0].detail.as_deref().unwrap().contains("rolled_back"));
}

#[test]
fn imports_schedule_failed_result_once() {
    let (root, _db, storage, importer) = arrange();
    let value = result("upgrade-schedule-failed", UpgradeStatus::ScheduleFailed);
    write_active_and_result(root.path(), &value);
    importer.import_result(&value).unwrap();
    importer.import_result(&value).unwrap();
    assert_eq!(storage.operation_log_count().unwrap(), 1);
}

#[test]
fn does_not_import_rollback_failed_into_business_log() {
    let (root, _db, storage, importer) = arrange();
    let value = result("upgrade-rollback-failed", UpgradeStatus::RollbackFailed);
    write_active_and_result(root.path(), &value);
    assert_eq!(
        importer.import_result(&value).unwrap(),
        ImportDisposition::NotImportable
    );
    assert_eq!(storage.operation_log_count().unwrap(), 0);
    assert!(!root
        .path()
        .join("imports/upgrade-rollback-failed.done")
        .exists());
}

#[test]
fn repeated_startup_scan_does_not_duplicate_audit_log() {
    let (root, _db, storage, importer) = arrange();
    let value = result("upgrade-scan", UpgradeStatus::Committed);
    write_active_and_result(root.path(), &value);
    assert_eq!(importer.scan_pending().unwrap(), 1);
    assert_eq!(importer.scan_pending().unwrap(), 0);
    assert_eq!(storage.operation_log_count().unwrap(), 1);
}

struct FailingStorage {
    insert_result: Mutex<Result<InsertOnceResult, String>>,
    sync_results: Mutex<Vec<Result<(), String>>>,
}

struct RecoveringStorage {
    insert_results: Mutex<Vec<Result<InsertOnceResult, String>>>,
}

impl UpgradeImportStorage for RecoveringStorage {
    fn insert_once(&self, _item: &OperationLogInsert) -> Result<InsertOnceResult, String> {
        self.insert_results.lock().unwrap().remove(0)
    }

    fn sync_system_version(&self, _version: &str) -> Result<(), String> {
        Ok(())
    }
}

impl UpgradeImportStorage for FailingStorage {
    fn insert_once(&self, _item: &OperationLogInsert) -> Result<InsertOnceResult, String> {
        self.insert_result.lock().unwrap().clone()
    }

    fn sync_system_version(&self, _version: &str) -> Result<(), String> {
        self.sync_results.lock().unwrap().remove(0)
    }
}

#[test]
fn keeps_importable_result_pending_when_business_log_write_fails() {
    let root = tempfile::tempdir().unwrap();
    let value = result("upgrade-log-failure", UpgradeStatus::Committed);
    let store = ReleaseStateStore::new(root.path().to_path_buf()).unwrap();
    store
        .commit_active_release(&active(&value.upgrade_id, "3.0.2"))
        .unwrap();
    store.write_result(&value).unwrap();
    let importer = UpgradeResultImporter::with_storage(
        root.path().to_path_buf(),
        Arc::new(FailingStorage {
            insert_result: Mutex::new(Err("injected log failure".into())),
            sync_results: Mutex::new(vec![Ok(())]),
        }),
    );
    assert!(importer.import_result(&value).is_err());
    assert!(root
        .path()
        .join("history/upgrade-log-failure.result.json")
        .is_file());
    assert!(!root
        .path()
        .join("imports/upgrade-log-failure.done")
        .exists());
}

#[test]
fn unacknowledged_importable_history_is_not_pruned() {
    let root = tempfile::tempdir().unwrap();
    let value = result("upgrade-unacknowledged", UpgradeStatus::Committed);
    let store = ReleaseStateStore::new(root.path().to_path_buf()).unwrap();
    store
        .commit_active_release(&active(&value.upgrade_id, "3.0.2"))
        .unwrap();
    store.write_result(&value).unwrap();
    let importer = UpgradeResultImporter::with_storage(
        root.path().to_path_buf(),
        Arc::new(FailingStorage {
            insert_result: Mutex::new(Err("injected log failure".into())),
            sync_results: Mutex::new(vec![Ok(())]),
        }),
    );

    assert!(importer.scan_pending().is_err());
    assert!(root
        .path()
        .join("history/upgrade-unacknowledged.result.json")
        .is_file());
    assert!(!root
        .path()
        .join("imports/upgrade-unacknowledged.done")
        .exists());
}

#[tokio::test(start_paused = true)]
async fn startup_scan_retries_pending_result_until_done() {
    let root = tempfile::tempdir().unwrap();
    let value = result("upgrade-startup-retry", UpgradeStatus::Committed);
    let store = ReleaseStateStore::new(root.path().to_path_buf()).unwrap();
    store
        .commit_active_release(&active(&value.upgrade_id, "3.0.2"))
        .unwrap();
    store.write_result(&value).unwrap();
    let importer = Arc::new(UpgradeResultImporter::with_storage(
        root.path().to_path_buf(),
        Arc::new(RecoveringStorage {
            insert_results: Mutex::new(vec![
                Err("injected first attempt failure".into()),
                Ok(InsertOnceResult::Inserted(9)),
            ]),
        }),
    ));

    assert!(importer.scan_pending().is_err());
    let retry = tokio::spawn(async move { importer.retry_pending_until_done().await });
    tokio::task::yield_now().await;
    assert!(!retry.is_finished());
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    retry.await.unwrap().unwrap();
    assert!(root
        .path()
        .join("imports/upgrade-startup-retry.done")
        .is_file());
}

#[test]
fn retries_version_copy_when_log_exists_but_config_sync_failed() {
    let root = tempfile::tempdir().unwrap();
    let value = result("upgrade-sync-retry", UpgradeStatus::Committed);
    ReleaseStateStore::new(root.path().to_path_buf())
        .unwrap()
        .commit_active_release(&active(&value.upgrade_id, "3.0.2"))
        .unwrap();
    let importer = UpgradeResultImporter::with_storage(
        root.path().to_path_buf(),
        Arc::new(FailingStorage {
            insert_result: Mutex::new(Ok(InsertOnceResult::AlreadyExists(7))),
            sync_results: Mutex::new(vec![Err("injected config failure".into()), Ok(())]),
        }),
    );
    assert!(importer.import_result(&value).is_err());
    assert!(!root.path().join("imports/upgrade-sync-retry.done").exists());
    assert_eq!(
        importer.import_result(&value).unwrap(),
        ImportDisposition::AlreadyImported
    );
    assert!(root
        .path()
        .join("imports/upgrade-sync-retry.done")
        .is_file());
}

#[test]
fn concurrent_result_import_inserts_one_business_log() {
    let (root, _db, storage, importer) = arrange();
    let value = result("upgrade-concurrent", UpgradeStatus::Committed);
    write_active_and_result(root.path(), &value);
    let importer = Arc::new(importer);
    let value = Arc::new(value);
    let threads = (0..2)
        .map(|_| {
            let importer = Arc::clone(&importer);
            let value = Arc::clone(&value);
            std::thread::spawn(move || importer.import_result(&value))
        })
        .collect::<Vec<_>>();
    for thread in threads {
        thread.join().unwrap().unwrap();
    }
    assert_eq!(storage.operation_log_count().unwrap(), 1);
}

#[tokio::test(start_paused = true)]
async fn monitor_imports_terminal_result_written_after_service_start() {
    let (root, _db, storage, importer) = arrange();
    let active_store = ReleaseStateStore::new(root.path().to_path_buf()).unwrap();
    active_store
        .commit_active_release(&active("previous-upgrade", "3.0.1"))
        .unwrap();
    let active_task = task("upgrade-delayed-result", UpgradeStatus::RollingBack);
    fs_write_json(&root.path().join("current.json"), &active_task);

    let monitor = tokio::spawn(async move { importer.monitor_active_task(active_task).await });
    tokio::task::yield_now().await;
    assert!(!monitor.is_finished());

    active_store
        .write_result(&result("upgrade-delayed-result", UpgradeStatus::RolledBack))
        .unwrap();
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    monitor.await.unwrap().unwrap();
    assert_eq!(storage.operation_log_count().unwrap(), 1);
}

#[tokio::test(start_paused = true)]
async fn live_observer_imports_schedule_failed_without_restart() {
    let (root, _db, storage, importer) = arrange();
    ReleaseStateStore::new(root.path().to_path_buf())
        .unwrap()
        .commit_active_release(&active("previous-upgrade", "3.0.1"))
        .unwrap();
    let active_task = task("upgrade-live-schedule", UpgradeStatus::Accepted);
    fs_write_json(&root.path().join("current.json"), &active_task);
    TokioUpgradeResultObserver::new(Arc::new(importer)).observe(active_task.clone());
    tokio::task::yield_now().await;

    ReleaseStateStore::new(root.path().to_path_buf())
        .unwrap()
        .write_result(&result(
            "upgrade-live-schedule",
            UpgradeStatus::ScheduleFailed,
        ))
        .unwrap();
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    tokio::task::yield_now().await;

    assert_eq!(storage.operation_log_count().unwrap(), 1);
    assert!(root
        .path()
        .join("imports/upgrade-live-schedule.done")
        .is_file());
}

#[tokio::test(start_paused = true)]
async fn live_observer_imports_pre_stop_rolled_back_without_restart() {
    let (root, _db, storage, importer) = arrange();
    ReleaseStateStore::new(root.path().to_path_buf())
        .unwrap()
        .commit_active_release(&active("previous-upgrade", "3.0.1"))
        .unwrap();
    let active_task = task("upgrade-live-pre-stop", UpgradeStatus::Accepted);
    fs_write_json(&root.path().join("current.json"), &active_task);
    TokioUpgradeResultObserver::new(Arc::new(importer)).observe(active_task.clone());
    tokio::task::yield_now().await;

    ReleaseStateStore::new(root.path().to_path_buf())
        .unwrap()
        .write_result(&result("upgrade-live-pre-stop", UpgradeStatus::RolledBack))
        .unwrap();
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    tokio::task::yield_now().await;

    assert_eq!(storage.operation_log_count().unwrap(), 1);
}

#[tokio::test(start_paused = true)]
async fn live_observer_retries_storage_failure_until_done_marker() {
    let root = tempfile::tempdir().unwrap();
    let value = result("upgrade-live-retry", UpgradeStatus::RolledBack);
    let releases = ReleaseStateStore::new(root.path().to_path_buf()).unwrap();
    releases
        .commit_active_release(&active("previous-upgrade", "3.0.1"))
        .unwrap();
    let active_task = task("upgrade-live-retry", UpgradeStatus::Accepted);
    fs_write_json(&root.path().join("current.json"), &active_task);
    let importer = Arc::new(UpgradeResultImporter::with_storage(
        root.path().to_path_buf(),
        Arc::new(RecoveringStorage {
            insert_results: Mutex::new(vec![
                Err("injected first attempt failure".into()),
                Ok(InsertOnceResult::Inserted(9)),
            ]),
        }),
    ));
    TokioUpgradeResultObserver::new(importer).observe(active_task);
    tokio::task::yield_now().await;
    releases.write_result(&value).unwrap();

    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert!(!root.path().join("imports/upgrade-live-retry.done").exists());
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert!(root
        .path()
        .join("imports/upgrade-live-retry.done")
        .is_file());
}

#[tokio::test(start_paused = true)]
async fn live_observer_ignores_rollback_failed() {
    let (root, _db, storage, importer) = arrange();
    ReleaseStateStore::new(root.path().to_path_buf())
        .unwrap()
        .commit_active_release(&active("previous-upgrade", "3.0.1"))
        .unwrap();
    let active_task = task("upgrade-live-rollback-failed", UpgradeStatus::Accepted);
    fs_write_json(&root.path().join("current.json"), &active_task);
    TokioUpgradeResultObserver::new(Arc::new(importer)).observe(active_task);
    tokio::task::yield_now().await;
    ReleaseStateStore::new(root.path().to_path_buf())
        .unwrap()
        .write_result(&result(
            "upgrade-live-rollback-failed",
            UpgradeStatus::RollbackFailed,
        ))
        .unwrap();

    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert_eq!(storage.operation_log_count().unwrap(), 0);
    assert!(!root
        .path()
        .join("imports/upgrade-live-rollback-failed.done")
        .exists());
}

#[tokio::test(start_paused = true)]
async fn reconstructs_committed_result_when_active_release_matches_current_task() {
    let (root, _db, storage, importer) = arrange();
    let active_task = task("upgrade-reconstruct", UpgradeStatus::HealthChecking);
    fs_write_json(&root.path().join("current.json"), &active_task);
    let release_store = ReleaseStateStore::new(root.path().to_path_buf()).unwrap();
    release_store
        .commit_active_release(&active("upgrade-reconstruct", "3.0.2"))
        .unwrap();

    importer.monitor_active_task(active_task).await.unwrap();
    let reconstructed = release_store
        .result("upgrade-reconstruct")
        .unwrap()
        .unwrap();
    assert_eq!(reconstructed.status, UpgradeStatus::Committed);
    assert_eq!(storage.operation_log_count().unwrap(), 1);
}

fn fs_write_json(path: &std::path::Path, value: &impl serde::Serialize) {
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}
