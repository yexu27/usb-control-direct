use std::collections::HashSet;

use file_access::exfat::fs::VirtualExfatFs;
use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::types::PolicySnapshot;

fn snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

#[test]
fn write_interpreter_rejects_boot_sector_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let err = fs.write_at(0, &[0x55; 512]).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn write_interpreter_commits_root_file_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let file_cluster = 200;
    let dir_entry = build_file_entry_set("created.txt", false, file_cluster, 11, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();

    let mut data_sector = vec![0u8; 512];
    data_sector[..11].copy_from_slice(b"hello world");
    fs.write_at(fs.cluster_offset_for_test(file_cluster), &data_sector)
        .unwrap();
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("created.txt")).unwrap(),
        b"hello world"
    );
}
