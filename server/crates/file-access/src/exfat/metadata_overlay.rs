//! Committed metadata overlay for virtual exFAT reads.

use std::collections::HashMap;

use crate::exfat::layout::SECTOR_SIZE;
use crate::exfat::transaction::CommittedMetadataUpdate;

#[derive(Debug, Default, Clone)]
pub struct MetadataOverlay {
    sectors: HashMap<u64, Vec<u8>>,
}

impl MetadataOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_sector(&self, sector: u64) -> Option<&[u8]> {
        self.sectors.get(&sector).map(Vec::as_slice)
    }

    pub fn insert_sector(&mut self, sector: u64, data: &[u8]) {
        self.sectors.insert(sector, padded_sector(data));
    }

    pub fn apply_committed(
        &mut self,
        updates: &[CommittedMetadataUpdate],
    ) -> Result<(), std::io::Error> {
        for update in updates {
            match update {
                CommittedMetadataUpdate::Sector { sector, data } => {
                    if data.len() != SECTOR_SIZE as usize {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "committed metadata sector must be exactly one sector",
                        ));
                    }
                    self.sectors.insert(*sector, data.clone());
                }
            }
        }
        Ok(())
    }
}

fn padded_sector(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; SECTOR_SIZE as usize];
    let copy_len = data.len().min(SECTOR_SIZE as usize);
    out[..copy_len].copy_from_slice(&data[..copy_len]);
    out
}
