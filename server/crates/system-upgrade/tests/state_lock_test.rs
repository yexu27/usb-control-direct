use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use system_upgrade::{
    SystemVersion, UpgradeError, UpgradeStateLock, UpgradeStatus, UpgradeTask, UpgradeTaskStore,
};
use tempfile::TempDir;

#[test]
fn second_process_cannot_acquire_state_lock() {
    let root = TempDir::new().unwrap();
    let _guard = UpgradeStateLock::acquire(root.path()).unwrap();

    let status = Command::new(env::current_exe().unwrap())
        .args(["--exact", "lock_helper_process", "--nocapture"])
        .env("USB_CONTROL_STATE_LOCK_ROOT", root.path())
        .status()
        .unwrap();

    assert!(status.success());
}

#[test]
fn lock_helper_process() {
    let Some(root) = env::var_os("USB_CONTROL_STATE_LOCK_ROOT") else {
        return;
    };

    assert!(matches!(
        UpgradeStateLock::acquire(root),
        Err(UpgradeError::Busy)
    ));
}

#[test]
fn lock_file_is_state_lock_with_mode_0600() {
    let root = TempDir::new().unwrap();
    let _guard = UpgradeStateLock::acquire(root.path()).unwrap();

    let metadata = fs::metadata(root.path().join("state.lock")).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

#[test]
fn store_rejects_guard_from_a_different_root() {
    let first = TempDir::new().unwrap();
    let second = TempDir::new().unwrap();
    let store = UpgradeTaskStore::new(first.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(second.path()).unwrap();

    let error = store.create(&guard, &prepared_task("upgrade-root-mismatch"));

    assert!(matches!(error, Err(UpgradeError::State(_))));
    assert!(store.current().unwrap().is_none());
}

#[test]
fn same_terminal_transition_is_idempotent_and_cleans_current() {
    let root = TempDir::new().unwrap();
    let store = UpgradeTaskStore::new(root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(root.path()).unwrap();
    let task = prepared_task("upgrade-terminal-idempotent");
    store.create(&guard, &task).unwrap();
    let terminal = store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Cancelled, 101)
        .unwrap();

    fs::copy(
        root.path()
            .join("history")
            .join(format!("{}.json", task.upgrade_id)),
        root.path().join("current.json"),
    )
    .unwrap();

    let repeated = store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Cancelled, 102)
        .unwrap();

    assert_eq!(repeated, terminal);
    assert!(store.current().unwrap().is_none());
}

#[test]
fn different_terminal_transition_is_rejected() {
    let root = TempDir::new().unwrap();
    let store = UpgradeTaskStore::new(root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(root.path()).unwrap();
    let task = prepared_task("upgrade-terminal-conflict");
    store.create(&guard, &task).unwrap();
    store
        .ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Cancelled, 101)
        .unwrap();

    let error = store.ensure_terminal(&guard, &task.upgrade_id, UpgradeStatus::Failed, 102);

    assert!(matches!(error, Err(UpgradeError::State(_))));
}

fn prepared_task(upgrade_id: &str) -> UpgradeTask {
    UpgradeTask {
        format_version: 1,
        upgrade_id: upgrade_id.into(),
        status: UpgradeStatus::Prepared,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: SystemVersion::parse("3.0.1").unwrap(),
        target_version: SystemVersion::parse("3.0.2").unwrap(),
        package_sha256: "a".repeat(64),
        created_at: 100,
        updated_at: 100,
    }
}
