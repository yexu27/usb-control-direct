mod support;

use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};

use support::{version, FakeClock, FakeCommandRunner, TEST_CERTIFICATE_PEM};
use system_upgrade::{certificate_sha256, ActiveReleaseStore, InstalledRelease, ServiceReady};
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
        ]
    );
}

#[test]
fn direct_finalize_commits_without_online_upgrade_id() {
    let fixture = FinalizerFixture::new(FakeClock::fixed(200));

    fixture.finalizer().finalize().unwrap();

    let active = ActiveReleaseStore::new(fixture.paths.root.clone())
        .unwrap()
        .current()
        .unwrap()
        .unwrap();
    assert_eq!(active.version, version("3.0.2"));
    assert_eq!(active.schema_version, 2);
    assert_eq!(active.online_upgrade_id, None);
}

#[test]
fn direct_finalize_stops_before_commit_when_clock_fails() {
    let fixture = FinalizerFixture::new(FakeClock::sequence([200]));

    assert!(fixture.finalizer().finalize().is_err());

    assert!(ActiveReleaseStore::new(fixture.paths.root.clone())
        .unwrap()
        .current()
        .unwrap()
        .is_none());
}

struct FinalizerFixture {
    _temp: tempfile::TempDir,
    paths: UpgradePaths,
    runner: FakeCommandRunner,
    clock: FakeClock,
}

impl FinalizerFixture {
    fn new(clock: FakeClock) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let paths = UpgradePaths::for_root(temp.path().join("upgrade"));
        let runner = FakeCommandRunner::default();
        for output in ["", "", "0\n", "", "active\n", "42\n", "0\n", ""] {
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
            clock,
        }
    }

    fn finalizer(&self) -> InstallFinalizer<&FakeCommandRunner, &FakeClock> {
        InstallFinalizer::new(self.paths.clone(), &self.runner, &self.clock)
    }
}
