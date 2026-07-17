mod support;

use std::fs;
use std::sync::Arc;

use support::{
    target_release, version, FakeClock, FakeCommandRunner, FakePackageRevalidator,
    FakeUpgradeDatabase, TEST_CERTIFICATE_PEM,
};
use system_upgrade::{
    certificate_sha256, UpgradeResultStore, UpgradeStateLock, UpgradeStatus, UpgradeTask,
    UpgradeTaskStore,
};
use usb_control_updater::{UpgradeExecutor, UpgradePaths};

fn arrange() -> (
    tempfile::TempDir,
    UpgradePaths,
    FakePackageRevalidator,
    FakeUpgradeDatabase,
) {
    let dir = tempfile::tempdir().unwrap();
    let paths = UpgradePaths::for_root(dir.path().to_path_buf());
    let tasks = UpgradeTaskStore::new(paths.root.clone()).unwrap();
    let guard = UpgradeStateLock::acquire(&paths.root).unwrap();
    let task = UpgradeTask {
        format_version: 1,
        upgrade_id: "upgrade-test".into(),
        status: UpgradeStatus::Prepared,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        package_sha256: "a".repeat(64),
        created_at: 100,
        updated_at: 100,
    };
    tasks.create(&guard, &task).unwrap();
    tasks
        .transition(&guard, "upgrade-test", UpgradeStatus::Accepted, 101)
        .unwrap();
    fs::create_dir_all(paths.staging_dir.join("upgrade-test")).unwrap();
    fs::write(
        paths.staging_dir.join("upgrade-test/payload.deb"),
        b"candidate",
    )
    .unwrap();
    for path in [
        &paths.ready_file,
        &paths.installed_release,
        &paths.tls_certificate,
    ] {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
    }
    let tls = certificate_sha256(TEST_CERTIFICATE_PEM.as_bytes()).unwrap();
    let release = target_release(tls);
    fs::write(
        &paths.installed_release,
        serde_json::to_vec(&release).unwrap(),
    )
    .unwrap();
    fs::write(&paths.tls_certificate, TEST_CERTIFICATE_PEM).unwrap();
    fs::write(
        &paths.ready_file,
        serde_json::to_vec(&system_upgrade::ServiceReady {
            format_version: 1,
            version: version("3.0.2"),
            schema_version: 1,
            pid: 42,
            started_at: 200,
        })
        .unwrap(),
    )
    .unwrap();
    (
        dir,
        paths,
        FakePackageRevalidator {
            fail: false,
            target_release: release,
        },
        FakeUpgradeDatabase::new("3.0.1", 1),
    )
}

fn success_runner() -> FakeCommandRunner {
    let runner = FakeCommandRunner::default();
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    runner
}

#[test]
fn successful_upgrade_runs_one_install_chain_and_commits() {
    let (_dir, paths, revalidator, database) = arrange();
    let runner = success_runner();
    runner.observe_path(paths.managed_marker.clone());
    let report = UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        Arc::new(database.clone()),
        FakeClock::fixed(200),
    )
    .execute("upgrade-test")
    .unwrap();

    assert!(report.post_commit_warning.is_none());
    let calls = runner.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.program.ends_with("dpkg")
                && call.args.first().is_some_and(|arg| arg == "--unpack"))
            .count(),
        1
    );
    assert_eq!(
        UpgradeTaskStore::new(paths.root.clone())
            .unwrap()
            .history("upgrade-test")
            .unwrap()
            .unwrap()
            .status,
        UpgradeStatus::Committed
    );
    assert_eq!(
        UpgradeResultStore::new(paths.root.clone())
            .unwrap()
            .get("upgrade-test")
            .unwrap()
            .unwrap()
            .status,
        UpgradeStatus::Committed
    );
    assert_eq!(database.state().system_version, "3.0.2");
    assert_eq!(database.read_count(), 1);
    assert_eq!(database.compare_count(), 1);
    assert!(runner.observations().into_iter().all(|exists| exists));
    assert!(!paths.managed_marker.exists());
}

#[test]
fn revalidation_failure_records_failed_without_commands() {
    let (_dir, paths, mut revalidator, database) = arrange();
    revalidator.fail = true;
    let runner = FakeCommandRunner::default();
    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        Arc::new(database),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());
    assert!(runner.calls().is_empty());
    assert!(!paths.managed_marker.exists());
    assert_failed(&paths, "revalidating");
}

#[test]
fn source_version_mismatch_fails_before_stopping_service() {
    let (_dir, paths, revalidator, database) = arrange();
    database.set_system_version("3.0.0");
    let runner = FakeCommandRunner::default();

    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        Arc::new(database.clone()),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());

    assert!(runner.calls().is_empty());
    assert_eq!(database.compare_count(), 0);
    assert_failed(&paths, "revalidating");
}

#[test]
fn same_version_target_fails_before_stopping_service() {
    let (_dir, paths, revalidator, database) = arrange();
    let current_path = paths.root.join("current.json");
    let mut task: UpgradeTask = serde_json::from_slice(&fs::read(&current_path).unwrap()).unwrap();
    task.target_version = version("3.0.1");
    fs::write(&current_path, serde_json::to_vec(&task).unwrap()).unwrap();
    let runner = FakeCommandRunner::default();

    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        Arc::new(database),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());

    assert!(runner.calls().is_empty());
    assert_failed(&paths, "revalidating");
}

#[test]
fn stop_failure_records_failed_and_never_runs_dpkg() {
    let (_dir, paths, revalidator, database) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_failure("stopping");
    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        Arc::new(database),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());
    assert_eq!(runner.calls().len(), 1);
    assert!(!paths.managed_marker.exists());
    assert_failed(&paths, "stopping");
}

#[test]
fn install_failure_records_failed_without_version_commit() {
    let (_dir, paths, revalidator, database) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_success("");
    runner.push_failure("installing");
    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        Arc::new(database.clone()),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());
    assert_eq!(
        runner
            .calls()
            .iter()
            .filter(|call| call.program.ends_with("dpkg"))
            .count(),
        1
    );
    assert!(!paths.managed_marker.exists());
    assert_failed(&paths, "installing");
    assert_eq!(database.state().system_version, "3.0.1");
}

#[test]
fn migration_failure_records_failed_without_configure_or_start() {
    let (_dir, paths, revalidator, database) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_success("");
    runner.push_success("");
    runner.push_failure("migrating");
    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        Arc::new(database),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());
    assert_eq!(runner.calls().len(), 3);
    assert!(!paths.managed_marker.exists());
    assert_failed(&paths, "migrating");
}

#[test]
fn health_failure_records_failed_without_changing_database_version() {
    let (_dir, paths, revalidator, database) = arrange();
    let runner = FakeCommandRunner::default();
    for output in ["", "", "", "", "", "0\n", "", "active\n", "99\n", "0\n", ""] {
        runner.push_success(output);
    }
    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner,
        revalidator,
        Arc::new(database.clone()),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());
    assert_failed(&paths, "health_checking");
    assert_eq!(database.state().system_version, "3.0.1");
    assert_eq!(database.compare_count(), 0);
    assert!(!paths.managed_marker.exists());
}

#[test]
fn compare_and_set_failure_writes_failed_committing_result() {
    let (_dir, paths, revalidator, database) = arrange();
    database.fail_compare();
    let runner = success_runner();

    assert!(UpgradeExecutor::new(
        paths.clone(),
        runner,
        revalidator,
        Arc::new(database.clone()),
        FakeClock::fixed(200)
    )
    .execute("upgrade-test")
    .is_err());

    assert_eq!(database.state().system_version, "3.0.1");
    assert_eq!(database.compare_count(), 1);
    assert_failed(&paths, "committing");
    assert_ne!(
        UpgradeResultStore::new(paths.root)
            .unwrap()
            .get("upgrade-test")
            .unwrap()
            .unwrap()
            .status,
        UpgradeStatus::Committed
    );
}

fn assert_failed(paths: &UpgradePaths, stage: &str) {
    let task = UpgradeTaskStore::new(paths.root.clone())
        .unwrap()
        .history("upgrade-test")
        .unwrap()
        .unwrap();
    let result = UpgradeResultStore::new(paths.root.clone())
        .unwrap()
        .get("upgrade-test")
        .unwrap()
        .unwrap();
    assert_eq!(task.status, UpgradeStatus::Failed);
    assert_eq!(result.status, UpgradeStatus::Failed);
    assert_eq!(result.failed_stage.as_deref(), Some(stage));
    assert!(result.original_error.is_some());
}
