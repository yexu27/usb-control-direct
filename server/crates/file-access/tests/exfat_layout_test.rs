use file_access::exfat::layout::{
    DiskLayout, BOOT_REGION_SECTORS, CLUSTER_SIZE, MBR_SECTOR, MIN_VIRTUAL_VOLUME_BYTES,
    PARTITION_OFFSET_SECTORS, SECTORS_PER_CLUSTER, SECTOR_SIZE,
};

#[test]
fn constants_correct() {
    assert_eq!(SECTOR_SIZE, 512);
    assert_eq!(CLUSTER_SIZE, 4096);
    assert_eq!(SECTORS_PER_CLUSTER, 8);
    assert_eq!(MBR_SECTOR, 0);
    assert_eq!(PARTITION_OFFSET_SECTORS, 2048);
    assert_eq!(BOOT_REGION_SECTORS, 12);
}

#[test]
fn layout_with_small_volume() {
    let layout = DiskLayout::new(10);
    assert_eq!(layout.fat_offset_sectors, 24);
    assert_eq!(layout.fat_length_sectors, 1);
    assert_eq!(layout.cluster_heap_offset_sectors, 32);
    assert_eq!(layout.cluster_count, 10);
    let expected_total = PARTITION_OFFSET_SECTORS
        + layout.cluster_heap_offset_sectors
        + (layout.cluster_count as u64) * (SECTORS_PER_CLUSTER as u64);
    assert_eq!(layout.total_sectors, expected_total);
}

#[test]
fn layout_fat_grows_with_cluster_count() {
    let layout = DiskLayout::new(1000);
    assert_eq!(layout.fat_length_sectors, 8);
}

#[test]
fn cluster_to_sector_offset() {
    let layout = DiskLayout::new(10);
    let sector = layout.cluster_to_sector(2);
    assert_eq!(
        sector,
        PARTITION_OFFSET_SECTORS + layout.cluster_heap_offset_sectors
    );
    let sector3 = layout.cluster_to_sector(3);
    assert_eq!(sector3, sector + SECTORS_PER_CLUSTER as u64);
}

#[test]
fn sector_to_cluster() {
    let layout = DiskLayout::new(10);
    let first_data_sector = PARTITION_OFFSET_SECTORS + layout.cluster_heap_offset_sectors;
    assert_eq!(layout.sector_to_cluster(first_data_sector), Some(2));
    assert_eq!(layout.sector_to_cluster(first_data_sector + 8), Some(3));
    assert_eq!(layout.sector_to_cluster(0), None);
}

#[test]
fn layout_respects_minimum_virtual_volume_size() {
    let layout = DiskLayout::new_with_min_total_bytes(8, MIN_VIRTUAL_VOLUME_BYTES);

    assert!(layout.cluster_count >= 8);
    assert!(layout.total_sectors * SECTOR_SIZE as u64 >= MIN_VIRTUAL_VOLUME_BYTES);
}

#[test]
fn layout_respects_source_partition_size_when_larger_than_minimum() {
    let source_size = 64 * 1024 * 1024;
    let layout = DiskLayout::new_with_min_total_bytes(8, source_size);

    assert!(layout.cluster_count >= 8);
    assert!(layout.total_sectors * SECTOR_SIZE as u64 >= source_size);
}

#[test]
fn layout_exposes_total_byte_boundary() {
    let layout = DiskLayout::new(10);

    assert_eq!(
        layout.total_bytes(),
        layout.total_sectors * SECTOR_SIZE as u64
    );
    assert_eq!(layout.end_byte_exclusive(), layout.total_bytes());
}

#[test]
fn layout_accepts_mbr_representable_volume() {
    let layout = DiskLayout::new(10);

    layout.ensure_mbr_compatible().unwrap();
}

#[test]
fn layout_rejects_volume_length_beyond_mbr_u32() {
    let layout = DiskLayout {
        fat_offset_sectors: 24,
        fat_length_sectors: 1,
        cluster_heap_offset_sectors: 32,
        cluster_count: 1,
        total_sectors: PARTITION_OFFSET_SECTORS + u32::MAX as u64 + 1,
        volume_length_sectors: u32::MAX as u64 + 1,
    };

    let err = layout.ensure_mbr_compatible().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}
