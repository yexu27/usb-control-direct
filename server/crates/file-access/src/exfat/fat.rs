//! FAT 表生成。

use crate::exfat::layout::{FAT_ENTRY_SIZE, FAT_MEDIA_TYPE, FAT_END_OF_CHAIN, SECTOR_SIZE};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fat_reserved_entries() {
        let builder = FatBuilder::new(10);
        let data = builder.build(1);
        let e0 = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let e1 = u32::from_le_bytes(data[4..8].try_into().unwrap());
        assert_eq!(e0, FAT_MEDIA_TYPE);
        assert_eq!(e1, FAT_END_OF_CHAIN);
    }

    #[test]
    fn fat_single_cluster() {
        let mut builder = FatBuilder::new(10);
        builder.set_single(2);
        let data = builder.build(1);
        let e2 = u32::from_le_bytes(data[8..12].try_into().unwrap());
        assert_eq!(e2, FAT_END_OF_CHAIN);
    }

    #[test]
    fn fat_chain() {
        let mut builder = FatBuilder::new(10);
        builder.set_chain(2, 3);
        let data = builder.build(1);
        let e2 = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let e3 = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let e4 = u32::from_le_bytes(data[16..20].try_into().unwrap());
        assert_eq!(e2, 3);
        assert_eq!(e3, 4);
        assert_eq!(e4, FAT_END_OF_CHAIN);
    }
}
