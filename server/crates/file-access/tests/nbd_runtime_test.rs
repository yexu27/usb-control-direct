use std::fs;
use std::path::Path;
use std::time::Duration;

use file_access::nbd::sysfs::{
    nbd_name_from_device_path, parse_nbd_max_part, NbdPartitionScanStatus, NbdSysfs,
};
use tempfile::tempdir;

fn make_nbd_sysfs(root: &Path, name: &str, pid: &str, size: &str) {
    let dir = root.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("pid"), pid).unwrap();
    fs::write(dir.join("size"), size).unwrap();
}

#[test]
fn wait_ready_under_accepts_matching_pid_and_size() {
    let dir = tempdir().unwrap();
    make_nbd_sysfs(dir.path(), "nbd3", "1234\n", "32768\n");

    let sysfs = NbdSysfs::new(dir.path());

    sysfs
        .wait_ready("nbd3", 32768, Duration::from_millis(50))
        .unwrap();
}

#[test]
fn wait_ready_under_times_out_when_size_does_not_match() {
    let dir = tempdir().unwrap();
    make_nbd_sysfs(dir.path(), "nbd3", "1234\n", "0\n");

    let sysfs = NbdSysfs::new(dir.path());
    let err = sysfs
        .wait_ready("nbd3", 32768, Duration::from_millis(20))
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
}

#[test]
fn wait_disconnected_under_accepts_empty_pid() {
    let dir = tempdir().unwrap();
    make_nbd_sysfs(dir.path(), "nbd3", "\n", "0\n");

    let sysfs = NbdSysfs::new(dir.path());

    sysfs
        .wait_disconnected("nbd3", Duration::from_millis(50))
        .unwrap();
}

#[test]
fn wait_disconnected_under_times_out_when_pid_remains() {
    let dir = tempdir().unwrap();
    make_nbd_sysfs(dir.path(), "nbd3", "2267\n", "32768\n");

    let sysfs = NbdSysfs::new(dir.path());
    let err = sysfs
        .wait_disconnected("nbd3", Duration::from_millis(20))
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
}

#[test]
fn nbd_name_from_device_path_accepts_whole_nbd() {
    assert_eq!(
        nbd_name_from_device_path(Path::new("/dev/nbd3")).unwrap(),
        "nbd3"
    );
}

#[test]
fn nbd_name_from_device_path_rejects_partition() {
    let err = nbd_name_from_device_path(Path::new("/dev/nbd3p1")).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn parse_zero_max_part_as_disabled() {
    assert_eq!(
        parse_nbd_max_part("0\n").unwrap(),
        NbdPartitionScanStatus::Disabled
    );
}

#[test]
fn parse_nonzero_max_part_as_enabled() {
    assert_eq!(
        parse_nbd_max_part("31\n").unwrap(),
        NbdPartitionScanStatus::Enabled(31)
    );
}
