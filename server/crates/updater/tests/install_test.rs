mod support;

use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use support::{version, FakeClock, FakeCommandRunner, FakeUpgradeDatabase, TEST_CERTIFICATE_PEM};
use system_upgrade::{certificate_sha256, InstalledRelease, ServiceReady};
use usb_control_updater::{InstallFinalizer, ManagedInstallGuard, UpgradePaths};

#[test]
fn managed_marker_is_0600_and_removed_on_success() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("run/managed");
    {
        let _guard = ManagedInstallGuard::create(&marker).unwrap();
        assert_eq!(
            fs::metadata(&marker).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::read_to_string(&marker).unwrap(),
            format!("{}\n", std::process::id())
        );
    }
    assert!(!marker.exists());
}

#[test]
fn managed_marker_is_removed_when_install_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("run/managed");

    let result: Result<(), &'static str> = {
        let _guard = ManagedInstallGuard::create(&marker).unwrap();
        Err("injected failure")
    };

    assert!(result.is_err());
    assert!(!marker.exists());
}

#[test]
fn managed_marker_is_removed_during_unwind() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("run/managed");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = ManagedInstallGuard::create(&marker).unwrap();
        panic!("injected panic");
    }));

    assert!(result.is_err());
    assert!(!marker.exists());
}

#[test]
fn direct_finalize_runs_migrate_reload_start_health_commit() {
    let fixture = FinalizerFixture::new(FakeClock::fixed(200));

    fixture.finalizer().finalize().unwrap();

    let calls = fixture.runner.calls();
    let programs_and_first_args = calls
        .iter()
        .map(|call| {
            (
                call.program.to_string_lossy().into_owned(),
                call.args
                    .first()
                    .map(|value| value.to_string_lossy().into_owned()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        programs_and_first_args,
        vec![
            (
                fixture.paths.migrator.to_string_lossy().into_owned(),
                Some(fixture.paths.database.to_string_lossy().into_owned()),
            ),
            ("systemctl".into(), Some("daemon-reload".into())),
            ("systemctl".into(), Some("show".into())),
            ("systemctl".into(), Some("start".into())),
            ("systemctl".into(), Some("is-active".into())),
            ("systemctl".into(), Some("show".into())),
            ("systemctl".into(), Some("show".into())),
            ("openssl".into(), Some("s_client".into())),
            ("/usr/bin/clamscan".into(), Some("--version".into())),
        ]
    );
}

#[test]
fn direct_install_commits_installed_and_clamav_state_after_health() {
    let fixture = FinalizerFixture::new(FakeClock::fixed(200));

    fixture.finalizer().finalize().unwrap();

    assert_eq!(
        fixture.database.install_state(),
        ("3.0.2".into(), "28063".into(), 1_784_298_268, 200)
    );
    assert_eq!(fixture.database.direct_commit_count(), 1);
}

#[test]
fn direct_finalize_stops_before_database_set_when_clock_fails() {
    let fixture = FinalizerFixture::new(FakeClock::sequence([200]));

    assert!(fixture.finalizer().finalize().is_err());

    assert_eq!(fixture.database.state().system_version, "3.0.1");
    assert_eq!(fixture.database.direct_commit_count(), 0);
}

#[test]
fn migration_failure_does_not_set_version() {
    let fixture = FinalizerFixture::new(FakeClock::fixed(200));
    fixture.runner.clear_outputs();
    fixture.runner.push_failure("migrating");

    assert!(fixture.finalizer().finalize().is_err());

    assert_eq!(fixture.database.state().system_version, "3.0.1");
}

#[test]
fn health_failure_does_not_set_version() {
    let fixture = FinalizerFixture::new(FakeClock::fixed(200));
    fixture.runner.clear_outputs();
    for output in ["", "", "0\n", "", "active\n", "99\n", "0\n", ""] {
        fixture.runner.push_success(output);
    }

    assert!(fixture.finalizer().finalize().is_err());

    assert_eq!(fixture.database.state().system_version, "3.0.1");
    assert_eq!(fixture.database.direct_commit_count(), 0);
}

#[test]
fn clamav_status_failure_does_not_commit_install_state() {
    let fixture = FinalizerFixture::new(FakeClock::fixed(200));
    fixture.runner.clear_outputs();
    for output in ["", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
        fixture.runner.push_success(output);
    }
    fixture.runner.push_failure("reading_virus_database_status");

    assert!(fixture.finalizer().finalize().is_err());

    assert_eq!(fixture.database.state().system_version, "3.0.1");
    assert_eq!(fixture.database.direct_commit_count(), 0);
}

#[test]
fn database_commit_failure_fails_finalize_install() {
    let fixture = FinalizerFixture::new(FakeClock::fixed(200));
    fixture.database.fail_direct_commit();

    assert!(fixture.finalizer().finalize().is_err());

    assert_eq!(fixture.database.state().system_version, "3.0.1");
}

struct FinalizerFixture {
    _temp: tempfile::TempDir,
    paths: UpgradePaths,
    runner: FakeCommandRunner,
    database: FakeUpgradeDatabase,
    clock: FakeClock,
}

impl FinalizerFixture {
    fn new(clock: FakeClock) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let paths = UpgradePaths::for_root(temp.path().join("upgrade"));
        let runner = FakeCommandRunner::default();
        for output in [
            "",
            "",
            "0\n",
            "",
            "active\n",
            "42\n",
            "0\n",
            "",
            "ClamAV 1.4.4/28063/Fri Jul 17 14:24:28 2026\n",
        ] {
            runner.push_success(output);
        }
        for path in [
            &paths.installed_release,
            &paths.ready_file,
            &paths.tls_certificate,
        ] {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
        }
        let tls_sha256 = certificate_sha256(TEST_CERTIFICATE_PEM.as_bytes()).unwrap();
        let installed = InstalledRelease {
            format_version: 1,
            product: "usb-control".into(),
            version: version("3.0.2"),
            architecture: "arm64".into(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            tls_cert_sha256: tls_sha256,
            upgrade_signing_key_id: "upgrade-next".into(),
        };
        fs::write(
            &paths.installed_release,
            serde_json::to_vec(&installed).unwrap(),
        )
        .unwrap();
        fs::write(&paths.tls_certificate, TEST_CERTIFICATE_PEM).unwrap();
        fs::write(
            &paths.ready_file,
            serde_json::to_vec(&ServiceReady {
                format_version: 1,
                version: installed.version,
                schema_version: installed.supported_schema_max,
                pid: 42,
                started_at: 200,
            })
            .unwrap(),
        )
        .unwrap();
        Self {
            _temp: temp,
            paths,
            runner,
            database: FakeUpgradeDatabase::new("3.0.1", 1),
            clock,
        }
    }

    fn finalizer(&self) -> InstallFinalizer<&FakeCommandRunner, &FakeClock> {
        InstallFinalizer::new(
            self.paths.clone(),
            &self.runner,
            Arc::new(self.database.clone()),
            &self.clock,
        )
    }
}
