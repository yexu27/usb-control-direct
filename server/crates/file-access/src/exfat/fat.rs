//! FAT 表生成。

use crate::exfat::layout::{FAT_END_OF_CHAIN, FAT_ENTRY_SIZE, FAT_MEDIA_TYPE, SECTOR_SIZE};

/// FAT 簇链构建器。
pub struct FatBuilder {
    entries: Vec<u32>,
}

impl FatBuilder {
    /// 创建 FAT 构建器。
    ///
    /// 参数:
    ///   - cluster_count: 总簇数（entry 0 和 1 是保留的）。
    pub fn new(cluster_count: u32) -> Self {
        let total = cluster_count as usize + 2; // entry 0, 1 是保留的
        let mut entries = vec![0u32; total];
        entries[0] = FAT_MEDIA_TYPE;
        entries[1] = FAT_END_OF_CHAIN;
        FatBuilder { entries }
    }

    /// 分配单簇条目（无链）。
    pub fn set_single(&mut self, cluster: u32) {
        self.entries[cluster as usize] = FAT_END_OF_CHAIN;
    }

    /// 分配连续簇链。
    ///
    /// 返回起始簇号。
    pub fn set_chain(&mut self, start_cluster: u32, count: u32) {
        for i in 0..count {
            let cluster = start_cluster + i;
            if i + 1 < count {
                self.entries[cluster as usize] = cluster + 1;
            } else {
                self.entries[cluster as usize] = FAT_END_OF_CHAIN;
            }
        }
    }

    /// 分配分段簇链：first_cluster → extra_start → extra_start+1 → ... → EOF。
    ///
    /// 用于根目录等场景：第一个簇号固定，额外簇在后续分配。
    pub fn set_chain_from_parts(&mut self, first_cluster: u32, extra_start: u32, extra_count: u32) {
        self.entries[first_cluster as usize] = extra_start;
        for i in 0..extra_count {
            let cluster = extra_start + i;
            if i + 1 < extra_count {
                self.entries[cluster as usize] = cluster + 1;
            } else {
                self.entries[cluster as usize] = FAT_END_OF_CHAIN;
            }
        }
    }

    /// 生成 FAT 表数据（扇区对齐）。
    pub fn build(&self, fat_length_sectors: u64) -> Vec<u8> {
        let size = fat_length_sectors as usize * SECTOR_SIZE as usize;
        let mut data = vec![0u8; size];
        for (i, &entry) in self.entries.iter().enumerate() {
            let offset = i * FAT_ENTRY_SIZE as usize;
            if offset + 4 <= data.len() {
                data[offset..offset + 4].copy_from_slice(&entry.to_le_bytes());
            }
        }
        data
    }
}
