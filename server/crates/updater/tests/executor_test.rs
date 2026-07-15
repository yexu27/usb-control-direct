mod support;

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use sha2::Digest;
use support::{
    signed_package, FailAfterPreservingPrevious, FailPrepareRepository, FakeClock,
    FakeCommandRunner, FakePackageRevalidator, MatchingDebInspector, PublishThenFailDirectorySync,
    TEST_CERTIFICATE_PEM,
};
use system_upgrade::{PackageStager, SystemVersion, UpgradeStatus, UpgradeTask, UpgradeTaskStore};
use tempfile::TempDir;
use usb_control_updater::{
    certificate_sha256, ExecutionDisposition, LkgRepository, PackageRevalidator, ServiceSnapshot,
    SharedPackageRevalidator, UpdaterError, UpgradeExecutor, UpgradePaths,
};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn accepted_task() -> UpgradeTask {
    UpgradeTask {
        format_version: 1,
        upgrade_id: "upgrade-test".into(),
        status: UpgradeStatus::Accepted,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        package_sha256: "a".repeat(64),
        created_at: 100,
        updated_at: 101,
    }
}

fn arrange() -> (TempDir, UpgradePaths) {
    let dir = tempfile::tempdir().unwrap();
    let paths = UpgradePaths::for_root(dir.path().to_path_buf());
    fs::create_dir_all(paths.staging_dir.join("upgrade-test")).unwrap();
    fs::create_dir_all(&paths.rollback_dir).unwrap();
    for path in [
        &paths.ready_file,
        &paths.install_version_file,
        &paths.tls_certificate,
    ] {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
    }
    fs::write(
        &paths.current_task,
        serde_json::to_vec(&accepted_task()).unwrap(),
    )
    .unwrap();
    fs::write(
        paths.staging_dir.join("upgrade-test/payload.deb"),
        b"candidate",
    )
    .unwrap();
    fs::write(&paths.last_known_good_deb, b"old-release").unwrap();
    let certificate_hash = certificate_sha256(TEST_CERTIFICATE_PEM.as_bytes()).unwrap();
    fs::write(
        &paths.last_known_good_metadata,
        serde_json::json!({
            "format_version": 1,
            "version": "3.0.1",
            "deb_sha256": hex::encode(sha2::Sha256::digest(b"old-release")),
            "schema_version": 1,
            "tls_cert_sha256": certificate_hash.clone(),
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        paths.staging_dir.join("upgrade-test/manifest.json"),
        serde_json::json!({
            "format_version": 1,
            "product": "usb-control",
            "package_version": "3.0.2",
            "architecture": "arm64",
            "minimum_current_version": "3.0.1",
            "protocol_version": 1,
            "tls_cert_sha256": certificate_hash.clone(),
            "deb_file": "usb-control_V3.0.2_arm64.deb",
            "deb_size": 9,
            "deb_sha256": hex::encode(sha2::Sha256::digest(b"candidate")),
            "schema_from": 1,
            "schema_to": 1,
            "signing_key_id": "release-1"
        })
        .to_string(),
    )
    .unwrap();
    let ready = ServiceSnapshot {
        active: true,
        main_pid: 42,
        restarts_after: 0,
        ready: system_upgrade::ServiceReady {
            format_version: 1,
            version: version("3.0.2"),
            schema_version: 1,
            pid: 42,
            started_at: 200,
        },
        installed_version: version("3.0.2"),
        tls_cert_sha256: certificate_hash,
        tls_handshake_ok: true,
    };
    fs::write(&paths.ready_file, serde_json::to_vec(&ready.ready).unwrap()).unwrap();
    fs::write(&paths.install_version_file, "3.0.2\n").unwrap();
    fs::write(&paths.tls_certificate, TEST_CERTIFICATE_PEM).unwrap();
    (dir, paths)
}

fn command_names(runner: &FakeCommandRunner) -> Vec<(String, Vec<OsString>)> {
    runner
        .calls()
        .into_iter()
        .map(|call| (call.program.to_string_lossy().into_owned(), call.args))
        .collect()
}

fn success_runner() -> FakeCommandRunner {
    let runner = FakeCommandRunner::default();
    // stop, unpack, migrate, configure, daemon-reload, pre-start NRestarts, start,
    // active, MainPID, post-start NRestarts, TLS probe
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    runner
}

fn executor(
    paths: UpgradePaths,
    runner: FakeCommandRunner,
) -> UpgradeExecutor<FakeCommandRunner, FakePackageRevalidator, FakeClock> {
    UpgradeExecutor::new(
        paths,
        runner,
        FakePackageRevalidator,
        FakeClock::new([200; 20]),
    )
}

fn ready_bytes(version_text: &str, started_at: i64) -> Vec<u8> {
    serde_json::to_vec(&system_upgrade::ServiceReady {
        format_version: 1,
        version: version(version_text),
        schema_version: 1,
        pid: 42,
        started_at,
    })
    .unwrap()
}

fn publish_old_release_on_start(
    runner: &FakeCommandRunner,
    paths: &UpgradePaths,
    start_call: usize,
    started_at: i64,
) {
    runner.write_on_call(
        start_call,
        paths.ready_file.clone(),
        ready_bytes("3.0.1", started_at),
    );
    runner.write_on_call(
        start_call,
        paths.install_version_file.clone(),
        b"3.0.1\n".to_vec(),
    );
}

fn arrange_installed_trust(
    paths: &UpgradePaths,
    installed_tls_sha256: &str,
) -> SharedPackageRevalidator {
    let install_meta = paths.root.join("installed-release.json");
    let active_key_id = paths.root.join("upgrade_verify.id");
    let verify_key_dir = paths.root.join("keys");
    fs::create_dir_all(&verify_key_dir).unwrap();
    fs::write(&active_key_id, "release-1\n").unwrap();
    fs::write(
        &install_meta,
        serde_json::json!({
            "format_version": 1,
            "product": "usb-control",
            "version": "3.0.1",
            "architecture": "arm64",
            "supported_schema_min": 1,
            "supported_schema_max": 1,
            "tls_cert_sha256": installed_tls_sha256,
            "upgrade_signing_key_id": "release-1"
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        &paths.active_release,
        serde_json::to_vec(&system_upgrade::ActiveRelease {
            format_version: 1,
            upgrade_id: "installed-release".into(),
            version: version("3.0.1"),
            deb_sha256: hex::encode(sha2::Sha256::digest(b"old-release")),
            schema_version: 1,
            committed_at: 99,
        })
        .unwrap(),
    )
    .unwrap();
    SharedPackageRevalidator::new(verify_key_dir, install_meta, active_key_id)
}

fn arrange_real_revalidator(paths: &UpgradePaths) -> SharedPackageRevalidator {
    let tls_sha256 = certificate_sha256(TEST_CERTIFICATE_PEM.as_bytes()).unwrap();
    let manifest_raw =
        fs::read(paths.staging_dir.join("upgrade-test").join("manifest.json")).unwrap();
    let (package, public_key) = signed_package(&manifest_raw, b"candidate");
    fs::remove_dir_all(paths.staging_dir.join("upgrade-test")).unwrap();
    PackageStager::new(paths.root.clone(), 128 * 1024 * 1024)
        .stage("upgrade-test", &package)
        .unwrap();
    let mut task = accepted_task();
    task.package_sha256 = hex::encode(sha2::Sha256::digest(&package));
    fs::write(&paths.current_task, serde_json::to_vec(&task).unwrap()).unwrap();

    let _ = arrange_installed_trust(paths, &tls_sha256);
    let key_dir = paths.root.join("keys");
    fs::write(key_dir.join("upgrade_verify.id"), "release-1\n").unwrap();
    fs::write(key_dir.join("upgrade_verify.pub"), public_key).unwrap();
    SharedPackageRevalidator::with_deb_inspector(
        key_dir,
        paths.root.join("installed-release.json"),
        paths.root.join("upgrade_verify.id"),
        128 * 1024 * 1024,
        Arc::new(MatchingDebInspector {
            tls_cert_sha256: tls_sha256,
        }),
    )
}

#[test]
fn successful_upgrade_runs_exact_command_sequence() {
    let (_dir, paths) = arrange();
    let runner = success_runner();
    let outcome = executor(paths.clone(), runner.clone())
        .execute("upgrade-test")
        .unwrap();
    assert_eq!(outcome, ExecutionDisposition::Committed);
    let calls = command_names(&runner);
    assert_eq!(
        calls[0],
        (
            "systemctl".into(),
            vec!["stop".into(), "usb-control.service".into()]
        )
    );
    assert_eq!(calls[1].0, "dpkg");
    assert_eq!(calls[1].1[0], "--unpack");
    assert_eq!(calls[2].0, "/opt/usb-control/bin/usb-control-db-migrate");
    assert_eq!(
        calls[2].1,
        vec![
            paths.database.as_os_str().to_os_string(),
            paths.sql_root.as_os_str().to_os_string()
        ]
    );
    assert_eq!(
        calls[3],
        (
            "dpkg".into(),
            vec!["--configure".into(), "usb-control".into()]
        )
    );
    assert_eq!(calls[4], ("systemctl".into(), vec!["daemon-reload".into()]));
    assert_eq!(
        calls[6],
        (
            "systemctl".into(),
            vec!["start".into(), "usb-control.service".into()]
        )
    );
    assert_eq!(calls.len(), 11);
}

#[test]
fn unpack_failure_reinstalls_last_known_good_deb() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_success(""); // stop candidate
    runner.push_failure("installing");
    // rollback stop/unpack/migrate/configure/reload/NRestarts/start + health
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    publish_old_release_on_start(&runner, &paths, 9, 200);
    let outcome = executor(paths.clone(), runner.clone())
        .execute("upgrade-test")
        .unwrap();
    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    let calls = runner.calls();
    let rollback_unpack = calls
        .iter()
        .rev()
        .find(|call| {
            call.program == Path::new("dpkg") && call.args.first() == Some(&"--unpack".into())
        })
        .unwrap();
    assert_eq!(
        rollback_unpack.args[1],
        paths.last_known_good_deb.as_os_str()
    );
}

#[test]
fn stop_failure_still_attempts_complete_rollback() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_error(UpdaterError::CommandTimeout {
        stage: "stopping".into(),
        program: "systemctl".into(),
    });
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    publish_old_release_on_start(&runner, &paths, 8, 200);
    let result = executor(paths.clone(), runner.clone())
        .execute("upgrade-test")
        .unwrap();
    assert_eq!(result, ExecutionDisposition::RolledBack);
    let calls = runner.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.args == ["stop", "usb-control.service"])
            .count(),
        2
    );
    assert!(calls.iter().any(|call| {
        call.program == Path::new("dpkg")
            && call.args
                == [
                    OsString::from("--unpack"),
                    paths.last_known_good_deb.as_os_str().to_os_string(),
                ]
    }));
    assert!(calls.iter().any(|call| call.args == ["daemon-reload"]));
    assert!(calls
        .iter()
        .any(|call| call.args == ["start", "usb-control.service"]));
}

#[test]
fn migration_failure_does_not_commit_active_release() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_success("");
    runner.push_success("");
    runner.push_failure("migrating");
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    publish_old_release_on_start(&runner, &paths, 10, 200);
    let result = executor(paths.clone(), runner).execute("upgrade-test");
    assert_eq!(result.unwrap(), ExecutionDisposition::RolledBack);
    assert!(!paths.active_release.exists());
}

#[test]
fn health_failure_stops_candidate_and_restores_old_service() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    for output in [
        "", "", "", "", "", "0\n", "", "active\n", "999\n", "0\n", "",
    ] {
        runner.push_success(output);
    }
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    publish_old_release_on_start(&runner, &paths, 18, 201);
    let result = executor(paths, runner.clone())
        .execute("upgrade-test")
        .unwrap();
    assert_eq!(result, ExecutionDisposition::RolledBack);
    let stops = runner
        .calls()
        .into_iter()
        .filter(|c| {
            c.program == Path::new("systemctl") && c.args == ["stop", "usb-control.service"]
        })
        .count();
    assert_eq!(stops, 2);
}

#[test]
fn rollback_failure_preserves_both_errors() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_success("");
    runner.push_failure("installing");
    runner.push_failure("rollback_stop");
    let error = executor(paths, runner).execute("upgrade-test").unwrap_err();
    match error {
        UpdaterError::RollbackFailed { original, rollback } => {
            assert!(original.contains("installing"));
            assert!(rollback.contains("rollback_stop"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn candidate_and_rollback_each_reject_ready_older_than_their_own_start_attempt() {
    let (_dir, paths) = arrange();
    fs::write(&paths.ready_file, ready_bytes("3.0.2", 199)).unwrap();
    let runner = FakeCommandRunner::default();
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    // rollback stop/unpack/migrate/configure/reload/restarts/start + health
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    runner.write_on_call(18, paths.ready_file.clone(), ready_bytes("3.0.1", 299));
    let error = UpgradeExecutor::new(
        paths,
        runner,
        FakePackageRevalidator,
        FakeClock::new([200, 200, 200, 200, 200, 200, 200, 300, 300, 300]),
    )
    .execute("upgrade-test")
    .unwrap_err();
    assert!(matches!(error, UpdaterError::RollbackFailed { .. }));
}

#[test]
fn command_arguments_are_never_interpreted_by_a_shell() {
    let (_dir, mut paths) = arrange();
    paths.migrator = "/opt/usb-control/bin/migrate;touch injected".into();
    let runner = success_runner();
    executor(paths, runner.clone())
        .execute("upgrade-test")
        .unwrap();
    assert!(runner
        .calls()
        .iter()
        .all(|call| call.program != Path::new("sh") && call.program != Path::new("/bin/sh")));
}

#[test]
fn lkg_promotion_failure_rolls_back_with_preserved_previous_deb() {
    let (_dir, paths) = arrange();
    let runner = success_runner();
    // rollback sequence after promotion failure
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    publish_old_release_on_start(&runner, &paths, 18, 201);
    let result = UpgradeExecutor::with_repository(
        paths.clone(),
        runner.clone(),
        FakePackageRevalidator,
        FakeClock::new([200, 200, 200, 200, 200, 200, 200, 201, 201, 201]),
        FailAfterPreservingPrevious,
    )
    .execute("upgrade-test")
    .unwrap();
    assert_eq!(result, ExecutionDisposition::RolledBack);
    let rollback_unpack = runner
        .calls()
        .into_iter()
        .rev()
        .find(|call| {
            call.program == Path::new("dpkg") && call.args.first() == Some(&"--unpack".into())
        })
        .unwrap();
    assert_eq!(rollback_unpack.args[1], paths.previous_deb.as_os_str());
    assert_eq!(
        fs::read(&paths.last_known_good_deb).unwrap(),
        b"old-release"
    );
}

#[test]
fn active_release_is_the_last_commit_point() {
    let (_dir, paths) = arrange();
    let runner = success_runner();
    executor(paths.clone(), runner)
        .execute("upgrade-test")
        .unwrap();
    let active: system_upgrade::ActiveRelease =
        serde_json::from_slice(&fs::read(paths.active_release).unwrap()).unwrap();
    assert_eq!(active.upgrade_id, "upgrade-test");
    assert_eq!(active.version, version("3.0.2"));
    assert_eq!(fs::read(paths.last_known_good_deb).unwrap(), b"candidate");
    assert!(!paths.previous_deb.exists());
}

#[test]
fn post_commit_result_write_failure_does_not_roll_back_healthy_release() {
    let (_dir, paths) = arrange();
    fs::create_dir_all(&paths.history_dir).unwrap();
    fs::create_dir(paths.history_dir.join("upgrade-test.result.json")).unwrap();
    let runner = success_runner();
    let result = executor(paths.clone(), runner.clone())
        .execute("upgrade-test")
        .unwrap();
    assert_eq!(result, ExecutionDisposition::CommittedResultPending);
    assert!(paths.active_release.exists());
    assert_eq!(
        runner
            .calls()
            .iter()
            .filter(|c| c.args == ["stop", "usb-control.service"])
            .count(),
        1
    );
}

#[test]
fn active_release_parent_fsync_failure_does_not_roll_back_committed_candidate() {
    let (_dir, paths) = arrange();
    let runner = success_runner();
    let result = UpgradeExecutor::with_components(
        paths.clone(),
        runner.clone(),
        FakePackageRevalidator,
        FakeClock::new([200; 9]),
        usb_control_updater::FileLkgRepository,
        PublishThenFailDirectorySync,
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(result, ExecutionDisposition::CommittedResultPending);
    let active: system_upgrade::ActiveRelease =
        serde_json::from_slice(&fs::read(&paths.active_release).unwrap()).unwrap();
    assert_eq!(active.version, version("3.0.2"));
    assert_eq!(fs::read(&paths.last_known_good_deb).unwrap(), b"candidate");
    assert!(!paths.previous_deb.exists());
    assert_eq!(
        runner
            .calls()
            .iter()
            .filter(|call| call.args == ["stop", "usb-control.service"])
            .count(),
        1
    );
    assert!(runner.calls().iter().all(|call| {
        !(call.program == Path::new("dpkg")
            && call.args
                == [
                    OsString::from("--unpack"),
                    paths.previous_deb.as_os_str().to_os_string(),
                ])
    }));
}

#[test]
fn successful_upgrade_records_each_actual_stage_and_completion_time() {
    let (_dir, paths) = arrange();
    let runner = success_runner();
    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner,
        FakePackageRevalidator,
        FakeClock::new([110, 120, 130, 140, 150, 160, 170, 180, 190]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::Committed);
    let history: UpgradeTask =
        serde_json::from_slice(&fs::read(paths.history_dir.join("upgrade-test.json")).unwrap())
            .unwrap();
    let active: system_upgrade::ActiveRelease =
        serde_json::from_slice(&fs::read(&paths.active_release).unwrap()).unwrap();
    let result: system_upgrade::UpgradeResult = serde_json::from_slice(
        &fs::read(paths.history_dir.join("upgrade-test.result.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(active.committed_at, 170);
    assert_eq!(history.status, UpgradeStatus::Committed);
    assert_eq!(history.updated_at, 180);
    assert_eq!(result.finished_at, 190);
    assert!(history.updated_at >= active.committed_at);
    assert!(result.finished_at >= history.updated_at);
}

#[test]
fn rollback_records_actual_rollback_and_completion_times() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_success("");
    runner.push_failure("installing");
    for output in ["", "", "", "", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        runner.push_success(output);
    }
    publish_old_release_on_start(&runner, &paths, 9, 140);
    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner,
        FakePackageRevalidator,
        FakeClock::new([110, 120, 130, 140, 150, 160]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    let history: UpgradeTask =
        serde_json::from_slice(&fs::read(paths.history_dir.join("upgrade-test.json")).unwrap())
            .unwrap();
    let result: system_upgrade::UpgradeResult = serde_json::from_slice(
        &fs::read(paths.history_dir.join("upgrade-test.result.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(history.status, UpgradeStatus::RolledBack);
    assert_eq!(history.updated_at, 150);
    assert_eq!(result.finished_at, 160);
    assert!(result.finished_at >= history.updated_at);
}

#[test]
fn rollback_failure_records_actual_failure_and_result_times() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    runner.push_success("");
    runner.push_failure("installing");
    runner.push_failure("rollback_stop");
    let error = UpgradeExecutor::new(
        paths.clone(),
        runner,
        FakePackageRevalidator,
        FakeClock::new([110, 120, 130, 140, 150]),
    )
    .execute("upgrade-test")
    .unwrap_err();

    assert!(matches!(error, UpdaterError::RollbackFailed { .. }));
    let history: UpgradeTask =
        serde_json::from_slice(&fs::read(paths.history_dir.join("upgrade-test.json")).unwrap())
            .unwrap();
    let result: system_upgrade::UpgradeResult = serde_json::from_slice(
        &fs::read(paths.history_dir.join("upgrade-test.result.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(history.status, UpgradeStatus::RollbackFailed);
    assert_eq!(history.updated_at, 140);
    assert_eq!(result.finished_at, 150);
    assert!(result.finished_at >= history.updated_at);
}

#[test]
fn clock_failure_after_active_commit_never_rolls_back_the_candidate() {
    let (_dir, paths) = arrange();
    let runner = success_runner();
    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        FakePackageRevalidator,
        // 170 is captured before publishing active-release; every later clock read fails.
        FakeClock::new([110, 120, 130, 140, 150, 160, 170]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::Committed);
    let active: system_upgrade::ActiveRelease =
        serde_json::from_slice(&fs::read(&paths.active_release).unwrap()).unwrap();
    let history: UpgradeTask =
        serde_json::from_slice(&fs::read(paths.history_dir.join("upgrade-test.json")).unwrap())
            .unwrap();
    let result: system_upgrade::UpgradeResult = serde_json::from_slice(
        &fs::read(paths.history_dir.join("upgrade-test.result.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(active.committed_at, 170);
    assert!(history.updated_at >= active.committed_at);
    assert!(result.finished_at >= active.committed_at);
    assert_eq!(
        runner
            .calls()
            .iter()
            .filter(|call| call.args == ["stop", "usb-control.service"])
            .count(),
        1
    );
}

#[test]
fn production_revalidator_rejects_installed_tls_mismatch_before_stop() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    let revalidator = arrange_installed_trust(&paths, &"b".repeat(64));

    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        FakeClock::new([200; 20]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    assert_pre_stop_rolled_back(&paths, &runner, "revalidating");
}

#[test]
fn production_revalidator_rejects_malformed_lkg_tls_before_stop() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    let revalidator = arrange_installed_trust(&paths, &"b".repeat(64));
    let mut lkg: serde_json::Value =
        serde_json::from_slice(&fs::read(&paths.last_known_good_metadata).unwrap()).unwrap();
    lkg["tls_cert_sha256"] = serde_json::Value::String("ABC".into());
    fs::write(
        &paths.last_known_good_metadata,
        serde_json::to_vec(&lkg).unwrap(),
    )
    .unwrap();

    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        FakeClock::new([200; 20]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    assert_pre_stop_rolled_back(&paths, &runner, "revalidating");
}

#[test]
fn production_revalidator_reopens_and_verifies_the_current_task_package() {
    let (_dir, paths) = arrange();
    let revalidator = arrange_real_revalidator(&paths);
    let task: UpgradeTask =
        serde_json::from_slice(&fs::read(&paths.current_task).unwrap()).unwrap();

    let verified = revalidator.revalidate(&paths, &task).unwrap();

    assert_eq!(verified.manifest.package_version, task.target_version);
    assert_eq!(
        verified.candidate_deb,
        paths.staging_dir.join("upgrade-test/payload.deb")
    );
}

#[test]
fn production_revalidator_rejects_task_package_digest_mismatch_before_stop() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    let revalidator = arrange_real_revalidator(&paths);
    let mut task: UpgradeTask =
        serde_json::from_slice(&fs::read(&paths.current_task).unwrap()).unwrap();
    task.package_sha256 = "0".repeat(64);
    fs::write(&paths.current_task, serde_json::to_vec(&task).unwrap()).unwrap();

    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        FakeClock::new([200; 20]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    assert_pre_stop_rolled_back(&paths, &runner, "revalidating");
}

#[test]
fn production_revalidator_rejects_container_parser_violation_before_stop() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    let revalidator = arrange_real_revalidator(&paths);
    let package_path = paths.staging_dir.join("upgrade-test/package.bin");
    let mut package = fs::read(&package_path).unwrap();
    package.extend_from_slice(b"nonzero trailing data");
    fs::write(&package_path, &package).unwrap();
    let mut task: UpgradeTask =
        serde_json::from_slice(&fs::read(&paths.current_task).unwrap()).unwrap();
    task.package_sha256 = hex::encode(sha2::Sha256::digest(&package));
    fs::write(&paths.current_task, serde_json::to_vec(&task).unwrap()).unwrap();

    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        revalidator,
        FakeClock::new([200; 20]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    assert_pre_stop_rolled_back(&paths, &runner, "revalidating");
}

#[test]
fn production_revalidator_rejects_active_lkg_and_key_mismatches_before_stop() {
    for case in ["active", "lkg", "key"] {
        let (_dir, paths) = arrange();
        let runner = FakeCommandRunner::default();
        let revalidator = arrange_real_revalidator(&paths);
        match case {
            "active" => {
                let mut active: serde_json::Value =
                    serde_json::from_slice(&fs::read(&paths.active_release).unwrap()).unwrap();
                active["version"] = serde_json::Value::String("2.9.9".into());
                fs::write(&paths.active_release, serde_json::to_vec(&active).unwrap()).unwrap();
            }
            "lkg" => {
                let mut lkg: serde_json::Value =
                    serde_json::from_slice(&fs::read(&paths.last_known_good_metadata).unwrap())
                        .unwrap();
                lkg["deb_sha256"] = serde_json::Value::String("c".repeat(64));
                fs::write(
                    &paths.last_known_good_metadata,
                    serde_json::to_vec(&lkg).unwrap(),
                )
                .unwrap();
            }
            "key" => fs::write(paths.root.join("upgrade_verify.id"), "release-2\n").unwrap(),
            _ => unreachable!(),
        }

        let outcome = UpgradeExecutor::new(
            paths.clone(),
            runner.clone(),
            revalidator,
            FakeClock::new([200; 20]),
        )
        .execute("upgrade-test")
        .unwrap();
        assert_eq!(outcome, ExecutionDisposition::RolledBack, "case {case}");
        assert_pre_stop_rolled_back(&paths, &runner, "revalidating");
    }
}

#[test]
fn prepare_failure_finishes_rolled_back_without_stopping_service() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    let outcome = UpgradeExecutor::with_repository(
        paths.clone(),
        runner.clone(),
        FakePackageRevalidator,
        FakeClock::new([200; 5]),
        FailPrepareRepository,
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    assert_pre_stop_rolled_back(&paths, &runner, "preparing");
}

#[test]
fn clock_failure_after_prepare_aborts_candidate_and_finishes_rolled_back() {
    let (_dir, paths) = arrange();
    let runner = FakeCommandRunner::default();
    let outcome = UpgradeExecutor::new(
        paths.clone(),
        runner.clone(),
        FakePackageRevalidator,
        FakeClock::new([]),
    )
    .execute("upgrade-test")
    .unwrap();

    assert_eq!(outcome, ExecutionDisposition::RolledBack);
    assert_pre_stop_rolled_back(&paths, &runner, "preparing");
    assert!(!paths.next_last_known_good_deb.exists());
    assert!(!paths.managed_marker.exists());
}

#[test]
fn abort_prepared_removes_candidate_and_marker_and_is_idempotent() {
    let (_dir, paths) = arrange();
    fs::write(&paths.next_last_known_good_deb, b"candidate").unwrap();
    fs::create_dir_all(paths.managed_marker.parent().unwrap()).unwrap();
    fs::write(&paths.managed_marker, b"").unwrap();

    usb_control_updater::FileLkgRepository
        .abort_prepared(&paths)
        .unwrap();
    usb_control_updater::FileLkgRepository
        .abort_prepared(&paths)
        .unwrap();

    assert!(!paths.next_last_known_good_deb.exists());
    assert!(!paths.managed_marker.exists());
}

#[test]
fn second_task_can_be_accepted_after_pre_stop_failure() {
    let (_dir, paths) = arrange();
    UpgradeExecutor::new(
        paths.clone(),
        FakeCommandRunner::default(),
        FakePackageRevalidator,
        FakeClock::new([]),
    )
    .execute("upgrade-test")
    .unwrap();

    let store = UpgradeTaskStore::new(paths.root.clone()).unwrap();
    let mut second = accepted_task();
    second.upgrade_id = "upgrade-second".into();
    second.status = UpgradeStatus::Prepared;
    second.created_at = 300;
    second.updated_at = 300;
    store.create(&second).unwrap();
    store
        .transition("upgrade-second", UpgradeStatus::Accepted, 301)
        .unwrap();

    assert_eq!(
        store.current().unwrap().unwrap().status,
        UpgradeStatus::Accepted
    );
}

fn assert_pre_stop_rolled_back(
    paths: &UpgradePaths,
    runner: &FakeCommandRunner,
    failed_stage: &str,
) {
    assert!(runner
        .calls()
        .iter()
        .all(|call| call.program != Path::new("systemctl")));
    assert!(!paths.current_task.exists());
    let history: UpgradeTask =
        serde_json::from_slice(&fs::read(paths.history_dir.join("upgrade-test.json")).unwrap())
            .unwrap();
    assert_eq!(history.status, UpgradeStatus::RolledBack);
    let result: system_upgrade::UpgradeResult = serde_json::from_slice(
        &fs::read(paths.history_dir.join("upgrade-test.result.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(result.status, UpgradeStatus::RolledBack);
    assert_eq!(result.effective_version, version("3.0.1"));
    assert_eq!(result.failed_stage.as_deref(), Some(failed_stage));
    assert!(result.original_error.is_some());
}

#[test]
fn prepare_preserves_original_and_cleanup_errors_and_attempts_every_cleanup() {
    let (_dir, mut paths) = arrange();
    let cleanup_parent = paths.root.join("cleanup-parent");
    paths.managed_marker = cleanup_parent.clone();
    paths.next_last_known_good_deb = cleanup_parent.join("next.deb");
    let candidate_directory = paths.root.join("candidate-directory");
    fs::create_dir(&candidate_directory).unwrap();

    let error = usb_control_updater::FileLkgRepository
        .prepare(&paths, &candidate_directory, &"a".repeat(64))
        .unwrap_err();
    let message = error.to_string();

    assert!(
        message.contains("原始"),
        "missing original error: {message}"
    );
    assert!(message.contains("清理"), "missing cleanup error: {message}");
    assert!(
        message.contains("cleanup-parent"),
        "missing cleanup path: {message}"
    );
    assert!(!paths.next_last_known_good_deb.exists());
    assert!(paths.managed_marker.is_dir());
}
