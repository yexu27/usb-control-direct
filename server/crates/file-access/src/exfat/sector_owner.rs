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
    AllocatedZero { node_id: u64, file_offset: u64 },
    FreeCluster { cluster: u32 },
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
    cluster_lookup: Vec<(u32, u64, u64)>,
    ranges: Vec<OwnerRange>,
}

impl SectorOwnerMap {
    pub fn new(total_sectors: u64) -> Self {
        Self {
            total_sectors,
            cluster_lookup: Vec::new(),
            ranges: Vec::new(),
        }
    }

    pub fn register_cluster(&mut self, cluster: u32, start_sector: u64, sectors: u64) {
        self.cluster_lookup.push((cluster, start_sector, sectors));
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
                return range.owner.clone();
            }
        }
        for (cluster, start, len) in &self.cluster_lookup {
            if sector >= *start && sector < *start + *len {
                return SectorOwner::FreeCluster { cluster: *cluster };
            }
        }
        SectorOwner::Reserved
    }
}
