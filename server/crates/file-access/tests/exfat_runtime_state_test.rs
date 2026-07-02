use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::layout::{PARTITION_OFFSET_SECTORS, SECTOR_SIZE};
use file_access::exfat::runtime_state::ExfatRuntimeState;
use file_access::exfat::sector_owner::SectorOwner;
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

#[test]
fn runtime_initial_state_contains_complete_tree_and_explicit_sector_owners() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("empty_dir")).unwrap();
    std::fs::write(tmp.path().join("zero.txt"), []).unwrap();
    std::fs::create_dir_all(tmp.path().join("1/2/3")).unwrap();
    std::fs::write(tmp.path().join("1/2/3/4.txt"), b"deep").unwrap();
    std::fs::write(tmp.path().join("normal.bin"), vec![0x33; SECTOR_SIZE as usize]).unwrap();

    let tree = vec![
        dir(tmp.path().join("empty_dir"), "empty_dir", vec![]),
        file(tmp.path().join("zero.txt"), "zero.txt", 0),
        dir(
            tmp.path().join("1"),
            "1",
            vec![dir(
                tmp.path().join("1/2"),
                "2",
                vec![dir(
                    tmp.path().join("1/2/3"),
                    "3",
                    vec![file(tmp.path().join("1/2/3/4.txt"), "4.txt", 4)],
                )],
            )],
        ),
        file(tmp.path().join("normal.bin"), "normal.bin", SECTOR_SIZE as u64),
    ];

    let state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024)
            .unwrap();

    let empty_dir = state.lookup_path("/empty_dir").unwrap();
    assert!(empty_dir.is_dir());
    let zero_file = state.lookup_path("/zero.txt").unwrap();
    assert!(!zero_file.is_dir());
    assert_eq!(zero_file.size, 0);
    assert!(zero_file.first_cluster.is_none());
    let deep_file = state.lookup_path("/1/2/3/4.txt").unwrap();
    assert!(!deep_file.is_dir());
    assert_eq!(deep_file.size, 4);

    for path in ["/", "/empty_dir", "/1", "/1/2", "/1/2/3"] {
        assert!(
            state.directory_store().directory_clusters(path).is_some(),
            "directory {path} should have its own directory cluster mapping"
        );
    }

    assert!(matches!(state.sector_owner(0), SectorOwner::Mbr));
    assert!(matches!(
        state.sector_owner(PARTITION_OFFSET_SECTORS),
        SectorOwner::BootRegion
    ));

    let root_sector = state
        .directory_store()
        .directory_clusters("/")
        .and_then(|clusters| clusters.first().copied())
        .map(|cluster| state.cluster_to_sector(cluster))
        .unwrap();
    assert!(matches!(
        state.sector_owner(root_sector),
        SectorOwner::RootDirectory
    ));

    let deep_dir_sector = state
        .directory_store()
        .directory_clusters("/1/2/3")
        .and_then(|clusters| clusters.first().copied())
        .map(|cluster| state.cluster_to_sector(cluster))
        .unwrap();
    assert!(matches!(
        state.sector_owner(deep_dir_sector),
        SectorOwner::DirectoryData { .. }
    ));

    let mut saw_file_data = false;
    for sector in 0..state.total_sectors() {
        if matches!(state.sector_owner(sector), SectorOwner::FileData { .. }) {
            saw_file_data = true;
            break;
        }
    }
    assert!(saw_file_data, "normal file should have file data sector owner");

    let mut saw_free_cluster = false;
    for sector in 0..state.total_sectors() {
        if matches!(state.sector_owner(sector), SectorOwner::FreeCluster { .. }) {
            saw_free_cluster = true;
            break;
        }
    }
    assert!(saw_free_cluster, "virtual volume should expose explicit free clusters");

    state.validate_consistency().unwrap();
    assert!(matches!(
        state.sector_owner(state.total_sectors() + 1),
        SectorOwner::OutOfRange
    ));
}
