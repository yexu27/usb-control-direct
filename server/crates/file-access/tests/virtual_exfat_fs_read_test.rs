use std::collections::HashSet;
use std::fs;

use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::exfat::layout::SECTOR_SIZE;
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

fn root_file_data_offset(fs: &VirtualExfatFs, name: &str) -> u64 {
    let root = fs
        .read_at(fs.root_dir_offset_for_test(), 4096)
        .expect("read root directory");
    let entries = parse_entry_sets(&root).expect("parse root directory");
    let file = entries
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("root directory contains {name}"));
    assert_ne!(
        file.first_cluster, 0,
        "{name} should have file data cluster"
    );
    fs.cluster_offset_for_test(file.first_cluster)
}

#[test]
fn read_at_allows_unblocked_file_data_from_real_mount() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("ok.txt"), b"hello").unwrap();
    let tree = vec![entry(tmp.path(), "ok.txt", 5, "txt")];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let offset = root_file_data_offset(&fs, "ok.txt");
    let data = fs.read_at(offset, SECTOR_SIZE as usize).unwrap();

    assert_eq!(&data[..5], b"hello");
}

#[test]
fn read_at_denies_blacklisted_file_data_without_hiding_node() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("bad.blocked"), b"secret").unwrap();
    let tree = vec![entry(tmp.path(), "bad.blocked", 6, "blocked")];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    assert!(fs.lookup_path("/bad.blocked").is_some());
    let offset = root_file_data_offset(&fs, "bad.blocked");
    let err = fs.read_at(offset, SECTOR_SIZE as usize).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn read_at_denies_executable_file_data_when_exec_control_enabled() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("setup.exe"), b"MZ executable").unwrap();
    let mut executable = entry(tmp.path(), "setup.exe", 13, "exe");
    executable.exec_type = Some(ExecFileType::Pe);
    let fs =
        VirtualExfatFs::build(tmp.path(), &[executable], snapshot(), 16 * 1024 * 1024).unwrap();

    assert!(fs.lookup_path("/setup.exe").is_some());
    let offset = root_file_data_offset(&fs, "setup.exe");
    let err = fs.read_at(offset, SECTOR_SIZE as usize).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn read_at_denies_autorun_file_data_when_auto_read_control_enabled() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("autorun.inf"), b"[autorun]").unwrap();
    let mut autorun = entry(tmp.path(), "autorun.inf", 9, "inf");
    autorun.is_autorun_inf = true;
    let fs = VirtualExfatFs::build(tmp.path(), &[autorun], snapshot(), 16 * 1024 * 1024).unwrap();

    assert!(fs.lookup_path("/autorun.inf").is_some());
    let offset = root_file_data_offset(&fs, "autorun.inf");
    let err = fs.read_at(offset, SECTOR_SIZE as usize).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn virtual_fs_reads_last_legal_sector() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();
    let tree = vec![entry(tmp.path(), "hello.txt", 5, "txt")];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let last_sector_offset = (fs.total_sectors() - 1) * SECTOR_SIZE as u64;
    let data = fs
        .read_at(last_sector_offset, SECTOR_SIZE as usize)
        .unwrap();

    assert_eq!(data.len(), SECTOR_SIZE as usize);
}

#[test]
fn virtual_fs_rejects_read_starting_at_disk_end() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();
    let tree = vec![entry(tmp.path(), "hello.txt", 5, "txt")];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let disk_end = fs.total_sectors() * SECTOR_SIZE as u64;
    let err = fs.read_at(disk_end, SECTOR_SIZE as usize).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn virtual_fs_rejects_read_crossing_disk_end() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();
    let tree = vec![entry(tmp.path(), "hello.txt", 5, "txt")];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024).unwrap();

    let disk_end = fs.total_sectors() * SECTOR_SIZE as u64;
    let err = fs.read_at(disk_end - 256, 512).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
}
