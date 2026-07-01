use std::collections::HashSet;

use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::PolicySnapshot;

fn readonly_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::new(),
        permission: 0,
    }
}

#[test]
fn readonly_denies_overlay_writes_before_real_fs_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], readonly_snapshot(), 16 * 1024 * 1024).unwrap();

    let err = fs
        .write_at(fs.root_dir_offset_for_test(), &[0u8; 512])
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 0);
}
