use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::exfat::dir_entry::build_file_entry_set;
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

fn dir(path: PathBuf, name: &str, children: Vec<ControlledEntry>) -> ControlledEntry {
    ControlledEntry {
        real_path: path,
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

fn root_entry_cluster(fs: &VirtualExfatFs, name: &str) -> u32 {
    let root = fs.read_at(fs.root_dir_offset_for_test(), 4096).unwrap();
    let entries = parse_entry_sets(&root).unwrap();
    entries
        .into_iter()
        .find(|entry| entry.name == name)
        .unwrap()
        .first_cluster
}

fn mark_entry_set_deleted(entry_set: &mut [u8]) {
    let secondary_count = entry_set[1] as usize;
    for idx in 0..=secondary_count {
        let offset = idx * 32;
        entry_set[offset] &= 0x7f;
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
fn write_interpreter_commits_empty_root_file_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let dir_entry = build_file_entry_set("empty.txt", false, 0, 0, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();
    fs.flush().unwrap();

    let real = tmp.path().join("empty.txt");
    assert!(real.exists());
    assert_eq!(std::fs::metadata(real).unwrap().len(), 0);
}

#[test]
fn write_interpreter_commits_empty_root_directory_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let dir_entry = build_file_entry_set("empty_dir", true, 100, 0, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();
    fs.flush().unwrap();

    assert!(tmp.path().join("empty_dir").is_dir());
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

#[test]
fn write_interpreter_commits_nested_file_in_existing_directory_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("dir")).unwrap();
    let tree = vec![dir(tmp.path().join("dir"), "dir", vec![])];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();
    let dir_cluster = root_entry_cluster(&fs, "dir");
    let file_cluster = 220;
    let dir_entry = build_file_entry_set("nested.txt", false, file_cluster, 6, false);
    let mut dir_sector = vec![0u8; 512];
    dir_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.cluster_offset_for_test(dir_cluster), &dir_sector)
        .unwrap();
    let mut data_sector = vec![0u8; 512];
    data_sector[..6].copy_from_slice(b"nested");
    fs.write_at(fs.cluster_offset_for_test(file_cluster), &data_sector)
        .unwrap();
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("dir/nested.txt")).unwrap(),
        b"nested"
    );
}

#[test]
fn write_interpreter_commits_nested_empty_directory_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("dir")).unwrap();
    let tree = vec![dir(tmp.path().join("dir"), "dir", vec![])];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();
    let dir_cluster = root_entry_cluster(&fs, "dir");
    let dir_entry = build_file_entry_set("child", true, 230, 0, false);
    let mut dir_sector = vec![0u8; 512];
    dir_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.cluster_offset_for_test(dir_cluster), &dir_sector)
        .unwrap();
    fs.flush().unwrap();

    assert!(tmp.path().join("dir/child").is_dir());
}

#[test]
fn write_interpreter_commits_file_inside_runtime_created_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let dir_cluster = 240;
    let dir_entry = build_file_entry_set("runtime_dir", true, dir_cluster, 0, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();
    fs.flush().unwrap();

    let file_cluster = 250;
    let file_entry = build_file_entry_set("child.txt", false, file_cluster, 5, false);
    let mut dir_sector = vec![0u8; 512];
    dir_sector[..file_entry.len()].copy_from_slice(&file_entry);
    fs.write_at(fs.cluster_offset_for_test(dir_cluster), &dir_sector)
        .unwrap();
    let mut data_sector = vec![0u8; 512];
    data_sector[..5].copy_from_slice(b"child");
    fs.write_at(fs.cluster_offset_for_test(file_cluster), &data_sector)
        .unwrap();
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("runtime_dir/child.txt")).unwrap(),
        b"child"
    );
}

#[test]
fn write_interpreter_commits_second_write_to_runtime_created_file() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let file_cluster = 260;
    let dir_entry = build_file_entry_set("runtime.txt", false, file_cluster, 5, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);
    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();

    let mut data_sector = vec![0u8; 512];
    data_sector[..5].copy_from_slice(b"first");
    fs.write_at(fs.cluster_offset_for_test(file_cluster), &data_sector)
        .unwrap();
    fs.flush().unwrap();

    let mut next_sector = vec![0u8; 512];
    next_sector[..5].copy_from_slice(b"again");
    fs.write_at(fs.cluster_offset_for_test(file_cluster), &next_sector)
        .unwrap();
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("runtime.txt")).unwrap(),
        b"again"
    );
}

#[test]
fn write_interpreter_commits_deep_directory_tree_created_before_single_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();

    let nested_cluster = 240;
    let a_cluster = 241;
    let b_cluster = 242;
    let file_cluster = 250;

    let nested_entry = build_file_entry_set("nested", true, nested_cluster, 0, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..nested_entry.len()].copy_from_slice(&nested_entry);
    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();

    let a_entry = build_file_entry_set("a", true, a_cluster, 0, false);
    let mut nested_sector = vec![0u8; 512];
    nested_sector[..a_entry.len()].copy_from_slice(&a_entry);
    fs.write_at(fs.cluster_offset_for_test(nested_cluster), &nested_sector)
        .unwrap();

    let b_entry = build_file_entry_set("b", true, b_cluster, 0, false);
    let mut a_sector = vec![0u8; 512];
    a_sector[..b_entry.len()].copy_from_slice(&b_entry);
    fs.write_at(fs.cluster_offset_for_test(a_cluster), &a_sector)
        .unwrap();

    let file_entry = build_file_entry_set("data.txt", false, file_cluster, 4, false);
    let mut b_sector = vec![0u8; 512];
    b_sector[..file_entry.len()].copy_from_slice(&file_entry);
    fs.write_at(fs.cluster_offset_for_test(b_cluster), &b_sector)
        .unwrap();

    let mut data_sector = vec![0u8; 512];
    data_sector[..4].copy_from_slice(b"deep");
    fs.write_at(fs.cluster_offset_for_test(file_cluster), &data_sector)
        .unwrap();

    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("nested/a/b/data.txt")).unwrap(),
        b"deep"
    );
}

#[test]
fn write_interpreter_commits_rename_and_delete_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("old.txt"), b"hello").unwrap();
    std::fs::write(tmp.path().join("gone.txt"), b"bye").unwrap();
    let tree = vec![
        file(tmp.path().join("old.txt"), "old.txt", 5),
        file(tmp.path().join("gone.txt"), "gone.txt", 3),
    ];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();
    let old_cluster = root_entry_cluster(&fs, "old.txt");
    let dir_entry = build_file_entry_set("new.txt", false, old_cluster, 5, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();
    fs.flush().unwrap();

    assert!(!tmp.path().join("old.txt").exists());
    assert!(tmp.path().join("new.txt").exists());
    assert!(!tmp.path().join("gone.txt").exists());
}

#[test]
fn write_interpreter_ignores_windows_deleted_entry_sets_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("old.txt"), b"hello").unwrap();
    std::fs::write(tmp.path().join("gone.txt"), b"bye").unwrap();
    let tree = vec![
        file(tmp.path().join("old.txt"), "old.txt", 5),
        file(tmp.path().join("gone.txt"), "gone.txt", 3),
    ];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();
    let old_cluster = root_entry_cluster(&fs, "old.txt");

    let mut deleted_old = build_file_entry_set("old.txt", false, old_cluster, 5, false);
    mark_entry_set_deleted(&mut deleted_old);
    let mut deleted_gone = build_file_entry_set("gone.txt", false, root_entry_cluster(&fs, "gone.txt"), 3, false);
    mark_entry_set_deleted(&mut deleted_gone);
    let new_entry = build_file_entry_set("new.txt", false, old_cluster, 5, false);

    let mut root_sector = vec![0u8; 512];
    let mut cursor = 0usize;
    for entry in [&deleted_old, &deleted_gone, &new_entry] {
        root_sector[cursor..cursor + entry.len()].copy_from_slice(entry);
        cursor += entry.len();
    }

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();
    fs.flush().unwrap();

    assert!(!tmp.path().join("old.txt").exists());
    assert!(tmp.path().join("new.txt").exists());
    assert!(!tmp.path().join("gone.txt").exists());
}

#[test]
fn write_interpreter_commits_truncate_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("file.txt"), b"hello").unwrap();
    let tree = vec![file(tmp.path().join("file.txt"), "file.txt", 5)];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();
    let cluster = root_entry_cluster(&fs, "file.txt");
    let dir_entry = build_file_entry_set("file.txt", false, cluster, 2, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector).unwrap();
    fs.flush().unwrap();

    assert_eq!(std::fs::read(tmp.path().join("file.txt")).unwrap(), b"he");
}
