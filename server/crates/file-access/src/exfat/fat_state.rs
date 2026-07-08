//! Runtime FAT state.

use std::collections::HashMap;

use crate::exfat::layout::{FAT_END_OF_CHAIN, FIRST_CLUSTER};

#[derive(Debug, Clone)]
pub struct FatState {
    cluster_count: u32,
    entries: HashMap<u32, u32>,
}

impl FatState {
    pub fn new(cluster_count: u32) -> Self {
        Self {
            cluster_count,
            entries: HashMap::new(),
        }
    }

    pub fn set_chain(&mut self, chain: &[u32]) -> Result<(), std::io::Error> {
        if chain.is_empty() {
            return Ok(());
        }
        for cluster in chain {
            self.validate_cluster(*cluster)?;
        }
        for pair in chain.windows(2) {
            self.entries.insert(pair[0], pair[1]);
        }
        self.entries
            .insert(*chain.last().unwrap(), FAT_END_OF_CHAIN);
        Ok(())
    }

    pub fn mark_free(&mut self, cluster: u32) -> Result<(), std::io::Error> {
        self.validate_cluster(cluster)?;
        self.entries.remove(&cluster);
        Ok(())
    }

    pub fn entry_for(&self, cluster: u32) -> Option<u32> {
        self.entries.get(&cluster).copied()
    }

    pub fn cluster_count(&self) -> u32 {
        self.cluster_count
    }

    pub fn chain_from(&self, start: u32) -> Result<Vec<u32>, std::io::Error> {
        self.validate_cluster(start)?;
        let mut out = Vec::new();
        let mut current = start;
        loop {
            out.push(current);
            let next = *self.entries.get(&current).unwrap_or(&FAT_END_OF_CHAIN);
            if next == FAT_END_OF_CHAIN {
                return Ok(out);
            }
            self.validate_cluster(next)?;
            current = next;
        }
    }

    fn validate_cluster(&self, cluster: u32) -> Result<(), std::io::Error> {
        if cluster < FIRST_CLUSTER || cluster >= FIRST_CLUSTER + self.cluster_count {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cluster out of range",
            ));
        }
        Ok(())
    }
}
