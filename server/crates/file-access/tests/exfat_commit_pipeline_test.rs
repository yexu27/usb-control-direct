use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::exfat::runtime_state::ExfatRuntimeState;
use file_access::exfat::sector_owner::SectorOwner;
use file_access::types::{ControlledEntry, PolicySnapshot};
use file_access::vfs::mutation::{ClusterChain, FileDataPatch, FsMutation};

fn rw_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn readonly_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        permission: 0,
        ..rw_snapshot()
    }
}

fn first_free_cluster(state: &ExfatRuntimeState) -> u32 {
    for sector in 0..state.total_sectors() {
        if let SectorOwner::FreeCluster { cluster } = state.sector_owner(sector) {
            return cluster;
        }
    }
    panic!("expected free cluster");
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
    let data = fs.read_at(fs.cluster_offset_for_test(dir_cluster), 4096).unwrap();
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
fn committed_mutation_updates_real_fs_vfs_and_runtime() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], rw_snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let dir_cluster = first_free_cluster(&state);
    state
        .commit_mutation(FsMutation::CreateDir {
            parent: "/".to_string(),
            name: "dir".to_string(),
            chain: Some(ClusterChain {
                first_cluster: dir_cluster,
                clusters: vec![dir_cluster],
            }),
        })
        .unwrap();
    assert!(tmp.path().join("dir").is_dir());
    assert!(state.lookup_path("/dir").unwrap().is_dir());
    assert!(state.directory_store().directory_clusters("/dir").is_some());

    let file_cluster = first_free_cluster(&state);
    state
        .commit_mutation(FsMutation::CreateFile {
            parent: "/dir".to_string(),
            name: "created.txt".to_string(),
            size: 5,
            valid_data_len: 5,
            chain: Some(ClusterChain {
                first_cluster: file_cluster,
                clusters: vec![file_cluster],
            }),
            data_patches: vec![FileDataPatch {
                virtual_path: "/dir/created.txt".to_string(),
                offset: 0,
                data: b"hello".to_vec(),
            }],
        })
        .unwrap();

    assert_eq!(std::fs::read(tmp.path().join("dir/created.txt")).unwrap(), b"hello");
    assert_eq!(state.lookup_path("/dir/created.txt").unwrap().size, 5);
    let file_sector = state.cluster_to_sector(file_cluster);
    assert!(matches!(
        state.sector_owner(file_sector),
        SectorOwner::FileData { .. }
    ));
}

#[test]
fn rejected_mutation_does_not_update_real_fs_or_runtime() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], readonly_snapshot(), 16 * 1024 * 1024)
            .unwrap();

    let err = state
        .commit_mutation(FsMutation::CreateFile {
            parent: "/".to_string(),
            name: "blocked.txt".to_string(),
            size: 0,
            valid_data_len: 0,
            chain: None,
            data_patches: Vec::new(),
        })
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!tmp.path().join("blocked.txt").exists());
    assert!(state.lookup_path("/blocked.txt").is_none());
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
