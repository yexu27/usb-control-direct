use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::types::{ControlledEntry, PolicySnapshot};

fn snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
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

fn write_file_data(fs: &VirtualExfatFs, cluster: u32, data: &[u8]) {
    let mut sector = vec![0u8; 512];
    sector[..data.len()].copy_from_slice(data);
    fs.write_at(fs.cluster_offset_for_test(cluster), &sector)
        .unwrap();
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

#[test]
fn facade_write_at_creates_file_inside_existing_directory_and_persists() {
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
