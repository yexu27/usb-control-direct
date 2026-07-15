use system_upgrade::{InstalledRelease, ServiceReady, SystemVersion};
use usb_control_updater::{
    validate_health_snapshot, HealthExpectation, ServiceSnapshot, UpdaterError,
};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn release() -> InstalledRelease {
    InstalledRelease {
        format_version: 1,
        product: "usb-control".into(),
        version: version("3.0.2"),
        architecture: "arm64".into(),
        supported_schema_min: 1,
        supported_schema_max: 2,
        tls_cert_sha256: "a".repeat(64),
        upgrade_signing_key_id: "release-1".into(),
    }
}

fn expectation() -> HealthExpectation {
    HealthExpectation {
        release: release(),
        schema_version: 2,
        start_attempt_at: 100,
        restarts_before: 3,
    }
}

fn matching() -> ServiceSnapshot {
    ServiceSnapshot {
        active: true,
        main_pid: 42,
        restarts_after: 3,
        ready: ServiceReady {
            format_version: 1,
            version: version("3.0.2"),
            schema_version: 2,
            pid: 42,
            started_at: 100,
        },
        installed_release: release(),
        tls_cert_sha256: "a".repeat(64),
        tls_handshake_ok: true,
    }
}

#[test]
fn accepts_matching_target_release_before_active_release_commit() {
    validate_health_snapshot(&expectation(), &matching()).unwrap();
}

#[test]
fn rejects_ready_or_installed_release_mismatch() {
    let mut ready = matching();
    ready.ready.version = version("3.0.1");
    assert!(matches!(
        validate_health_snapshot(&expectation(), &ready),
        Err(UpdaterError::HealthFailed(_))
    ));

    let mut installed = matching();
    installed.installed_release.version = version("3.0.1");
    assert!(matches!(
        validate_health_snapshot(&expectation(), &installed),
        Err(UpdaterError::HealthFailed(_))
    ));
}

#[test]
fn rejects_stale_pid_schema_tls_or_restart() {
    let mut cases = Vec::new();
    let mut stale = matching();
    stale.ready.started_at = 99;
    cases.push(stale);
    let mut pid = matching();
    pid.ready.pid = 41;
    cases.push(pid);
    let mut schema = matching();
    schema.ready.schema_version = 1;
    cases.push(schema);
    let mut tls = matching();
    tls.tls_cert_sha256 = "b".repeat(64);
    cases.push(tls);
    let mut restart = matching();
    restart.restarts_after = 4;
    cases.push(restart);
    for snapshot in cases {
        assert!(matches!(
            validate_health_snapshot(&expectation(), &snapshot),
            Err(UpdaterError::HealthFailed(_))
        ));
    }
}
