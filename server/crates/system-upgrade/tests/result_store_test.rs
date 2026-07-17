use std::fs;

use system_upgrade::{
    SystemVersion, UpgradeResult, UpgradeResultStore, UpgradeStateLock, UpgradeStatus,
};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn result(upgrade_id: &str, status: UpgradeStatus, finished_at: i64) -> UpgradeResult {
    let failed = matches!(
        status,
        UpgradeStatus::ScheduleFailed | UpgradeStatus::Failed
    );
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
        failed_stage: failed.then(|| "installing".into()),
        original_error: failed.then(|| "injected failure".into()),
        finished_at,
    }
}

#[test]
fn upgrade_result_store_writes_results_under_results_directory() {
    let dir = tempfile::tempdir().unwrap();
    let store = UpgradeResultStore::new(dir.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(dir.path()).unwrap();
    let value = result("upgrade-result", UpgradeStatus::Committed, 200);
    store.write(&guard, &value).unwrap();

    assert_eq!(store.get("upgrade-result").unwrap(), Some(value));
    assert!(dir
        .path()
        .join("results/upgrade-result.result.json")
        .is_file());
    assert!(!dir
        .path()
        .join("history/upgrade-result.result.json")
        .exists());
}

#[test]
fn result_validation_matches_first_release_terminal_contract() {
    let dir = tempfile::tempdir().unwrap();
    let store = UpgradeResultStore::new(dir.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(dir.path()).unwrap();

    for status in [
        UpgradeStatus::Committed,
        UpgradeStatus::ScheduleFailed,
        UpgradeStatus::Failed,
    ] {
        store
            .write(&guard, &result(&format!("upgrade-{status:?}"), status, 200))
            .unwrap();
    }

    let mut committed_with_error = result("bad-committed", UpgradeStatus::Committed, 200);
    committed_with_error.original_error = Some("must be rejected".into());
    assert!(store.write(&guard, &committed_with_error).is_err());

    let mut failed_without_stage = result("bad-failed", UpgradeStatus::Failed, 200);
    failed_without_stage.failed_stage = None;
    assert!(store.write(&guard, &failed_without_stage).is_err());
}

#[test]
fn schedule_failed_and_failed_are_business_log_importable_when_observed() {
    assert!(result("schedule", UpgradeStatus::ScheduleFailed, 200).is_business_log_importable());
    assert!(result("failed", UpgradeStatus::Failed, 200).is_business_log_importable());
}

#[test]
fn result_store_keeps_latest_twenty() {
    let dir = tempfile::tempdir().unwrap();
    let store = UpgradeResultStore::new(dir.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(dir.path()).unwrap();
    for index in 0..22 {
        store
            .write(
                &guard,
                &result(
                    &format!("upgrade-{index:02}"),
                    UpgradeStatus::Committed,
                    100 + index,
                ),
            )
            .unwrap();
    }
    assert_eq!(
        fs::read_dir(dir.path().join("results")).unwrap().count(),
        20
    );
    assert!(store.get("upgrade-00").unwrap().is_none());
    assert!(store.get("upgrade-21").unwrap().is_some());
}
