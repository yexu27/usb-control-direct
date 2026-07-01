//! Runtime exFAT state built from the controlled USB tree.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::exfat::bitmap_state::BitmapState;
use crate::exfat::directory_store::DirectoryStore;
use crate::exfat::fat_state::FatState;
use crate::exfat::layout::{
    BOOT_REGION_SECTORS, FIRST_CLUSTER, PARTITION_OFFSET_SECTORS, SECTORS_PER_CLUSTER,
};
use crate::exfat::sector_owner::{SectorOwner, SectorOwnerMap};
use crate::exfat::volume::{FileDataSectorInfo, VirtualVolume};
use crate::types::{ControlledEntry, PolicySnapshot};
use crate::vfs::{VfsIndex, VfsNode};

#[derive(Debug, Clone)]
pub struct ExfatRuntimeState {
    index: VfsIndex,
    volume: VirtualVolume,
    directory_store: DirectoryStore,
    fat: FatState,
    bitmap: BitmapState,
    sector_owners: SectorOwnerMap,
    snapshot: PolicySnapshot,
}

impl ExfatRuntimeState {
    pub fn from_controlled_tree(
        mount_root: &Path,
        entries: &[ControlledEntry],
        snapshot: PolicySnapshot,
        source_size_bytes: u64,
    ) -> Result<Self, std::io::Error> {
        let index = VfsIndex::from_controlled_tree(mount_root, entries)?;
        let volume = VirtualVolume::build_with_capacity(entries, &snapshot, source_size_bytes);
        let layout = volume.layout().clone();
        let mut directory_store = DirectoryStore::default();
        let mut fat = FatState::new(layout.cluster_count);
        let mut bitmap = BitmapState::new(layout.cluster_count);
        let mut sector_owners = SectorOwnerMap::new(layout.total_sectors);

        for cluster in FIRST_CLUSTER..FIRST_CLUSTER + layout.cluster_count {
            sector_owners.register_cluster(
                cluster,
                layout.cluster_to_sector(cluster),
                SECTORS_PER_CLUSTER as u64,
            );
        }

        sector_owners.mark_range(0, 1, SectorOwner::Mbr)?;
        sector_owners.mark_range(
            PARTITION_OFFSET_SECTORS,
            BOOT_REGION_SECTORS,
            SectorOwner::BootRegion,
        )?;
        sector_owners.mark_range(
            PARTITION_OFFSET_SECTORS + BOOT_REGION_SECTORS,
            BOOT_REGION_SECTORS,
            SectorOwner::BackupBootRegion,
        )?;
        sector_owners.mark_range(
            PARTITION_OFFSET_SECTORS + layout.fat_offset_sectors,
            layout.fat_length_sectors,
            SectorOwner::Fat,
        )?;

        let path_to_id = index
            .iter_nodes()
            .map(|node| (node.virtual_path.clone(), node.id))
            .collect::<HashMap<_, _>>();
        let real_to_node = index
            .iter_nodes()
            .map(|node| (normalize_path(&node.real_path), node))
            .collect::<HashMap<_, _>>();

        for (path, clusters) in volume.directory_cluster_entries() {
            if let Some(id) = path_to_id.get(&path) {
                directory_store.insert_directory(path.clone(), *id, clusters.clone());
                fat.set_chain(&clusters)?;
                for cluster in &clusters {
                    bitmap.mark_allocated(*cluster)?;
                    let owner = if path == "/" {
                        SectorOwner::RootDirectory
                    } else {
                        SectorOwner::DirectoryData { node_id: id.0 }
                    };
                    sector_owners.mark_range(
                        layout.cluster_to_sector(*cluster),
                        SECTORS_PER_CLUSTER as u64,
                        owner,
                    )?;
                }
            }
        }

        for sector in volume.metadata_sector_numbers() {
            let owner = match layout.sector_to_cluster(sector) {
                Some(cluster) => {
                    if bitmap.is_allocated(cluster) {
                        sector_owners.owner_of(sector)
                    } else {
                        SectorOwner::AllocationBitmap
                    }
                }
                None => sector_owners.owner_of(sector),
            };
            if matches!(owner, SectorOwner::FreeCluster { .. } | SectorOwner::Reserved) {
                sector_owners.mark_range(sector, 1, SectorOwner::UpcaseTable)?;
            }
        }

        for info in volume.file_data_sector_entries() {
            if let Some(node) = real_to_node.get(&normalize_path(&info.real_path)) {
                mark_file_sector(
                    &mut fat,
                    &mut bitmap,
                    &mut sector_owners,
                    &layout,
                    info,
                    node.id.0,
                )?;
            }
        }

        Ok(Self {
            index,
            volume,
            directory_store,
            fat,
            bitmap,
            sector_owners,
            snapshot,
        })
    }

    pub fn lookup_path(&self, path: &str) -> Option<&VfsNode> {
        self.index.lookup_path(path).and_then(|id| self.index.node(id))
    }

    pub fn directory_store(&self) -> &DirectoryStore {
        &self.directory_store
    }

    pub fn sector_owner(&self, sector: u64) -> SectorOwner {
        self.sector_owners.owner_of(sector)
    }

    pub fn total_sectors(&self) -> u64 {
        self.volume.total_sectors()
    }

    pub fn cluster_to_sector(&self, cluster: u32) -> u64 {
        self.volume.layout().cluster_to_sector(cluster)
    }
}

fn mark_file_sector(
    fat: &mut FatState,
    bitmap: &mut BitmapState,
    sector_owners: &mut SectorOwnerMap,
    layout: &crate::exfat::layout::DiskLayout,
    info: FileDataSectorInfo,
    node_id: u64,
) -> Result<(), std::io::Error> {
    if let Some(cluster) = layout.sector_to_cluster(info.sector) {
        if !bitmap.is_allocated(cluster) {
            bitmap.mark_allocated(cluster)?;
            fat.set_chain(&[cluster])?;
        }
    }
    sector_owners.mark_range(
        info.sector,
        1,
        SectorOwner::FileData {
            node_id,
            file_offset: info.offset,
            valid_bytes: info.valid_bytes,
        },
    )
}

fn normalize_path(path: &Path) -> PathBuf {
    path.components().collect()
}
