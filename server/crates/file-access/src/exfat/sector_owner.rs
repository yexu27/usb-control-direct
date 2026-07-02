//! Runtime exFAT sector ownership map.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectorOwner {
    OutOfRange,
    Mbr,
    BootRegion,
    BackupBootRegion,
    Fat,
    AllocationBitmap,
    UpcaseTable,
    RootDirectory,
    DirectoryData { node_id: u64 },
    FileData { node_id: u64, file_offset: u64, valid_bytes: u32 },
    FileDataRange {
        node_id: u64,
        file_offset: u64,
        byte_len: u64,
    },
    AllocatedZero { node_id: u64, file_offset: u64 },
    FreeCluster { cluster: u32 },
    FreeClusterRange {
        first_cluster: u32,
        first_sector: u64,
        sectors_per_cluster: u64,
    },
    Reserved,
}

#[derive(Debug, Clone)]
struct OwnerRange {
    start: u64,
    len: u64,
    owner: SectorOwner,
}

#[derive(Debug, Clone)]
pub struct SectorOwnerMap {
    total_sectors: u64,
    cluster_range: Option<ClusterRange>,
    ranges: Vec<OwnerRange>,
}

#[derive(Debug, Clone)]
struct ClusterRange {
    first_cluster: u32,
    first_sector: u64,
    cluster_count: u32,
    sectors_per_cluster: u64,
}

impl SectorOwnerMap {
    pub fn new(total_sectors: u64) -> Self {
        Self {
            total_sectors,
            cluster_range: None,
            ranges: Vec::new(),
        }
    }

    pub fn register_cluster(&mut self, cluster: u32, start_sector: u64, sectors: u64) {
        self.register_cluster_range(cluster, start_sector, 1, sectors);
    }

    pub fn register_cluster_range(
        &mut self,
        first_cluster: u32,
        first_sector: u64,
        cluster_count: u32,
        sectors_per_cluster: u64,
    ) {
        self.cluster_range = Some(ClusterRange {
            first_cluster,
            first_sector,
            cluster_count,
            sectors_per_cluster,
        });
    }

    pub fn mark_range(
        &mut self,
        start: u64,
        len: u64,
        owner: SectorOwner,
    ) -> Result<(), std::io::Error> {
        if len == 0 {
            return Ok(());
        }
        let end = start.checked_add(len).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "sector range overflow")
        })?;
        if end > self.total_sectors {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "sector range out of virtual volume",
            ));
        }
        self.ranges.push(OwnerRange { start, len, owner });
        Ok(())
    }

    pub fn owner_of(&self, sector: u64) -> SectorOwner {
        if sector >= self.total_sectors {
            return SectorOwner::OutOfRange;
        }
        for range in self.ranges.iter().rev() {
            if sector >= range.start && sector < range.start + range.len {
                if let SectorOwner::FileDataRange {
                    node_id,
                    file_offset,
                    byte_len,
                } = &range.owner
                {
                    let sector_delta = sector - range.start;
                    let offset = *file_offset + sector_delta * 512;
                    let remaining = byte_len.saturating_sub(sector_delta * 512);
                    return SectorOwner::FileData {
                        node_id: *node_id,
                        file_offset: offset,
                        valid_bytes: remaining.min(512) as u32,
                    };
                }
                if let SectorOwner::FreeClusterRange {
                    first_cluster,
                    first_sector,
                    sectors_per_cluster,
                } = &range.owner
                {
                    let cluster =
                        *first_cluster + ((sector - *first_sector) / *sectors_per_cluster) as u32;
                    return SectorOwner::FreeCluster { cluster };
                }
                return range.owner.clone();
            }
        }
        if let Some(range) = &self.cluster_range {
            let total_len = range.cluster_count as u64 * range.sectors_per_cluster;
            if sector >= range.first_sector && sector < range.first_sector + total_len {
                let delta = sector - range.first_sector;
                let cluster = range.first_cluster + (delta / range.sectors_per_cluster) as u32;
                return SectorOwner::FreeCluster { cluster };
            }
        }
        SectorOwner::Reserved
    }

    pub fn explicit_ranges(&self) -> Vec<(u64, u64, SectorOwner)> {
        self.ranges
            .iter()
            .map(|range| (range.start, range.len, range.owner.clone()))
            .collect()
    }
}
