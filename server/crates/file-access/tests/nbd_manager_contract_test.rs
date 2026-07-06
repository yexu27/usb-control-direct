use std::fs;
use std::path::{Path, PathBuf};

use file_access::nbd::manager::NbdDeviceManager;
use tempfile::tempdir;

fn make_nbd_sysfs(root: &Path, name: &str, pid: &str, size: &str) {
    let dir = root.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("pid"), pid).unwrap();
    fs::write(dir.join("size"), size).unwrap();
}

#[test]
fn recover_candidates_include_only_connected_devices() {
    let dir = tempdir().unwrap();
    make_nbd_sysfs(dir.path(), "nbd0", "\n", "0\n");
    make_nbd_sysfs(dir.path(), "nbd1", "1234\n", "32768\n");
    make_nbd_sysfs(dir.path(), "nbd2", "0\n", "0\n");
    make_nbd_sysfs(dir.path(), "nbd3", "2345\n", "32768\n");

    let manager = NbdDeviceManager::new(dir.path(), "/dev", "/sys/module/nbd/parameters/max_part");
    let candidates = manager.connected_devices_for_recovery(4).unwrap();

    assert_eq!(
        candidates,
        vec![PathBuf::from("/dev/nbd1"), PathBuf::from("/dev/nbd3")]
    );
}
