use std::collections::HashSet;
use std::fs;

use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::{ControlledEntry, ExecFileType, PolicySnapshot};

fn snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::from([".blocked".to_string()]),
        permission: 1,
    }
}

fn entry(root: &std::path::Path, name: &str, size: u64, extension: &str) -> ControlledEntry {
    ControlledEntry {
        real_path: root.join(name),
        virtual_name: name.to_string(),
        file_size: size,
        is_dir: false,
        is_virus: false,
        exec_type: None::<ExecFileType>,
        extension: extension.to_string(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![],
    }
}

#[test]
fn virtual_fs_reads_file_data_from_real_mount() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("ok.txt"), b"hello").unwrap();
    let tree = vec![entry(tmp.path(), "ok.txt", 5, "txt")];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let node = fs.lookup_path("/ok.txt").unwrap();
    let data = fs.read_file(node, 0, 5).unwrap();
    assert_eq!(data, b"hello");
}

#[test]
fn virtual_fs_denies_blacklisted_read_without_hiding_node() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("bad.blocked"), b"secret").unwrap();
    let tree = vec![entry(tmp.path(), "bad.blocked", 6, "blocked")];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let node = fs.lookup_path("/bad.blocked").unwrap();
    let err = fs.read_file(node, 0, 6).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}
