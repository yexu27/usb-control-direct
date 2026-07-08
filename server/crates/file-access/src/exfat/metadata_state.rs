//! Authoritative runtime exFAT metadata state.

use crate::exfat::bitmap_state::BitmapState;
use crate::exfat::directory_store::DirectoryStore;
use crate::exfat::fat_state::FatState;
use crate::exfat::layout::{DiskLayout, SECTORS_PER_CLUSTER};
use crate::exfat::sector_owner::{SectorOwner, SectorOwnerMap};
use crate::vfs::mutation::ClusterChain;
use crate::vfs::{NodeId, VfsIndex};

#[derive(Debug, Clone)]
pub struct ExfatMetadataState {
    fat: FatState,
    bitmap: BitmapState,
    directory_store: DirectoryStore,
    sector_owners: SectorOwnerMap,
}

impl ExfatMetadataState {
    pub fn new(
        fat: FatState,
        bitmap: BitmapState,
        directory_store: DirectoryStore,
        sector_owners: SectorOwnerMap,
    ) -> Self {
        Self {
            fat,
            bitmap,
            directory_store,
            sector_owners,
        }
    }

    pub fn directory_store(&self) -> &DirectoryStore {
        &self.directory_store
    }

    pub fn directory_clusters(&self, path: &str) -> Option<&[u32]> {
        self.directory_store.directory_clusters(path)
    }

    pub fn owner_of(&self, sector: u64) -> SectorOwner {
        self.sector_owners.owner_of(sector)
    }

    pub fn allocated_clusters(&self) -> impl Iterator<Item = u32> + '_ {
        self.bitmap.allocated_clusters()
    }

    pub fn explicit_ranges(&self) -> Vec<(u64, u64, SectorOwner)> {
        self.sector_owners.explicit_ranges()
    }

    pub fn is_allocated(&self, cluster: u32) -> bool {
        self.bitmap.is_allocated(cluster)
    }

    pub fn fat_entry_for(&self, cluster: u32) -> Option<u32> {
        self.fat.entry_for(cluster)
    }

    pub(crate) fn chain_from(&self, first_cluster: u32) -> Result<Vec<u32>, std::io::Error> {
        self.fat.chain_from(first_cluster)
    }

    pub fn fat_cluster_count(&self) -> u32 {
        self.fat.cluster_count()
    }

    pub fn bitmap_cluster_count(&self) -> u32 {
        self.bitmap.cluster_count()
    }

    pub fn set_directory_chain(
        &mut self,
        layout: &DiskLayout,
        node_id: NodeId,
        virtual_path: String,
        clusters: Vec<u32>,
    ) -> Result<(), std::io::Error> {
        self.directory_store
            .insert_directory(virtual_path.clone(), node_id, clusters.clone());
        self.fat.set_chain(&clusters)?;
        for cluster in clusters {
            self.bitmap.mark_allocated(cluster)?;
            let owner = if virtual_path == "/" {
                SectorOwner::RootDirectory
            } else {
                SectorOwner::DirectoryData { node_id: node_id.0 }
            };
            self.sector_owners.mark_range(
                layout.cluster_to_sector(cluster),
                SECTORS_PER_CLUSTER as u64,
                owner,
            )?;
        }
        Ok(())
    }

    pub fn set_file_chain(
        &mut self,
        layout: &DiskLayout,
        node_id: NodeId,
        chain: &ClusterChain,
        size: u64,
    ) -> Result<(), std::io::Error> {
        self.fat.set_chain(&chain.clusters)?;
        let mut remaining = size;
        let mut offset = 0_u64;
        for cluster in &chain.clusters {
            self.bitmap.mark_allocated(*cluster)?;
            for i in 0..SECTORS_PER_CLUSTER as u64 {
                let sector = layout.cluster_to_sector(*cluster) + i;
                let owner = if remaining == 0 {
                    SectorOwner::AllocatedZero {
                        node_id: node_id.0,
                        file_offset: offset,
                    }
                } else {
                    let valid_bytes = remaining.min(512) as u32;
                    SectorOwner::FileData {
                        node_id: node_id.0,
                        file_offset: offset,
                        valid_bytes,
                    }
                };
                self.sector_owners.mark_range(sector, 1, owner)?;
                remaining = remaining.saturating_sub(512);
                offset += 512;
            }
        }
        Ok(())
    }

    pub fn rename_subtree(&mut self, from: &str, to: &str) {
        self.directory_store.rename_subtree(from, to);
    }

    pub fn remove_subtree(
        &mut self,
        layout: &DiskLayout,
        virtual_path: &str,
    ) -> Result<Vec<u32>, std::io::Error> {
        let removed_clusters = self.directory_store.remove_subtree(virtual_path);
        for cluster in &removed_clusters {
            self.bitmap.mark_free(*cluster)?;
            self.fat.mark_free(*cluster)?;
            self.sector_owners.mark_range(
                layout.cluster_to_sector(*cluster),
                SECTORS_PER_CLUSTER as u64,
                SectorOwner::FreeCluster { cluster: *cluster },
            )?;
        }
        Ok(removed_clusters)
    }

    pub fn mark_range(
        &mut self,
        start: u64,
        len: u64,
        owner: SectorOwner,
    ) -> Result<(), std::io::Error> {
        self.sector_owners.mark_range(start, len, owner)
    }

    pub fn mark_cluster_free(
        &mut self,
        layout: &DiskLayout,
        cluster: u32,
    ) -> Result<(), std::io::Error> {
        self.bitmap.mark_free(cluster)?;
        self.fat.mark_free(cluster)?;
        self.sector_owners.mark_range(
            layout.cluster_to_sector(cluster),
            SECTORS_PER_CLUSTER as u64,
            SectorOwner::FreeCluster { cluster },
        )
    }

    pub fn mark_file_chain(
        &mut self,
        layout: &DiskLayout,
        id: NodeId,
        chain: &ClusterChain,
        size: u64,
    ) -> Result<(), std::io::Error> {
        self.set_file_chain(layout, id, chain, size)
    }

    pub fn validate(&self, index: &VfsIndex, layout: &DiskLayout) -> Result<(), std::io::Error> {
        for record in self.directory_store.records() {
            if index.node(record.node_id).is_none() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "directory {} points to missing VFS node",
                        record.virtual_path
                    ),
                ));
            }
            for cluster in &record.clusters {
                if !self.bitmap.is_allocated(*cluster) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "directory {} cluster {} is not allocated",
                            record.virtual_path, cluster
                        ),
                    ));
                }
                let owner = self.owner_of(layout.cluster_to_sector(*cluster));
                let valid_owner = if record.virtual_path == "/" {
                    matches!(owner, SectorOwner::RootDirectory)
                } else {
                    matches!(owner, SectorOwner::DirectoryData { node_id } if node_id == record.node_id.0)
                };
                if !valid_owner {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "directory {} cluster {} has invalid owner",
                            record.virtual_path, cluster
                        ),
                    ));
                }
            }
        }

        for cluster in self.bitmap.allocated_clusters() {
            let owner = self.owner_of(layout.cluster_to_sector(cluster));
            if matches!(owner, SectorOwner::FreeCluster { .. }) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("allocated cluster {cluster} still marked free"),
                ));
            }
        }
        Ok(())
    }
}
