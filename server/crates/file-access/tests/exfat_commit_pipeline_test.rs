use std::collections::HashSet;

use file_access::exfat::runtime_state::ExfatRuntimeState;
use file_access::exfat::sector_owner::SectorOwner;
use file_access::types::PolicySnapshot;
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
