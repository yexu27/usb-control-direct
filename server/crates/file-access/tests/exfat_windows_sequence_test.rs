use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::dir_entry::build_file_entry_set;
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
    let data = fs.read_at(fs.root_dir_offset_for_test(), 4096).unwrap();
    parse_entry_sets(&data)
        .unwrap()
        .into_iter()
        .find(|entry| entry.name == name)
        .unwrap()
        .first_cluster
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
    let mut sector = vec![0u8; 4096];
    let mut cursor = 0usize;
    for entry in entries {
        sector[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    fs.write_at(fs.cluster_offset_for_test(dir_cluster), &sector)
        .unwrap();
}

fn write_file_data(fs: &VirtualExfatFs, cluster: u32, data: &[u8]) {
    let mut sector = vec![0u8; 512];
    sector[..data.len()].copy_from_slice(data);
    fs.write_at(fs.cluster_offset_for_test(cluster), &sector)
        .unwrap();
}

#[test]
fn windows_sequence_commits_file_created_inside_existing_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("t2")).unwrap();
    let tree = vec![dir(tmp.path().join("t2"), "t2", vec![])];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let t2_cluster = root_entry_cluster(&fs, "t2");
    let file_cluster = 720;

    write_file_data(&fs, file_cluster, b"inside-old-dir");
    write_dir_entries(
        &fs,
        t2_cluster,
        vec![build_file_entry_set(
            "created.txt",
            false,
            file_cluster,
            14,
            false,
        )],
    );
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("t2/created.txt")).unwrap(),
        b"inside-old-dir"
    );
}

#[test]
fn windows_sequence_commits_file_created_inside_existing_non_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
    std::fs::write(tmp.path().join("docs/old.txt"), b"old").unwrap();
    let tree = vec![dir(
        tmp.path().join("docs"),
        "docs",
        vec![file(tmp.path().join("docs/old.txt"), "old.txt", 3)],
    )];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let docs_cluster = root_entry_cluster(&fs, "docs");
    let old_cluster = entry_cluster(&fs, docs_cluster, "old.txt");
    let new_cluster = 721;

    write_file_data(&fs, new_cluster, b"new-data");
    write_dir_entries(
        &fs,
        docs_cluster,
        vec![
            build_file_entry_set("old.txt", false, old_cluster, 3, false),
            build_file_entry_set("new.txt", false, new_cluster, 8, false),
        ],
    );
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("docs/old.txt")).unwrap(),
        b"old"
    );
    assert_eq!(
        std::fs::read(tmp.path().join("docs/new.txt")).unwrap(),
        b"new-data"
    );
}

#[test]
fn windows_sequence_commits_rename() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("old.txt"), b"data").unwrap();
    let tree = vec![file(tmp.path().join("old.txt"), "old.txt", 4)];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let old_cluster = root_entry_cluster(&fs, "old.txt");
    write_dir_entries(
        &fs,
        2,
        vec![build_file_entry_set(
            "new.txt",
            false,
            old_cluster,
            4,
            false,
        )],
    );
    fs.flush().unwrap();

    assert!(!tmp.path().join("old.txt").exists());
    assert_eq!(std::fs::read(tmp.path().join("new.txt")).unwrap(), b"data");
}

#[test]
fn windows_sequence_commits_truncate() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("data.txt"), b"abcdef").unwrap();
    let tree = vec![file(tmp.path().join("data.txt"), "data.txt", 6)];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let cluster = root_entry_cluster(&fs, "data.txt");
    write_dir_entries(
        &fs,
        2,
        vec![build_file_entry_set("data.txt", false, cluster, 3, false)],
    );
    fs.flush().unwrap();

    assert_eq!(std::fs::read(tmp.path().join("data.txt")).unwrap(), b"abc");
}

#[test]
fn windows_sequence_commits_empty_deep_directory_and_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("1/2/3")).unwrap();
    std::fs::write(tmp.path().join("1/1.txt"), b"").unwrap();
    std::fs::write(tmp.path().join("1/2/2.txt"), b"").unwrap();
    std::fs::write(tmp.path().join("1/2/3/3.txt"), b"").unwrap();
    let tree = vec![dir(
        tmp.path().join("1"),
        "1",
        vec![
            file(tmp.path().join("1/1.txt"), "1.txt", 0),
            dir(
                tmp.path().join("1/2"),
                "2",
                vec![
                    file(tmp.path().join("1/2/2.txt"), "2.txt", 0),
                    dir(
                        tmp.path().join("1/2/3"),
                        "3",
                        vec![file(tmp.path().join("1/2/3/3.txt"), "3.txt", 0)],
                    ),
                ],
            ),
        ],
    )];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let one = root_entry_cluster(&fs, "1");
    let two = entry_cluster(&fs, one, "2");
    let three = entry_cluster(&fs, two, "3");
    let four = 700;

    write_dir_entries(
        &fs,
        three,
        vec![
            build_file_entry_set("3.txt", false, entry_cluster(&fs, three, "3.txt"), 0, false),
            build_file_entry_set("4", true, four, 0, false),
        ],
    );
    write_dir_entries(
        &fs,
        four,
        vec![build_file_entry_set("4.txt", false, 0, 0, false)],
    );
    fs.flush().unwrap();

    assert!(tmp.path().join("1/2/3/4").is_dir());
    assert!(tmp.path().join("1/2/3/4/4.txt").is_file());
    assert_eq!(
        std::fs::metadata(tmp.path().join("1/2/3/4/4.txt"))
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn windows_sequence_commits_file_data_after_empty_file_create() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let file_cluster = 710;

    write_dir_entries(
        &fs,
        2,
        vec![build_file_entry_set(
            "created.txt",
            false,
            file_cluster,
            0,
            false,
        )],
    );
    fs.flush().unwrap();
    assert!(tmp.path().join("created.txt").is_file());

    write_file_data(&fs, file_cluster, b"hello world");
    write_dir_entries(
        &fs,
        2,
        vec![build_file_entry_set(
            "created.txt",
            false,
            file_cluster,
            11,
            false,
        )],
    );
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("created.txt")).unwrap(),
        b"hello world"
    );
}

#[test]
fn windows_sequence_commits_data_after_zero_cluster_empty_file_create() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let file_cluster = 711;

    write_dir_entries(
        &fs,
        2,
        vec![build_file_entry_set("created.txt", false, 0, 0, false)],
    );
    fs.flush().unwrap();
    assert!(tmp.path().join("created.txt").is_file());
    assert_eq!(
        std::fs::metadata(tmp.path().join("created.txt"))
            .unwrap()
            .len(),
        0
    );

    write_file_data(&fs, file_cluster, b"hello world");
    write_dir_entries(
        &fs,
        2,
        vec![build_file_entry_set(
            "created.txt",
            false,
            file_cluster,
            11,
            false,
        )],
    );
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("created.txt")).unwrap(),
        b"hello world"
    );
}

#[test]
fn windows_sequence_commits_outer_directory_tree_delete() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("matrix/a/b")).unwrap();
    std::fs::write(tmp.path().join("matrix/a/b/data.txt"), b"data").unwrap();
    let tree = vec![dir(
        tmp.path().join("matrix"),
        "matrix",
        vec![dir(
            tmp.path().join("matrix/a"),
            "a",
            vec![dir(
                tmp.path().join("matrix/a/b"),
                "b",
                vec![file(tmp.path().join("matrix/a/b/data.txt"), "data.txt", 4)],
            )],
        )],
    )];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    write_dir_entries(&fs, 2, vec![]);
    fs.flush().unwrap();

    assert!(!tmp.path().join("matrix").exists());
}
