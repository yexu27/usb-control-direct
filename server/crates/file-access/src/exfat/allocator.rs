//! 虚拟 exFAT 分配器。
//!
//! 该层只维护元数据和映射关系，不创建等同真实 U 盘容量的镜像文件。

use std::collections::HashMap;

use crate::exfat::layout::SECTOR_SIZE;
use crate::vfs::{NodeId, VfsIndex, VfsNodeKind};

#[derive(Debug, Clone)]
pub struct FileExtent {
    pub node_id: NodeId,
    pub file_offset: u64,
    pub valid_bytes: u32,
}

#[derive(Debug, Clone)]
pub enum VirtualSector {
    Metadata(Vec<u8>),
    FileData(FileExtent),
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorClass {
    Boot,
    Metadata,
    FileData,
    Free,
}

#[derive(Debug, Clone)]
pub struct ExfatAllocator {
    total_sectors: u64,
    metadata_sectors: HashMap<u64, Vec<u8>>,
    file_sectors: HashMap<u64, FileExtent>,
}

impl ExfatAllocator {
    pub fn build(index: &VfsIndex, source_size_bytes: u64) -> Result<Self, std::io::Error> {
        let total_sectors = source_size_bytes.div_ceil(SECTOR_SIZE as u64);
        let mut allocator = ExfatAllocator {
            total_sectors,
            metadata_sectors: HashMap::new(),
            file_sectors: HashMap::new(),
        };
        allocator.seed_minimal_metadata();
        allocator.seed_file_extents(index);
        Ok(allocator)
    }

    pub fn total_sectors(&self) -> u64 {
        self.total_sectors
    }

    pub fn estimated_memory_bytes(&self) -> usize {
        self.metadata_sectors.len() * SECTOR_SIZE as usize
            + self.file_sectors.len() * std::mem::size_of::<FileExtent>()
    }

    pub fn read_sector(&self, sector: u64) -> VirtualSector {
        if let Some(data) = self.metadata_sectors.get(&sector) {
            return VirtualSector::Metadata(data.clone());
        }
        if let Some(extent) = self.file_sectors.get(&sector) {
            return VirtualSector::FileData(extent.clone());
        }
        VirtualSector::Free
    }

    pub fn classify_sector(&self, sector: u64) -> SectorClass {
        if sector == 0 {
            return SectorClass::Boot;
        }
        if self.metadata_sectors.contains_key(&sector) {
            return SectorClass::Metadata;
        }
        if self.file_sectors.contains_key(&sector) {
            return SectorClass::FileData;
        }
        SectorClass::Free
    }

    fn seed_minimal_metadata(&mut self) {
        self.metadata_sectors
            .insert(0, vec![0u8; SECTOR_SIZE as usize]);
    }

    fn seed_file_extents(&mut self, index: &VfsIndex) {
        let mut sector = 2048_u64;
        for node in index.iter_nodes() {
            if node.kind != VfsNodeKind::File || node.is_virus || node.size == 0 {
                continue;
            }
            let sectors = node.size.div_ceil(SECTOR_SIZE as u64);
            for i in 0..sectors {
                let offset = i * SECTOR_SIZE as u64;
                let remaining = node.size.saturating_sub(offset);
                let valid_bytes = remaining.min(SECTOR_SIZE as u64) as u32;
                self.file_sectors.insert(
                    sector,
                    FileExtent {
                        node_id: node.id,
                        file_offset: offset,
                        valid_bytes,
                    },
                );
                sector += 1;
            }
        }
    }
}
