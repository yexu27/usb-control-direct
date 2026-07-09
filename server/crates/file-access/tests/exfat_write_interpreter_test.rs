use std::collections::HashSet;
use std::path::PathBuf;

use file_access::block_backend::BlockWriteOutcome;
use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::{ControlledEntry, ExecFileType, PolicySnapshot};

fn snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn exec_control_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
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

fn controlled_file(
    path: PathBuf,
    name: &str,
    size: u64,
    exec_type: Option<ExecFileType>,
) -> ControlledEntry {
    ControlledEntry {
        real_path: path,
        virtual_name: name.to_string(),
        file_size: size,
        is_dir: false,
        is_virus: false,
        exec_type,
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

fn root_entry_size(fs: &VirtualExfatFs, name: &str) -> u64 {
    let root = fs.read_at(fs.root_dir_offset_for_test(), 4096).unwrap();
    parse_entry_sets(&root)
        .unwrap()
        .into_iter()
        .find(|entry| entry.name == name)
        .unwrap()
        .data_length
}

fn assert_policy_rejected_and_restored(outcome: BlockWriteOutcome) {
    match outcome {
        BlockWriteOutcome::PolicyRejectedAndRestored { reason } => {
            assert!(
                reason.contains("blocked") || reason.contains("BlockedPlaceholderRewrite"),
                "unexpected policy rejection reason: {reason}"
            );
        }
        other => panic!("expected PolicyRejectedAndRestored, got {other:?}"),
    }
}

fn directory_entry_cluster(fs: &VirtualExfatFs, dir_cluster: u32, name: &str) -> u32 {
    let data = fs
        .read_at(fs.cluster_offset_for_test(dir_cluster), 4096)
        .unwrap();
    let entries = parse_entry_sets(&data).unwrap();
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

fn write_deleted_root_entries(fs: &VirtualExfatFs, entries: Vec<Vec<u8>>) {
    let mut root_sector = vec![0u8; 512];
    let mut cursor = 0usize;
    for entry in entries {
        root_sector[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
}

fn write_root_entries(fs: &VirtualExfatFs, entries: Vec<Vec<u8>>) {
    let mut root_sector = vec![0u8; 4096];
    let mut cursor = 0usize;
    for entry in entries {
        root_sector[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
}

fn write_dir_entries(fs: &VirtualExfatFs, dir_cluster: u32, entries: Vec<Vec<u8>>) {
    let mut dir_sector = vec![0u8; 4096];
    let mut cursor = 0usize;
    for entry in entries {
        dir_sector[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    fs.write_at(fs.cluster_offset_for_test(dir_cluster), &dir_sector)
        .unwrap();
}

fn write_file_data(fs: &VirtualExfatFs, cluster: u32, data: &[u8]) {
    let mut data_sector = vec![0u8; 512];
    data_sector[..data.len()].copy_from_slice(data);
    fs.write_at(fs.cluster_offset_for_test(cluster), &data_sector)
        .unwrap();
}

#[test]
fn write_interpreter_rejects_boot_sector_mutation() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let err = fs.write_at(0, &[0x55; 512]).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn flush_absorbs_blocked_placeholder_policy_rejection_without_device_error() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("setup.exe"), b"blocked-real-content").unwrap();
    let tree = vec![controlled_file(
        tmp.path().join("setup.exe"),
        "setup.exe",
        20,
        Some(ExecFileType::Pe),
    )];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, exec_control_snapshot(), 16 * 1024 * 1024)
        .unwrap();

    let cluster = root_entry_cluster(&fs, "setup.exe");
    let mut root_sector = vec![0_u8; 4096];
    let rename_entry = build_file_entry_set(
        "1.exe",
        false,
        cluster,
        root_entry_size(&fs, "setup.exe"),
        false,
    );
    root_sector[..rename_entry.len()].copy_from_slice(&rename_entry);

    let outcome = fs
        .write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
    assert_policy_rejected_and_restored(outcome);
    fs.flush().unwrap();

    assert!(tmp.path().join("setup.exe").exists());
    assert!(!tmp.path().join("1.exe").exists());
    assert!(fs.lookup_path("/setup.exe").is_some());
    assert!(fs.lookup_path("/1.exe").is_none());
}

#[test]
fn write_interpreter_commits_empty_root_file_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let dir_entry = build_file_entry_set("empty.txt", false, 0, 0, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
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

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
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

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();

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

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
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
fn write_interpreter_commits_deep_empty_file_without_explicit_flush() {
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
    let one_cluster = root_entry_cluster(&fs, "1");
    let two_cluster = directory_entry_cluster(&fs, one_cluster, "2");
    let three_cluster = directory_entry_cluster(&fs, two_cluster, "3");
    let four_cluster = 520;

    write_dir_entries(
        &fs,
        three_cluster,
        vec![
            build_file_entry_set(
                "3.txt",
                false,
                directory_entry_cluster(&fs, three_cluster, "3.txt"),
                0,
                false,
            ),
            build_file_entry_set("4", true, four_cluster, 0, false),
        ],
    );
    write_dir_entries(
        &fs,
        four_cluster,
        vec![build_file_entry_set("4.txt", false, 0, 0, false)],
    );

    assert!(tmp.path().join("1/2/3/4").is_dir());
    assert!(tmp.path().join("1/2/3/4/4.txt").is_file());
}

#[test]
fn write_interpreter_commits_deep_directory_when_child_cluster_is_written_before_parent_entry() {
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
    let one_cluster = root_entry_cluster(&fs, "1");
    let two_cluster = directory_entry_cluster(&fs, one_cluster, "2");
    let three_cluster = directory_entry_cluster(&fs, two_cluster, "3");
    let four_cluster = 530;

    write_dir_entries(
        &fs,
        four_cluster,
        vec![build_file_entry_set("4.txt", false, 0, 0, false)],
    );
    write_dir_entries(
        &fs,
        three_cluster,
        vec![
            build_file_entry_set(
                "3.txt",
                false,
                directory_entry_cluster(&fs, three_cluster, "3.txt"),
                0,
                false,
            ),
            build_file_entry_set("4", true, four_cluster, 0, false),
        ],
    );

    assert!(tmp.path().join("1/2/3/4").is_dir());
    assert!(tmp.path().join("1/2/3/4/4.txt").is_file());
}

#[test]
fn write_interpreter_commits_second_write_to_runtime_created_file() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let file_cluster = 260;
    let dir_entry = build_file_entry_set("runtime.txt", false, file_cluster, 5, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..dir_entry.len()].copy_from_slice(&dir_entry);
    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();

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
fn write_interpreter_commits_data_written_after_zero_length_runtime_create() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();
    let file_cluster = 270;

    write_root_entries(
        &fs,
        vec![build_file_entry_set(
            "zero_then_data.txt",
            false,
            file_cluster,
            0,
            false,
        )],
    );
    fs.flush().unwrap();

    write_file_data(&fs, file_cluster, b"data");
    write_root_entries(
        &fs,
        vec![build_file_entry_set(
            "zero_then_data.txt",
            false,
            file_cluster,
            4,
            false,
        )],
    );
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("zero_then_data.txt")).unwrap(),
        b"data"
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
    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();

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

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
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
    let mut deleted_gone = build_file_entry_set(
        "gone.txt",
        false,
        root_entry_cluster(&fs, "gone.txt"),
        3,
        false,
    );
    mark_entry_set_deleted(&mut deleted_gone);
    let new_entry = build_file_entry_set("new.txt", false, old_cluster, 5, false);

    let mut root_sector = vec![0u8; 512];
    let mut cursor = 0usize;
    for entry in [&deleted_old, &deleted_gone, &new_entry] {
        root_sector[cursor..cursor + entry.len()].copy_from_slice(entry);
        cursor += entry.len();
    }

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
    fs.flush().unwrap();

    assert!(!tmp.path().join("old.txt").exists());
    assert!(tmp.path().join("new.txt").exists());
    assert!(!tmp.path().join("gone.txt").exists());
}

#[test]
fn write_interpreter_commits_batch_file_delete_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("a.txt"), b"a").unwrap();
    std::fs::write(tmp.path().join("b.txt"), b"b").unwrap();
    std::fs::write(tmp.path().join("keep.txt"), b"keep").unwrap();
    let tree = vec![
        file(tmp.path().join("a.txt"), "a.txt", 1),
        file(tmp.path().join("b.txt"), "b.txt", 1),
        file(tmp.path().join("keep.txt"), "keep.txt", 4),
    ];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();
    let keep_cluster = root_entry_cluster(&fs, "keep.txt");
    let keep_entry = build_file_entry_set("keep.txt", false, keep_cluster, 4, false);

    write_deleted_root_entries(&fs, vec![keep_entry]);
    fs.flush().unwrap();

    assert!(!tmp.path().join("a.txt").exists());
    assert!(!tmp.path().join("b.txt").exists());
    assert_eq!(std::fs::read(tmp.path().join("keep.txt")).unwrap(), b"keep");
}

#[test]
fn write_interpreter_commits_non_empty_directory_tree_delete_on_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("tree/a/b")).unwrap();
    std::fs::write(tmp.path().join("tree/root.txt"), b"root").unwrap();
    std::fs::write(tmp.path().join("tree/a/child.txt"), b"child").unwrap();
    std::fs::write(tmp.path().join("tree/a/b/deep.txt"), b"deep").unwrap();
    std::fs::write(tmp.path().join("keep.txt"), b"keep").unwrap();
    let tree_entry = dir(
        tmp.path().join("tree"),
        "tree",
        vec![
            file(tmp.path().join("tree/root.txt"), "root.txt", 4),
            dir(
                tmp.path().join("tree/a"),
                "a",
                vec![dir(
                    tmp.path().join("tree/a/b"),
                    "b",
                    vec![file(tmp.path().join("tree/a/b/deep.txt"), "deep.txt", 4)],
                )],
            ),
        ],
    );
    let tree = vec![tree_entry, file(tmp.path().join("keep.txt"), "keep.txt", 4)];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();
    let keep_cluster = root_entry_cluster(&fs, "keep.txt");
    let keep_entry = build_file_entry_set("keep.txt", false, keep_cluster, 4, false);

    write_deleted_root_entries(&fs, vec![keep_entry]);
    fs.flush().unwrap();

    assert!(!tmp.path().join("tree").exists());
    assert_eq!(std::fs::read(tmp.path().join("keep.txt")).unwrap(), b"keep");
}

#[test]
fn write_interpreter_commits_mixed_delete_then_create_after_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("old_dir/nested")).unwrap();
    std::fs::write(tmp.path().join("old_dir/nested/old.txt"), b"old").unwrap();
    std::fs::write(tmp.path().join("old_file.txt"), b"old-file").unwrap();
    let tree = vec![
        dir(
            tmp.path().join("old_dir"),
            "old_dir",
            vec![dir(
                tmp.path().join("old_dir/nested"),
                "nested",
                vec![file(
                    tmp.path().join("old_dir/nested/old.txt"),
                    "old.txt",
                    3,
                )],
            )],
        ),
        file(tmp.path().join("old_file.txt"), "old_file.txt", 8),
    ];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    write_deleted_root_entries(&fs, vec![]);
    fs.flush().unwrap();

    assert!(!tmp.path().join("old_dir").exists());
    assert!(!tmp.path().join("old_file.txt").exists());

    let new_cluster = 300;
    let new_entry = build_file_entry_set("after.txt", false, new_cluster, 5, false);
    let mut root_sector = vec![0u8; 512];
    root_sector[..new_entry.len()].copy_from_slice(&new_entry);
    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
    let mut data_sector = vec![0u8; 512];
    data_sector[..5].copy_from_slice(b"after");
    fs.write_at(fs.cluster_offset_for_test(new_cluster), &data_sector)
        .unwrap();
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("after.txt")).unwrap(),
        b"after"
    );
}

#[test]
fn write_interpreter_deletes_runtime_created_directory_tree_after_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();

    let tree_cluster = 320;
    let child_cluster = 321;
    let file_cluster = 330;

    write_root_entries(
        &fs,
        vec![build_file_entry_set(
            "runtime_tree",
            true,
            tree_cluster,
            0,
            false,
        )],
    );
    fs.flush().unwrap();

    write_dir_entries(
        &fs,
        tree_cluster,
        vec![build_file_entry_set("child", true, child_cluster, 0, false)],
    );
    fs.flush().unwrap();

    write_dir_entries(
        &fs,
        child_cluster,
        vec![build_file_entry_set(
            "data.txt",
            false,
            file_cluster,
            4,
            false,
        )],
    );
    write_file_data(&fs, file_cluster, b"data");
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("runtime_tree/child/data.txt")).unwrap(),
        b"data"
    );

    write_root_entries(&fs, vec![]);
    fs.flush().unwrap();

    assert!(!tmp.path().join("runtime_tree").exists());
}

#[test]
fn write_interpreter_commits_create_after_runtime_delete() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();

    let old_cluster = 340;
    let new_cluster = 350;

    write_root_entries(
        &fs,
        vec![build_file_entry_set(
            "old.txt",
            false,
            old_cluster,
            3,
            false,
        )],
    );
    write_file_data(&fs, old_cluster, b"old");
    fs.flush().unwrap();

    assert_eq!(std::fs::read(tmp.path().join("old.txt")).unwrap(), b"old");

    write_root_entries(
        &fs,
        vec![build_file_entry_set(
            "new.txt",
            false,
            new_cluster,
            3,
            false,
        )],
    );
    write_file_data(&fs, new_cluster, b"new");
    fs.flush().unwrap();

    assert!(!tmp.path().join("old.txt").exists());
    assert_eq!(std::fs::read(tmp.path().join("new.txt")).unwrap(), b"new");
}

#[test]
fn write_interpreter_commits_runtime_rename_and_truncate_after_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();

    let cluster = 360;

    write_root_entries(
        &fs,
        vec![build_file_entry_set("source.txt", false, cluster, 8, false)],
    );
    write_file_data(&fs, cluster, b"abcdefgh");
    fs.flush().unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("source.txt")).unwrap(),
        b"abcdefgh"
    );

    write_root_entries(
        &fs,
        vec![build_file_entry_set("target.txt", false, cluster, 4, false)],
    );
    fs.flush().unwrap();

    assert!(!tmp.path().join("source.txt").exists());
    assert_eq!(
        std::fs::read(tmp.path().join("target.txt")).unwrap(),
        b"abcd"
    );
}

#[test]
fn write_interpreter_commits_delete_batch_and_nested_creates_in_same_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();

    let root_cluster = 370;
    let empty_cluster = 371;
    let tree_cluster = 372;
    let mixed_cluster = 373;
    let mutate_cluster = 374;
    let single_cluster = 375;
    let batch_1_cluster = 376;
    let mixed_file_cluster = 377;
    let rename_target_cluster = 378;
    let truncate_cluster = 379;

    write_root_entries(
        &fs,
        vec![build_file_entry_set("matrix", true, root_cluster, 0, false)],
    );
    fs.flush().unwrap();

    write_dir_entries(
        &fs,
        root_cluster,
        vec![
            build_file_entry_set("empty_dir", true, empty_cluster, 0, false),
            build_file_entry_set("tree", true, tree_cluster, 0, false),
            build_file_entry_set("mixed_dir", true, mixed_cluster, 0, false),
            build_file_entry_set("mutate", true, mutate_cluster, 0, false),
            build_file_entry_set("single_delete.txt", false, single_cluster, 6, false),
            build_file_entry_set("batch_1.txt", false, batch_1_cluster, 7, false),
            build_file_entry_set("mixed_file.txt", false, mixed_file_cluster, 10, false),
        ],
    );
    write_dir_entries(
        &fs,
        mutate_cluster,
        vec![
            build_file_entry_set("rename_target.txt", false, rename_target_cluster, 13, false),
            build_file_entry_set("truncate_me.txt", false, truncate_cluster, 10, false),
        ],
    );
    write_file_data(&fs, single_cluster, b"single");
    write_file_data(&fs, batch_1_cluster, b"batch-1");
    write_file_data(&fs, mixed_file_cluster, b"mixed-file");
    write_file_data(&fs, rename_target_cluster, b"rename-before");
    write_file_data(&fs, truncate_cluster, b"1234567890");
    fs.flush().unwrap();

    let after_delete_cluster = 390;
    let deep_cluster = 391;
    let created_cluster = 392;
    let post_rename_cluster = 393;
    let post_truncate_cluster = 394;

    write_dir_entries(
        &fs,
        root_cluster,
        vec![
            build_file_entry_set("mutate", true, mutate_cluster, 0, false),
            build_file_entry_set("after_delete", true, after_delete_cluster, 0, false),
        ],
    );
    write_dir_entries(
        &fs,
        after_delete_cluster,
        vec![build_file_entry_set("deep", true, deep_cluster, 0, false)],
    );
    write_dir_entries(
        &fs,
        deep_cluster,
        vec![build_file_entry_set(
            "created_after_delete.txt",
            false,
            created_cluster,
            20,
            false,
        )],
    );
    write_dir_entries(
        &fs,
        mutate_cluster,
        vec![
            build_file_entry_set("rename_target.txt", false, rename_target_cluster, 13, false),
            build_file_entry_set("truncate_me.txt", false, truncate_cluster, 3, false),
            build_file_entry_set(
                "post_delete_rename_target.txt",
                false,
                post_rename_cluster,
                18,
                false,
            ),
            build_file_entry_set(
                "post_delete_truncate.txt",
                false,
                post_truncate_cluster,
                4,
                false,
            ),
        ],
    );
    write_file_data(&fs, created_cluster, b"created-after-delete");
    write_file_data(&fs, post_rename_cluster, b"post-delete-rename");
    write_file_data(&fs, post_truncate_cluster, b"abcd");
    fs.flush().unwrap();

    let root = tmp.path().join("matrix");
    assert!(!root.join("single_delete.txt").exists());
    assert!(!root.join("batch_1.txt").exists());
    assert!(!root.join("empty_dir").exists());
    assert!(!root.join("tree").exists());
    assert!(!root.join("mixed_dir").exists());
    assert!(!root.join("mixed_file.txt").exists());
    assert_eq!(
        std::fs::read(root.join("after_delete/deep/created_after_delete.txt")).unwrap(),
        b"created-after-delete"
    );
    assert_eq!(
        std::fs::read(root.join("mutate/post_delete_rename_target.txt")).unwrap(),
        b"post-delete-rename"
    );
    assert_eq!(
        std::fs::read(root.join("mutate/post_delete_truncate.txt")).unwrap(),
        b"abcd"
    );
}

#[test]
fn write_interpreter_commits_directory_created_on_reused_file_cluster() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();

    let matrix_cluster = 410;
    let reused_cluster = 411;
    let deep_cluster = 412;
    let file_cluster = 413;

    write_root_entries(
        &fs,
        vec![build_file_entry_set(
            "matrix",
            true,
            matrix_cluster,
            0,
            false,
        )],
    );
    fs.flush().unwrap();

    write_dir_entries(
        &fs,
        matrix_cluster,
        vec![build_file_entry_set(
            "old.txt",
            false,
            reused_cluster,
            3,
            false,
        )],
    );
    write_file_data(&fs, reused_cluster, b"old");
    fs.flush().unwrap();

    write_dir_entries(
        &fs,
        matrix_cluster,
        vec![build_file_entry_set(
            "after_delete",
            true,
            reused_cluster,
            0,
            false,
        )],
    );
    write_dir_entries(
        &fs,
        reused_cluster,
        vec![build_file_entry_set("deep", true, deep_cluster, 0, false)],
    );
    write_dir_entries(
        &fs,
        deep_cluster,
        vec![build_file_entry_set(
            "created.txt",
            false,
            file_cluster,
            4,
            false,
        )],
    );
    write_file_data(&fs, file_cluster, b"data");
    fs.flush().unwrap();

    let root = tmp.path().join("matrix");
    assert!(!root.join("old.txt").exists());
    assert_eq!(
        std::fs::read(root.join("after_delete/deep/created.txt")).unwrap(),
        b"data"
    );
}

#[test]
fn write_interpreter_commits_file_created_on_reused_deleted_cluster() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], snapshot(), 16 * 1024 * 1024).unwrap();

    let old_cluster = 420;
    let mutate_cluster = 421;

    write_root_entries(
        &fs,
        vec![
            build_file_entry_set("old.txt", false, old_cluster, 3, false),
            build_file_entry_set("mutate", true, mutate_cluster, 0, false),
        ],
    );
    write_file_data(&fs, old_cluster, b"old");
    fs.flush().unwrap();

    write_root_entries(
        &fs,
        vec![build_file_entry_set(
            "mutate",
            true,
            mutate_cluster,
            0,
            false,
        )],
    );
    write_dir_entries(
        &fs,
        mutate_cluster,
        vec![build_file_entry_set(
            "new.txt",
            false,
            old_cluster,
            4,
            false,
        )],
    );
    write_file_data(&fs, old_cluster, b"data");
    fs.flush().unwrap();

    assert!(!tmp.path().join("old.txt").exists());
    assert_eq!(
        std::fs::read(tmp.path().join("mutate/new.txt")).unwrap(),
        b"data"
    );
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

    fs.write_at(fs.root_dir_offset_for_test(), &root_sector)
        .unwrap();
    fs.flush().unwrap();

    assert_eq!(std::fs::read(tmp.path().join("file.txt")).unwrap(), b"he");
}
