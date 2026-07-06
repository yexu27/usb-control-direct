use file_access::exfat::boot::{generate_boot_region, generate_mbr};
use file_access::exfat::layout::{
    DiskLayout, BYTES_PER_SECTOR_SHIFT, EXFAT_SIGNATURE, PARTITION_OFFSET_SECTORS,
    SECTORS_PER_CLUSTER_SHIFT, SECTOR_SIZE,
};

#[test]
fn boot_sector_signature() {
    let layout = DiskLayout::new(100);
    let boot = generate_boot_region(&layout);
    assert_eq!(boot[0], 0xEB);
    assert_eq!(boot[1], 0x76);
    assert_eq!(boot[2], 0x90);
    assert_eq!(&boot[3..11], EXFAT_SIGNATURE);
    assert_eq!(boot[510], 0x55);
    assert_eq!(boot[511], 0xAA);
}

#[test]
fn boot_sector_fields() {
    let layout = DiskLayout::new(100);
    let boot = generate_boot_region(&layout);
    assert_eq!(boot[108], BYTES_PER_SECTOR_SHIFT);
    assert_eq!(boot[109], SECTORS_PER_CLUSTER_SHIFT);
    assert_eq!(boot[110], 1);
    let part_offset = u64::from_le_bytes(boot[64..72].try_into().unwrap());
    assert_eq!(part_offset, 2048);
    let vol_len = u64::from_le_bytes(boot[72..80].try_into().unwrap());
    assert_eq!(vol_len, layout.volume_length_sectors);
    let fat_off = u32::from_le_bytes(boot[80..84].try_into().unwrap());
    assert_eq!(fat_off as u64, layout.fat_offset_sectors);
    let fat_len = u32::from_le_bytes(boot[84..88].try_into().unwrap());
    assert_eq!(fat_len as u64, layout.fat_length_sectors);
    let heap_off = u32::from_le_bytes(boot[88..92].try_into().unwrap());
    assert_eq!(heap_off as u64, layout.cluster_heap_offset_sectors);
    let cluster_count = u32::from_le_bytes(boot[92..96].try_into().unwrap());
    assert_eq!(cluster_count, layout.cluster_count);
    let root_cluster = u32::from_le_bytes(boot[96..100].try_into().unwrap());
    assert_eq!(root_cluster, 2);
}

#[test]
fn boot_region_size_is_12_sectors() {
    let layout = DiskLayout::new(100);
    let boot = generate_boot_region(&layout);
    assert_eq!(boot.len(), 12 * SECTOR_SIZE as usize);
}

#[test]
fn extended_boot_sectors_have_signature() {
    let layout = DiskLayout::new(100);
    let boot = generate_boot_region(&layout);
    for i in 1..=8 {
        let offset = i * SECTOR_SIZE as usize;
        let sig_offset = offset + SECTOR_SIZE as usize - 4;
        let sig = u32::from_le_bytes(boot[sig_offset..sig_offset + 4].try_into().unwrap());
        assert_eq!(sig, 0xAA550000, "Extended Boot Sector {} signature", i);
    }
}

#[test]
fn boot_checksum_sector_is_filled() {
    let layout = DiskLayout::new(100);
    let boot = generate_boot_region(&layout);
    let checksum_offset = 11 * SECTOR_SIZE as usize;
    let first_checksum = u32::from_le_bytes(
        boot[checksum_offset..checksum_offset + 4]
            .try_into()
            .unwrap(),
    );
    for i in 0..(SECTOR_SIZE as usize / 4) {
        let offset = checksum_offset + i * 4;
        let val = u32::from_le_bytes(boot[offset..offset + 4].try_into().unwrap());
        assert_eq!(val, first_checksum, "Checksum at position {}", i);
    }
    assert_ne!(first_checksum, 0);
}

#[test]
fn mbr_partition_fields_match_layout() {
    let layout = DiskLayout::new(100);
    let mbr = generate_mbr(&layout).unwrap();

    assert_eq!(mbr[510], 0x55);
    assert_eq!(mbr[511], 0xAA);
    assert_eq!(mbr[446 + 4], 0x07);
    assert_eq!(
        u32::from_le_bytes(mbr[446 + 8..446 + 12].try_into().unwrap()) as u64,
        PARTITION_OFFSET_SECTORS
    );
    assert_eq!(
        u32::from_le_bytes(mbr[446 + 12..446 + 16].try_into().unwrap()) as u64,
        layout.volume_length_sectors
    );
}

#[test]
fn mbr_rejects_unrepresentable_volume_length() {
    let layout = DiskLayout {
        fat_offset_sectors: 24,
        fat_length_sectors: 1,
        cluster_heap_offset_sectors: 32,
        cluster_count: 1,
        total_sectors: PARTITION_OFFSET_SECTORS + u32::MAX as u64 + 1,
        volume_length_sectors: u32::MAX as u64 + 1,
    };

    let err = generate_mbr(&layout).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}
