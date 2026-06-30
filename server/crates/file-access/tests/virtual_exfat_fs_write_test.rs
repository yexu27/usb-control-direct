use std::collections::HashSet;
use std::fs;

use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::PolicySnapshot;

fn snapshot(permission: i32) -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::new(),
        permission,
    }
}

#[test]
fn operation_api_creates_writes_flushes_renames_truncates_and_deletes() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(1), 16 * 1024 * 1024).unwrap();

    fs.create_file("/a.txt").unwrap();
    fs.write_file("/a.txt", 0, b"hello").unwrap();
    fs.flush().unwrap();
    assert_eq!(fs::read(tmp.path().join("a.txt")).unwrap(), b"hello");

    fs.rename("/a.txt", "/b.txt").unwrap();
    fs.truncate("/b.txt", 2).unwrap();
    fs.flush().unwrap();
    assert_eq!(fs::read(tmp.path().join("b.txt")).unwrap(), b"he");

    fs.delete_file("/b.txt").unwrap();
    fs.flush().unwrap();
    assert!(!tmp.path().join("b.txt").exists());
}

#[test]
fn readonly_operation_api_denies_writes() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(0), 16 * 1024 * 1024).unwrap();
    assert!(fs.create_file("/a.txt").is_err());
    assert!(!tmp.path().join("a.txt").exists());
}
