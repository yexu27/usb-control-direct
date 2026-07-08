//! Runtime allocation bitmap state.

use std::collections::BTreeSet;

use crate::exfat::layout::FIRST_CLUSTER;

#[derive(Debug, Clone)]
pub struct BitmapState {
    cluster_count: u32,
    allocated: BTreeSet<u32>,
}

impl BitmapState {
    pub fn new(cluster_count: u32) -> Self {
        Self {
            cluster_count,
            allocated: BTreeSet::new(),
        }
    }

    pub fn mark_allocated(&mut self, cluster: u32) -> Result<(), std::io::Error> {
        self.validate_cluster(cluster)?;
        self.allocated.insert(cluster);
        Ok(())
    }

    pub fn mark_free(&mut self, cluster: u32) -> Result<(), std::io::Error> {
        self.validate_cluster(cluster)?;
        self.allocated.remove(&cluster);
        Ok(())
    }

    pub fn is_allocated(&self, cluster: u32) -> bool {
        self.allocated.contains(&cluster)
    }

    pub fn allocated_clusters(&self) -> impl Iterator<Item = u32> + '_ {
        self.allocated.iter().copied()
    }

    pub fn cluster_count(&self) -> u32 {
        self.cluster_count
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
