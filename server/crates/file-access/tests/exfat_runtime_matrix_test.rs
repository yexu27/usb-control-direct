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

fn entry_cluster(fs: &VirtualExfatFs, dir_cluster: u32, name: &str) -> u32 {
    let data = fs
        .read_at(fs.cluster_offset_for_test(dir_cluster), 4096)
        .unwrap();
    parse_entry_sets(&data)
        .unwrap()
        .into_iter()
        .find(|entry| entry.name == name)
        .unwrap()
        .first_cluster
}

fn write_dir_entries(fs: &VirtualExfatFs, dir_cluster: u32, entries: Vec<Vec<u8>>) {
    let mut data = vec![0u8; 4096];
    let mut cursor = 0usize;
    for entry in entries {
        data[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    fs.write_at(fs.cluster_offset_for_test(dir_cluster), &data)
        .unwrap();
}

fn write_file_data(fs: &VirtualExfatFs, cluster: u32, data: &[u8]) {
    let mut sector = vec![0u8; 512];
    sector[..data.len()].copy_from_slice(data);
    fs.write_at(fs.cluster_offset_for_test(cluster), &sector)
        .unwrap();
}

#[test]
fn facade_matrix_commits_create_write_rename_truncate_and_delete_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(1), 16 * 1024 * 1024).unwrap();

    let dir_cluster = 700;
    write_dir_entries(
        &fs,
        2,
        vec![build_file_entry_set("matrix", true, dir_cluster, 0, false)],
    );
    assert!(tmp.path().join("matrix").is_dir());

    let file_cluster = 701;
    write_file_data(&fs, file_cluster, b"hello world");
    write_dir_entries(
        &fs,
        dir_cluster,
        vec![build_file_entry_set(
            "file.txt",
            false,
            file_cluster,
            11,
            false,
        )],
    );
    assert_eq!(
        std::fs::read(tmp.path().join("matrix/file.txt")).unwrap(),
        b"hello world"
    );

    write_dir_entries(
        &fs,
        dir_cluster,
        vec![build_file_entry_set(
            "renamed.txt",
            false,
            file_cluster,
            11,
            false,
        )],
    );
    assert!(!tmp.path().join("matrix/file.txt").exists());
    assert!(tmp.path().join("matrix/renamed.txt").is_file());

    write_dir_entries(
        &fs,
        dir_cluster,
        vec![build_file_entry_set(
            "renamed.txt",
            false,
            file_cluster,
            2,
            false,
        )],
    );
    assert_eq!(
        std::fs::read(tmp.path().join("matrix/renamed.txt")).unwrap(),
        b"he"
    );

    write_dir_entries(&fs, 2, Vec::new());
    assert!(!tmp.path().join("matrix").exists());
    assert!(fs.lookup_path("/matrix").is_none());
}

#[test]
fn facade_matrix_rejects_readonly_write_without_partial_state() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(0), 16 * 1024 * 1024).unwrap();
    let err = fs
        .write_at(
            fs.cluster_offset_for_test(2),
            &build_file_entry_set("blocked.txt", false, 0, 0, false),
        )
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!tmp.path().join("blocked.txt").exists());
    assert!(fs.lookup_path("/blocked.txt").is_none());
}

#[test]
fn facade_matrix_initial_mapping_preserves_nested_empty_objects() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("1/2/empty")).unwrap();
    std::fs::write(tmp.path().join("1/zero.txt"), []).unwrap();
    let tree = vec![file_access::types::ControlledEntry {
        real_path: tmp.path().join("1"),
        virtual_name: "1".to_string(),
        file_size: 0,
        is_dir: true,
        is_virus: false,
        exec_type: None,
        extension: String::new(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![
            file_access::types::ControlledEntry {
                real_path: tmp.path().join("1/zero.txt"),
                virtual_name: "zero.txt".to_string(),
                file_size: 0,
                is_dir: false,
                is_virus: false,
                exec_type: None,
                extension: "txt".to_string(),
                is_autorun_target: false,
                is_autorun_inf: false,
                is_root_shell_script: false,
                children: vec![],
            },
            file_access::types::ControlledEntry {
                real_path: tmp.path().join("1/2"),
                virtual_name: "2".to_string(),
                file_size: 0,
                is_dir: true,
                is_virus: false,
                exec_type: None,
                extension: String::new(),
                is_autorun_target: false,
                is_autorun_inf: false,
                is_root_shell_script: false,
                children: vec![file_access::types::ControlledEntry {
                    real_path: tmp.path().join("1/2/empty"),
                    virtual_name: "empty".to_string(),
                    file_size: 0,
                    is_dir: true,
                    is_virus: false,
                    exec_type: None,
                    extension: String::new(),
                    is_autorun_target: false,
                    is_autorun_inf: false,
                    is_root_shell_script: false,
                    children: vec![],
                }],
            },
        ],
    }];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(1), 16 * 1024 * 1024).unwrap();

    let one = entry_cluster(&fs, 2, "1");
    let two = entry_cluster(&fs, one, "2");
    let _empty = entry_cluster(&fs, two, "empty");
    assert!(fs.lookup_path("/1/zero.txt").is_some());
    assert!(fs.lookup_path("/1/2/empty").is_some());
}
