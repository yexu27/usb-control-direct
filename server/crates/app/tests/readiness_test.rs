use std::fs;
use std::os::unix::fs::PermissionsExt;

use system_upgrade::{ServiceReady, SystemVersion};
use tempfile::tempdir;
use usb_control_app::readiness::ReadinessGuard;

fn ready_document() -> ServiceReady {
    ServiceReady {
        format_version: 1,
        version: SystemVersion::parse("3.1.0").unwrap(),
        schema_version: 2,
        pid: 4242,
        started_at: 1_721_000_000,
    }
}

#[test]
fn startup_removes_stale_ready_file_before_initialization() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ready.json");
    fs::write(&path, b"stale").unwrap();

    ReadinessGuard::clear_stale(&path).unwrap();

    assert!(!path.exists());
}

#[test]
fn publishes_ready_atomically_only_after_all_dependencies_are_ready() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ready.json");
    ReadinessGuard::clear_stale(&path).unwrap();

    assert!(!path.exists(), "startup must not publish readiness early");
    let guard = ReadinessGuard::publish(&path, &ready_document()).unwrap();

    assert!(path.is_file());
    assert_eq!(
        fs::read_dir(dir.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .len(),
        1,
        "atomic publication must not leave temporary files"
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    std::mem::forget(guard);
}

#[test]
fn readiness_guard_removes_ready_on_normal_drop() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ready.json");

    {
        let _guard = ReadinessGuard::publish(&path, &ready_document()).unwrap();
        assert!(path.exists());
    }

    assert!(!path.exists());
}

#[test]
fn ready_json_contains_version_schema_pid_and_started_at() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ready.json");
    let expected = ready_document();

    let guard = ReadinessGuard::publish(&path, &expected).unwrap();
    let actual: ServiceReady = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();

    assert_eq!(actual, expected);
    assert_eq!(actual.version.to_string(), "3.1.0");
    assert_eq!(actual.schema_version, 2);
    assert_eq!(actual.pid, 4242);
    assert_eq!(actual.started_at, 1_721_000_000);
    drop(guard);
}
