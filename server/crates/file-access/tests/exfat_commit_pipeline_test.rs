use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::exfat::layout::{PARTITION_OFFSET_SECTORS, SECTOR_SIZE};
use file_access::types::{ControlledEntry, ExecFileType, PolicySnapshot};

fn rw_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn exec_control_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        ..rw_snapshot()
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
fn facade_write_at_commits_file_created_inside_existing_empty_directory_without_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("t2")).unwrap();
    let tree = vec![dir(tmp.path().join("t2"), "t2", vec![])];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, rw_snapshot(), 16 * 1024 * 1024).unwrap();

    let t2_cluster = root_entry_cluster(&fs, "t2");
    let file_cluster = 700;
    write_file_data(&fs, file_cluster, b"runtime-ok");
    write_dir_entries(
        &fs,
        t2_cluster,
        vec![build_file_entry_set(
            "from_windows.txt",
            false,
            file_cluster,
            10,
            false,
        )],
    );

    assert_eq!(
        std::fs::read(tmp.path().join("t2/from_windows.txt")).unwrap(),
        b"runtime-ok"
    );
    fs.flush().unwrap();
    assert_eq!(
        std::fs::read(tmp.path().join("t2/from_windows.txt")).unwrap(),
        b"runtime-ok"
    );
}

#[test]
fn boot_region_write_is_virtual_metadata_commit_not_pending_transaction_overlay() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], rw_snapshot(), 16 * 1024 * 1024).unwrap();
    let boot_offset = PARTITION_OFFSET_SECTORS * SECTOR_SIZE as u64;
    let mut sector = fs.read_at(boot_offset, SECTOR_SIZE as usize).unwrap();
    sector[112] ^= 0x01;

    fs.write_at(boot_offset, &sector).unwrap();
    let reread = fs.read_at(boot_offset, SECTOR_SIZE as usize).unwrap();

    assert_eq!(reread, sector);
    assert!(tmp.path().read_dir().unwrap().next().is_none());
}

#[test]
fn metadata_write_without_committed_mutation_is_not_exposed_to_virtual_reads() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = VirtualExfatFs::build(tmp.path(), &[], rw_snapshot(), 16 * 1024 * 1024).unwrap();

    let original = fs.read_at(fs.root_dir_offset_for_test(), 512).unwrap();
    let mut unknown_directory_update = vec![0u8; 512];
    unknown_directory_update[0] = 0xab;
    unknown_directory_update[1..5].copy_from_slice(b"junk");

    fs.write_at(fs.root_dir_offset_for_test(), &unknown_directory_update)
        .unwrap();

    assert_eq!(
        fs.read_at(fs.root_dir_offset_for_test(), 512).unwrap(),
        original,
        "metadata writes that do not resolve to committed mutations must not create virtual-only state"
    );
}

#[test]
fn root_directory_rewrite_with_existing_blocked_placeholder_allows_new_regular_file() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("blocked.exe"), b"blocked-real-content").unwrap();
    let tree = vec![controlled_file(
        tmp.path().join("blocked.exe"),
        "blocked.exe",
        20,
        Some(ExecFileType::Pe),
    )];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, exec_control_snapshot(), 16 * 1024 * 1024)
        .unwrap();

    let mut root = fs.read_at(fs.root_dir_offset_for_test(), 4096).unwrap();
    let existing_entries = parse_entry_sets(&root).unwrap();
    let blocked = existing_entries
        .iter()
        .find(|entry| entry.name == "blocked.exe")
        .unwrap();
    assert_ne!(blocked.first_cluster, 0);

    let new_cluster = 700;
    write_file_data(&fs, new_cluster, b"ok");
    let insert_at = existing_entries
        .iter()
        .map(|entry| entry.entry_offset + entry.set_len)
        .max()
        .unwrap();
    let new_entry = build_file_entry_set("created.txt", false, new_cluster, 2, false);
    root[insert_at..insert_at + new_entry.len()].copy_from_slice(&new_entry);

    fs.write_at(fs.root_dir_offset_for_test(), &root).unwrap();

    assert_eq!(
        std::fs::read(tmp.path().join("created.txt")).unwrap(),
        b"ok"
    );
    assert!(fs.lookup_path("/created.txt").is_some());
    assert!(fs.lookup_path("/blocked.exe").is_some());
}

#[test]
fn facade_write_at_commits_deep_empty_directory_and_zero_byte_file_without_flush() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("1/2/3")).unwrap();
    let tree = vec![dir(
        tmp.path().join("1"),
        "1",
        vec![dir(
            tmp.path().join("1/2"),
            "2",
            vec![dir(tmp.path().join("1/2/3"), "3", vec![])],
        )],
    )];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, rw_snapshot(), 16 * 1024 * 1024).unwrap();

    let one = root_entry_cluster(&fs, "1");
    let two = entry_cluster(&fs, one, "2");
    let three = entry_cluster(&fs, two, "3");
    let four_cluster = 701;
    write_dir_entries(
        &fs,
        three,
        vec![build_file_entry_set("4", true, four_cluster, 0, false)],
    );
    assert!(tmp.path().join("1/2/3/4").is_dir());

    write_dir_entries(
        &fs,
        four_cluster,
        vec![build_file_entry_set("4.txt", false, 0, 0, false)],
    );
    assert!(tmp.path().join("1/2/3/4/4.txt").is_file());
    assert_eq!(
        std::fs::metadata(tmp.path().join("1/2/3/4/4.txt"))
            .unwrap()
            .len(),
        0
    );
}
