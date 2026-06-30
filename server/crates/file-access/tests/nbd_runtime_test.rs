use std::fs;
use std::path::Path;
use std::time::Duration;

use file_access::nbd::NbdServer;
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

    let server = NbdServer::new(Path::new("/dev/nbd3"));

    server
        .wait_ready_under(dir.path(), 32768, Duration::from_millis(50))
        .unwrap();
}

#[test]
fn wait_ready_under_times_out_when_size_does_not_match() {
    let dir = tempdir().unwrap();
    make_nbd_sysfs(dir.path(), "nbd3", "1234\n", "0\n");

    let server = NbdServer::new(Path::new("/dev/nbd3"));
    let err = server
        .wait_ready_under(dir.path(), 32768, Duration::from_millis(20))
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
}
