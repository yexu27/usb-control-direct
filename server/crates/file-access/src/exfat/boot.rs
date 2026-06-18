//! Boot Region 生成。
//!
//! 包含 Boot Sector + 8 Extended Boot Sectors + OEM Parameters + Reserved + Boot Checksum。

use crate::exfat::layout::{
    DiskLayout, SECTOR_SIZE, BYTES_PER_SECTOR_SHIFT, SECTORS_PER_CLUSTER_SHIFT,
    PARTITION_OFFSET_SECTORS, EXFAT_SIGNATURE, FS_REVISION, FIRST_CLUSTER,
};

/// 生成 Main Boot Region（12 扇区，6144 字节）。
pub fn generate_boot_region(layout: &DiskLayout) -> Vec<u8> {
    let region_size = 12 * SECTOR_SIZE as usize;
    let mut region = vec![0u8; region_size];

    build_boot_sector(&mut region[..SECTOR_SIZE as usize], layout);

    for i in 1..=8 {
        let sig_offset = i * SECTOR_SIZE as usize + SECTOR_SIZE as usize - 4;
        region[sig_offset..sig_offset + 4].copy_from_slice(&0xAA550000u32.to_le_bytes());
    }

    let checksum = compute_boot_checksum(&region[..11 * SECTOR_SIZE as usize]);
    let checksum_offset = 11 * SECTOR_SIZE as usize;
    for i in 0..(SECTOR_SIZE as usize / 4) {
        let offset = checksum_offset + i * 4;
        region[offset..offset + 4].copy_from_slice(&checksum.to_le_bytes());
    }

    region
}

fn build_boot_sector(sector: &mut [u8], layout: &DiskLayout) {
    sector[0] = 0xEB;
    sector[1] = 0x76;
    sector[2] = 0x90;
    sector[3..11].copy_from_slice(EXFAT_SIGNATURE);
    sector[64..72].copy_from_slice(&PARTITION_OFFSET_SECTORS.to_le_bytes());
    sector[72..80].copy_from_slice(&layout.volume_length_sectors.to_le_bytes());
    sector[80..84].copy_from_slice(&(layout.fat_offset_sectors as u32).to_le_bytes());
    sector[84..88].copy_from_slice(&(layout.fat_length_sectors as u32).to_le_bytes());
    sector[88..92].copy_from_slice(&(layout.cluster_heap_offset_sectors as u32).to_le_bytes());
    sector[92..96].copy_from_slice(&layout.cluster_count.to_le_bytes());
    sector[96..100].copy_from_slice(&FIRST_CLUSTER.to_le_bytes());
    // VolumeSerialNumber — 使用进程启动时的时间戳低 32 位
    let serial = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0x12345678);
    sector[100..104].copy_from_slice(&serial.to_le_bytes());
    sector[104..106].copy_from_slice(&FS_REVISION.to_le_bytes());
    sector[108] = BYTES_PER_SECTOR_SHIFT;
    sector[109] = SECTORS_PER_CLUSTER_SHIFT;
    sector[110] = 1;
    sector[111] = 0x80;
    sector[112] = 0xFF;
    sector[510] = 0x55;
    sector[511] = 0xAA;
}

fn compute_boot_checksum(data: &[u8]) -> u32 {
    let mut checksum: u32 = 0;
    for (i, &byte) in data.iter().enumerate() {
        if i == 106 || i == 107 || i == 112 {
            continue;
        }
        checksum = if checksum & 1 != 0 {
            0x80000000u32.wrapping_add(checksum >> 1).wrapping_add(byte as u32)
        } else {
            (checksum >> 1).wrapping_add(byte as u32)
        };
    }
    checksum
}

/// 生成 MBR 扇区（512 字节）。
pub fn generate_mbr(layout: &DiskLayout) -> Vec<u8> {
    let mut mbr = vec![0u8; SECTOR_SIZE as usize];
    let entry_offset = 446;
    mbr[entry_offset] = 0x00;
    mbr[entry_offset + 4] = 0x07;
    mbr[entry_offset + 8..entry_offset + 12]
        .copy_from_slice(&(PARTITION_OFFSET_SECTORS as u32).to_le_bytes());
    let num_sectors = if layout.volume_length_sectors > u32::MAX as u64 {
        u32::MAX
    } else {
        layout.volume_length_sectors as u32
    };
    mbr[entry_offset + 12..entry_offset + 16].copy_from_slice(&num_sectors.to_le_bytes());
    mbr[510] = 0x55;
    mbr[511] = 0xAA;
    mbr
}
