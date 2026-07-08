use std::collections::HashSet;

use file_access::exfat::layout::{
    BOOT_REGION_SECTORS, FAT_END_OF_CHAIN, PARTITION_OFFSET_SECTORS, SECTOR_SIZE,
};
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

fn first_free_cluster(state: &ExfatRuntimeState) -> u32 {
    for sector in 0..state.total_sectors() {
        if let SectorOwner::FreeCluster { cluster } = state.sector_owner(sector) {
            return cluster;
        }
    }
    panic!("expected free cluster");
}

#[test]
fn metadata_state_registers_directory_chain_in_all_indexes() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], rw_snapshot(), 16 * 1024 * 1024)
            .unwrap();

    let cluster = first_free_cluster(&state);
    state
        .commit_mutation(FsMutation::CreateDir {
            parent: "/".to_string(),
            name: "dir".to_string(),
            chain: Some(ClusterChain {
                first_cluster: cluster,
                clusters: vec![cluster],
            }),
        })
        .unwrap();

    let node = state.lookup_path("/dir").unwrap();
    assert!(state.directory_store().directory_clusters("/dir").is_some());
    assert!(matches!(
        state.sector_owner(state.cluster_to_sector(cluster)),
        SectorOwner::DirectoryData { node_id } if node_id == node.id.0
    ));
    state.validate_consistency().unwrap();
}

#[test]
fn committed_file_chain_is_rendered_to_fat_and_bitmap_overlay() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], rw_snapshot(), 16 * 1024 * 1024)
            .unwrap();

    let cluster = first_free_cluster(&state);
    state
        .commit_mutation(FsMutation::CreateFile {
            parent: "/".to_string(),
            name: "created.txt".to_string(),
            size: 4,
            valid_data_len: 4,
            chain: Some(ClusterChain {
                first_cluster: cluster,
                clusters: vec![cluster],
            }),
            data_patches: vec![FileDataPatch {
                virtual_path: "/created.txt".to_string(),
                offset: 0,
                data: b"data".to_vec(),
            }],
        })
        .unwrap();

    let fat_entry_offset = (PARTITION_OFFSET_SECTORS + BOOT_REGION_SECTORS * 2)
        * SECTOR_SIZE as u64
        + cluster as u64 * 4;
    let fat_entry = state.read_at(fat_entry_offset, 4).unwrap();
    assert_eq!(
        u32::from_le_bytes(fat_entry.try_into().unwrap()),
        FAT_END_OF_CHAIN
    );

    let bitmap_bit = (cluster - 2) as usize;
    let bitmap_byte_offset =
        state.cluster_to_sector(3) * SECTOR_SIZE as u64 + (bitmap_bit / 8) as u64;
    let bitmap_byte = state.read_at(bitmap_byte_offset, 1).unwrap()[0];
    assert_ne!(bitmap_byte & (1 << (bitmap_bit % 8)), 0);
}
