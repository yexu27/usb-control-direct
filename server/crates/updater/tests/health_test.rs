use system_upgrade::{ServiceReady, SystemVersion};
use usb_control_updater::{
    validate_health_snapshot, HealthExpectation, ServiceSnapshot, UpdaterError,
};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn expectation() -> HealthExpectation {
    HealthExpectation {
        version: version("3.0.2"),
        schema_version: 2,
        tls_cert_sha256: "a".repeat(64),
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
        installed_version: version("3.0.2"),
        tls_cert_sha256: "a".repeat(64),
        tls_handshake_ok: true,
    }
}

#[test]
fn rejects_ready_from_before_this_start_attempt() {
    let mut snapshot = matching();
    snapshot.ready.started_at = 99;
    assert!(matches!(
        validate_health_snapshot(&expectation(), &snapshot),
        Err(UpdaterError::HealthFailed(_))
    ));
}

#[test]
fn rejects_ready_pid_that_differs_from_systemd_main_pid() {
    let mut snapshot = matching();
    snapshot.ready.pid = 41;
    assert!(matches!(
        validate_health_snapshot(&expectation(), &snapshot),
        Err(UpdaterError::HealthFailed(_))
    ));
}

#[test]
fn rejects_version_schema_tls_or_restart_mismatch() {
    let mut snapshots = Vec::new();
    let mut wrong_version = matching();
    wrong_version.ready.version = version("3.0.1");
    snapshots.push(wrong_version);
    let mut wrong_schema = matching();
    wrong_schema.ready.schema_version = 1;
    snapshots.push(wrong_schema);
    let mut wrong_tls = matching();
    wrong_tls.tls_cert_sha256 = "b".repeat(64);
    snapshots.push(wrong_tls);
    let mut restarted = matching();
    restarted.restarts_after = 4;
    snapshots.push(restarted);
    for snapshot in snapshots {
        assert!(matches!(
            validate_health_snapshot(&expectation(), &snapshot),
            Err(UpdaterError::HealthFailed(_))
        ));
    }
}

#[test]
fn accepts_matching_fresh_ready_and_stable_restart_count() {
    validate_health_snapshot(&expectation(), &matching()).unwrap();
}
