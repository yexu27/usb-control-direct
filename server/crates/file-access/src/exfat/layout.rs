//! exFAT 磁盘布局常量与偏移计算。
//!
//! 基于 exFAT 规范定义虚拟卷的布局参数。所有偏移和大小以扇区为单位。

/// 扇区大小（字节）。
pub const SECTOR_SIZE: u32 = 512;

/// 簇大小（字节）。
pub const CLUSTER_SIZE: u32 = 4096;

/// 每簇扇区数。
pub const SECTORS_PER_CLUSTER: u32 = 8;

/// SectorsPerClusterShift（log2(8) = 3）。
pub const SECTORS_PER_CLUSTER_SHIFT: u8 = 3;

/// BytesPerSectorShift（log2(512) = 9）。
pub const BYTES_PER_SECTOR_SHIFT: u8 = 9;

/// MBR 所在扇区。
pub const MBR_SECTOR: u64 = 0;

/// 分区起始扇区偏移（标准对齐）。
pub const PARTITION_OFFSET_SECTORS: u64 = 2048;

/// Boot Region 大小（扇区）：Boot Sector + 8 Extended + OEM + Reserved + Checksum = 12。
pub const BOOT_REGION_SECTORS: u64 = 12;

/// exFAT 文件系统修订版本号 (1.00)。
pub const FS_REVISION: u16 = 0x0100;

/// exFAT 签名。
pub const EXFAT_SIGNATURE: &[u8; 8] = b"EXFAT   ";

/// Boot Sector 签名（offset 510-511）。
pub const BOOT_SIGNATURE: u16 = 0xAA55;

/// FAT 条目大小（字节）。
pub const FAT_ENTRY_SIZE: u32 = 4;

/// FAT 特殊值：媒体类型。
pub const FAT_MEDIA_TYPE: u32 = 0xFFFFFFF8;

/// FAT 特殊值：簇链终止。
pub const FAT_END_OF_CHAIN: u32 = 0xFFFFFFFF;

/// 第一个数据簇编号。
pub const FIRST_CLUSTER: u32 = 2;

/// exFAT 目录项大小（字节）。
pub const DIR_ENTRY_SIZE: u32 = 32;

/// 虚拟卷磁盘布局。
#[derive(Debug, Clone)]
pub struct DiskLayout {
    /// FAT 起始位置（相对于分区起始，扇区）。
    pub fat_offset_sectors: u64,
    /// FAT 大小（扇区）。
    pub fat_length_sectors: u64,
    /// Cluster Heap 起始位置（相对于分区起始，扇区）。
    pub cluster_heap_offset_sectors: u64,
    /// 数据簇总数（从 cluster 2 开始编号）。
    pub cluster_count: u32,
    /// 虚拟卷总扇区数（含 MBR、Gap、分区）。
    pub total_sectors: u64,
    /// 卷总扇区数（不含 MBR 和 Gap）。
    pub volume_length_sectors: u64,
}

impl DiskLayout {
    /// 根据数据簇数量计算磁盘布局。
    pub fn new(data_cluster_count: u32) -> Self {
        let fat_offset_sectors = BOOT_REGION_SECTORS * 2;
        let fat_entries = data_cluster_count as u64 + FIRST_CLUSTER as u64;
        let fat_bytes = fat_entries * FAT_ENTRY_SIZE as u64;
        let fat_length_sectors = (fat_bytes + SECTOR_SIZE as u64 - 1) / SECTOR_SIZE as u64;
        let raw_heap_offset = fat_offset_sectors + fat_length_sectors;
        let cluster_heap_offset_sectors = align_up(raw_heap_offset, SECTORS_PER_CLUSTER as u64);
        let volume_length_sectors = cluster_heap_offset_sectors
            + data_cluster_count as u64 * SECTORS_PER_CLUSTER as u64;
        let total_sectors = PARTITION_OFFSET_SECTORS + volume_length_sectors;

        DiskLayout {
            fat_offset_sectors,
            fat_length_sectors,
            cluster_heap_offset_sectors,
            cluster_count: data_cluster_count,
            total_sectors,
            volume_length_sectors,
        }
    }

    /// 将簇号转换为绝对扇区号。
    pub fn cluster_to_sector(&self, cluster: u32) -> u64 {
        let offset_in_heap = (cluster as u64 - FIRST_CLUSTER as u64) * SECTORS_PER_CLUSTER as u64;
        PARTITION_OFFSET_SECTORS + self.cluster_heap_offset_sectors + offset_in_heap
    }

    /// 将绝对扇区号转换为簇号。
    pub fn sector_to_cluster(&self, sector: u64) -> Option<u32> {
        let heap_start = PARTITION_OFFSET_SECTORS + self.cluster_heap_offset_sectors;
        if sector < heap_start {
            return None;
        }
        let offset = sector - heap_start;
        let cluster = (offset / SECTORS_PER_CLUSTER as u64) as u32 + FIRST_CLUSTER;
        if cluster < FIRST_CLUSTER || cluster >= FIRST_CLUSTER + self.cluster_count {
            return None;
        }
        Some(cluster)
    }
}

fn align_up(value: u64, alignment: u64) -> u64 {
    (value + alignment - 1) / alignment * alignment
}
