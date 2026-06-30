use std::path::PathBuf;

use file_access::types::{ControlledEntry, ExecFileType};
use file_access::vfs::{NodeId, VfsIndex};

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
            "/mnt/usb_raw/a",
            "a",
            vec![file("/mnt/usb_raw/a/readme.txt", "readme.txt", 3)],
        ),
        dir(
            "/mnt/usb_raw/b",
            "b",
            vec![file("/mnt/usb_raw/b/readme.txt", "readme.txt", 4)],
        ),
    ];

    let index = VfsIndex::from_controlled_tree(&PathBuf::from("/mnt/usb_raw"), &tree).unwrap();
    let a = index.lookup_path("/a/readme.txt").unwrap();
    let b = index.lookup_path("/b/readme.txt").unwrap();

    assert_ne!(a, b);
    assert_eq!(index.node(a).unwrap().size, 3);
    assert_eq!(index.node(b).unwrap().size, 4);
}

#[test]
fn vfs_index_keeps_virus_file_visible_with_zero_virtual_size() {
    let mut infected = file("/mnt/usb_raw/bad.exe", "[病毒禁止访问]bad.exe", 4096);
    infected.is_virus = true;
    let index = VfsIndex::from_controlled_tree(&PathBuf::from("/mnt/usb_raw"), &[infected])
        .unwrap();
    let node = index
        .node(index.lookup_path("/[病毒禁止访问]bad.exe").unwrap())
        .unwrap();

    assert!(node.is_virus);
    assert_eq!(node.size, 0);
    assert_eq!(node.real_path, PathBuf::from("/mnt/usb_raw/bad.exe"));
}

#[test]
fn root_node_is_stable() {
    let index = VfsIndex::from_controlled_tree(&PathBuf::from("/mnt/usb_raw"), &[]).unwrap();
    assert_eq!(index.root_id(), NodeId(1));
    assert_eq!(index.node(index.root_id()).unwrap().virtual_path, "/");
}
