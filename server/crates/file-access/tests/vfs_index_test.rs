use std::collections::HashSet;
use std::path::PathBuf;

use file_access::types::{
    blocked_placeholder_bytes, ControlledEntry, ExecFileType, PolicySnapshot,
};
use file_access::vfs::{NodeId, VfsIndex};

fn snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn file(path: &str, name: &str, size: u64) -> ControlledEntry {
    ControlledEntry {
        real_path: PathBuf::from(path),
        virtual_name: name.to_string(),
        file_size: size,
        is_dir: false,
        is_virus: false,
        exec_type: None::<ExecFileType>,
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

fn dir(path: &str, name: &str, children: Vec<ControlledEntry>) -> ControlledEntry {
    ControlledEntry {
        real_path: PathBuf::from(path),
        virtual_name: name.to_string(),
        file_size: 0,
        is_dir: true,
        is_virus: false,
        exec_type: None,
        extension: String::new(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children,
    }
}

#[test]
fn vfs_index_preserves_nested_directories_and_same_names() {
    let tree = vec![
        dir(
            "/mnt/usb-control/raw/<session-id>/a",
            "a",
            vec![file("/mnt/usb-control/raw/<session-id>/a/readme.txt", "readme.txt", 3)],
        ),
        dir(
            "/mnt/usb-control/raw/<session-id>/b",
            "b",
            vec![file("/mnt/usb-control/raw/<session-id>/b/readme.txt", "readme.txt", 4)],
        ),
    ];

    let index = VfsIndex::from_controlled_tree(
        &PathBuf::from("/mnt/usb-control/raw/<session-id>"),
        &tree,
        &snapshot(),
    )
    .unwrap();
    let a = index.lookup_path("/a/readme.txt").unwrap();
    let b = index.lookup_path("/b/readme.txt").unwrap();

    assert_ne!(a, b);
    assert_eq!(index.node(a).unwrap().size, 3);
    assert_eq!(index.node(b).unwrap().size, 4);
}

#[test]
fn vfs_index_keeps_virus_file_visible_with_placeholder_size() {
    let mut infected = file("/mnt/usb-control/raw/<session-id>/bad.exe", "[病毒禁止访问]bad.exe", 4096);
    infected.is_virus = true;
    let index = VfsIndex::from_controlled_tree(
        &PathBuf::from("/mnt/usb-control/raw/<session-id>"),
        &[infected],
        &snapshot(),
    )
    .unwrap();
    let node = index
        .node(index.lookup_path("/[病毒禁止访问]bad.exe").unwrap())
        .unwrap();

    assert!(node.is_virus);
    assert_eq!(node.size, blocked_placeholder_bytes().len() as u64);
    assert!(node.is_blocked_placeholder());
    assert_eq!(node.real_path, PathBuf::from("/mnt/usb-control/raw/<session-id>/bad.exe"));
}

#[test]
fn root_node_is_stable() {
    let index = VfsIndex::from_controlled_tree(
        &PathBuf::from("/mnt/usb-control/raw/<session-id>"),
        &[],
        &snapshot(),
    )
    .unwrap();
    assert_eq!(index.root_id(), NodeId(1));
    assert_eq!(index.node(index.root_id()).unwrap().virtual_path, "/");
}
