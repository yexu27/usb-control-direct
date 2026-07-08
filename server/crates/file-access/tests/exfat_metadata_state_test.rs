use file_access::exfat::bitmap_state::BitmapState;
use file_access::exfat::directory_store::DirectoryStore;
use file_access::exfat::fat_state::FatState;
use file_access::exfat::layout::{DiskLayout, FAT_END_OF_CHAIN};
use file_access::exfat::metadata_state::ExfatMetadataState;
use file_access::exfat::sector_owner::{SectorOwner, SectorOwnerMap};
use file_access::vfs::mutation::ClusterChain;
use file_access::vfs::NodeId;

fn metadata_state() -> (DiskLayout, ExfatMetadataState) {
    let layout = DiskLayout::new_with_min_total_bytes(32, 16 * 1024 * 1024);
    let metadata = ExfatMetadataState::new(
        FatState::new(layout.cluster_count),
        BitmapState::new(layout.cluster_count),
        DirectoryStore::default(),
        SectorOwnerMap::new(layout.total_sectors),
    );
    (layout, metadata)
}

#[test]
fn metadata_state_registers_directory_chain_in_all_indexes() {
    let (layout, mut metadata) = metadata_state();
    let node_id = NodeId(1);
    let cluster = 10;

    metadata
        .set_directory_chain(&layout, node_id, "/dir".to_string(), vec![cluster])
        .unwrap();

    assert_eq!(metadata.directory_clusters("/dir").unwrap(), &[cluster]);
    assert!(metadata.is_allocated(cluster));
    assert!(matches!(
        metadata.owner_of(layout.cluster_to_sector(cluster)),
        SectorOwner::DirectoryData { node_id: 1 }
    ));
    assert_eq!(metadata.fat_entry_for(cluster), Some(FAT_END_OF_CHAIN));
}

#[test]
fn metadata_state_registers_file_chain_in_fat_bitmap_and_sector_owners() {
    let (layout, mut metadata) = metadata_state();
    let node_id = NodeId(2);
    let chain = ClusterChain {
        first_cluster: 20,
        clusters: vec![20, 21],
    };

    metadata
        .set_file_chain(&layout, node_id, &chain, layout.cluster_size() as u64 + 4)
        .unwrap();

    assert!(metadata.is_allocated(20));
    assert!(metadata.is_allocated(21));
    assert_eq!(metadata.fat_entry_for(20), Some(21));
    assert_eq!(metadata.fat_entry_for(21), Some(FAT_END_OF_CHAIN));
    assert!(matches!(
        metadata.owner_of(layout.cluster_to_sector(20)),
        SectorOwner::FileData { node_id: 2, .. }
    ));
    assert!(matches!(
        metadata.owner_of(layout.cluster_to_sector(21)),
        SectorOwner::FileData { node_id: 2, .. }
    ));
}
