use std::collections::HashSet;

use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::PolicySnapshot;

fn snapshot(permission: i32) -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission,
    }
}

fn root_entry_names(fs: &VirtualExfatFs) -> Vec<String> {
    parse_entry_sets(&fs.read_at(fs.root_dir_offset_for_test(), 4096).unwrap())
        .unwrap()
        .into_iter()
        .map(|entry| entry.name)
        .collect()
}

#[test]
fn readonly_policy_blocks_facade_write_without_real_file_or_virtual_entry() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(0), 16 * 1024 * 1024).unwrap();
    let mut root = vec![0u8; 4096];
    let entry = build_file_entry_set("blocked.txt", false, 0, 0, false);
    root[..entry.len()].copy_from_slice(&entry);

    let err = fs
        .write_at(fs.root_dir_offset_for_test(), &root)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!tmp.path().join("blocked.txt").exists());
    assert!(!root_entry_names(&fs).contains(&"blocked.txt".to_string()));
}
