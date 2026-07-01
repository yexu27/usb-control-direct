use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::{ControlledEntry, PolicySnapshot};

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
fn initial_mapping_preserves_deep_directory_tree() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("a/b")).unwrap();
    fs::write(tmp.path().join("a/b/c.txt"), b"deep").unwrap();

    let tree = vec![dir(
        tmp.path().join("a").to_str().unwrap(),
        "a",
        vec![dir(
            tmp.path().join("a/b").to_str().unwrap(),
            "b",
            vec![file(
                tmp.path().join("a/b/c.txt").to_str().unwrap(),
                "c.txt",
                4,
            )],
        )],
    )];

    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    assert!(fs.lookup_path("/a").is_some());
    assert!(fs.lookup_path("/a/b").is_some());
    assert!(fs.lookup_path("/a/b/c.txt").is_some());
    let node = fs.lookup_path("/a/b/c.txt").unwrap();
    assert_eq!(fs.read_file(node, 0, 4).unwrap(), b"deep");

    let root = fs
        .read_at(fs.root_dir_offset_for_test(), 4096)
        .expect("read root directory");
    let root_entries = parse_entry_sets(&root).expect("parse root directory");
    let a = root_entries
        .iter()
        .find(|entry| entry.name == "a")
        .expect("root contains a");

    let a_dir = fs
        .read_at(fs.cluster_offset_for_test(a.first_cluster), 4096)
        .expect("read /a directory");
    let a_entries = parse_entry_sets(&a_dir).expect("parse /a directory");
    let b = a_entries
        .iter()
        .find(|entry| entry.name == "b")
        .expect("/a contains b");

    let b_dir = fs
        .read_at(fs.cluster_offset_for_test(b.first_cluster), 4096)
        .expect("read /a/b directory");
    let b_entries = parse_entry_sets(&b_dir).expect("parse /a/b directory");
    assert!(b_entries.iter().any(|entry| entry.name == "c.txt"));
}
