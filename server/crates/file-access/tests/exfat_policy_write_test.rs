use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::runtime_state::ExfatRuntimeState;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::{ControlledEntry, PolicySnapshot};
use file_access::vfs::mutation::{FsMutation, NodeKind};

fn readonly_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::new(),
        permission: 0,
    }
}

fn rw_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn file(path: PathBuf, name: &str, size: u64) -> ControlledEntry {
    ControlledEntry {
        real_path: path,
        virtual_name: name.to_string(),
        file_size: size,
        is_dir: false,
        is_virus: false,
        exec_type: None,
        extension: name
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .unwrap_or_default(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![],
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

#[test]
fn runtime_readonly_rejects_create_without_real_fs_change() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], readonly_snapshot(), 16 * 1024 * 1024)
            .unwrap();

    let err = state
        .commit_mutation(FsMutation::CreateFile {
            parent: "/".to_string(),
            name: "blocked.txt".to_string(),
            size: 0,
            valid_data_len: 0,
            chain: None,
            data_patches: Vec::new(),
        })
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!tmp.path().join("blocked.txt").exists());
    assert!(state.lookup_path("/blocked.txt").is_none());
}

#[test]
fn runtime_blacklist_rejects_new_file_extension_without_real_fs_change() {
    let tmp = tempfile::tempdir().unwrap();
    let mut snapshot = rw_snapshot();
    snapshot.file_type_blacklist_enabled = true;
    snapshot.blacklist_extensions.insert(".exe".to_string());
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], snapshot, 16 * 1024 * 1024)
            .unwrap();

    let err = state
        .commit_mutation(FsMutation::CreateFile {
            parent: "/".to_string(),
            name: "bad.exe".to_string(),
            size: 0,
            valid_data_len: 0,
            chain: None,
            data_patches: Vec::new(),
        })
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!tmp.path().join("bad.exe").exists());
    assert!(state.lookup_path("/bad.exe").is_none());
}

#[test]
fn runtime_virus_file_is_visible_zero_size_and_cannot_be_deleted() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("bad.exe"), b"virus").unwrap();
    let mut infected = file(tmp.path().join("bad.exe"), "[病毒禁止访问]bad.exe", 5);
    infected.is_virus = true;
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[infected], rw_snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let node = state.lookup_path("/[病毒禁止访问]bad.exe").unwrap();
    assert!(node.is_virus);
    assert_eq!(node.size, 0);

    let err = state
        .commit_mutation(FsMutation::Delete {
            virtual_path: "/[病毒禁止访问]bad.exe".to_string(),
            kind: NodeKind::File,
        })
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(std::fs::read(tmp.path().join("bad.exe")).unwrap(), b"virus");
    assert!(state.lookup_path("/[病毒禁止访问]bad.exe").is_some());
}
