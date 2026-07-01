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
use crate::exfat::transaction::PendingTransaction;
use crate::exfat::transaction_resolver::TransactionResolver;
use crate::exfat::volume::{FileDataSectorInfo, VirtualVolume};
use crate::exfat::write_interpreter::WriteInterpreter;
use crate::types::{ControlledEntry, PolicySnapshot};
use crate::vfs::committer::RealFsCommitter;
use crate::vfs::mutation::{FsMutation, NodeKind};
use crate::vfs::operation_guard::{FsOperation, OperationGuard};
use crate::vfs::{VfsIndex, VfsNode};
use crate::vfs::VfsNodeKind;

#[derive(Debug, Clone)]
pub struct ExfatRuntimeState {
    index: VfsIndex,
    volume: VirtualVolume,
    directory_store: DirectoryStore,
    fat: FatState,
    bitmap: BitmapState,
    sector_owners: SectorOwnerMap,
    snapshot: PolicySnapshot,
    committer: RealFsCommitter,
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
        if FIRST_CLUSTER + 1 < FIRST_CLUSTER + layout.cluster_count {
            let bitmap_cluster = FIRST_CLUSTER + 1;
            bitmap.mark_allocated(bitmap_cluster)?;
            fat.set_chain(&[bitmap_cluster])?;
            sector_owners.mark_range(
                layout.cluster_to_sector(bitmap_cluster),
                SECTORS_PER_CLUSTER as u64,
                SectorOwner::AllocationBitmap,
            )?;
        }

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
            committer: RealFsCommitter::new(mount_root.to_path_buf()),
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

    pub fn record_write(
        &self,
        tx: &mut PendingTransaction,
        sector: u64,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        WriteInterpreter::new().record_sector_write(tx, sector, self.sector_owner(sector), data)
    }

    pub fn try_commit_closed_transaction(
        &mut self,
        tx: &PendingTransaction,
    ) -> Result<Vec<FsMutation>, std::io::Error> {
        let mutations = TransactionResolver::new().resolve_closed(tx, self)?;
        for mutation in &mutations {
            self.commit_mutation(mutation.clone())?;
        }
        Ok(mutations)
    }

    pub fn commit_mutation(&mut self, mutation: FsMutation) -> Result<(), std::io::Error> {
        self.check_mutation(&mutation)?;
        match &mutation {
            FsMutation::CreateDir { parent, name, .. } => {
                self.committer.create_dir(&join_virtual_path(parent, name))?;
            }
            FsMutation::CreateFile {
                parent,
                name,
                data_patches,
                ..
            } => {
                let path = join_virtual_path(parent, name);
                self.committer.create_file(&path)?;
                for patch in data_patches {
                    self.committer
                        .write_at(&patch.virtual_path, patch.offset, &patch.data)?;
                    self.committer.flush_file(&patch.virtual_path)?;
                }
            }
            FsMutation::WriteFile {
                virtual_path,
                offset,
                data,
            } => {
                self.committer.write_at(virtual_path, *offset, data)?;
                self.committer.flush_file(virtual_path)?;
            }
            FsMutation::Truncate { virtual_path, len } => {
                self.committer.truncate(virtual_path, *len)?;
                self.committer.flush_file(virtual_path)?;
            }
            FsMutation::Rename { from, to, .. } => {
                self.committer.rename(from, to)?;
            }
            FsMutation::Delete { virtual_path, kind } => match kind {
                NodeKind::File => self.committer.delete_file(virtual_path)?,
                NodeKind::Directory => self.committer.delete_dir(virtual_path)?,
            },
            FsMutation::RewriteFile {
                virtual_path,
                size,
                data_patches,
                ..
            } => {
                self.committer.truncate(virtual_path, *size)?;
                for patch in data_patches {
                    self.committer
                        .write_at(&patch.virtual_path, patch.offset, &patch.data)?;
                }
                self.committer.flush_file(virtual_path)?;
            }
        }
        self.index.apply_mutation(&mutation)?;
        self.refresh_runtime_metadata_after_mutation(&mutation)?;
        Ok(())
    }

    pub fn parent_path_for_directory_owner(&self, owner: &SectorOwner) -> Option<String> {
        match owner {
            SectorOwner::RootDirectory => Some("/".to_string()),
            SectorOwner::DirectoryData { node_id } => self
                .index
                .iter_nodes()
                .find(|node| node.id.0 == *node_id)
                .map(|node| node.virtual_path.clone()),
            _ => None,
        }
    }

    pub fn immediate_children(&self, parent_path: &str) -> Vec<(String, String, NodeKind)> {
        let Some(parent_id) = self.index.lookup_path(parent_path) else {
            return Vec::new();
        };
        let Some(parent) = self.index.node(parent_id) else {
            return Vec::new();
        };
        parent
            .children
            .iter()
            .filter_map(|child_id| self.index.node(*child_id))
            .map(|node| {
                let kind = match node.kind {
                    VfsNodeKind::File => NodeKind::File,
                    VfsNodeKind::Directory => NodeKind::Directory,
                };
                (node.name.clone(), node.virtual_path.clone(), kind)
            })
            .collect()
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

impl ExfatRuntimeState {
    fn check_mutation(&self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        let guard = OperationGuard::new(self.snapshot.clone());
        match mutation {
            FsMutation::CreateDir { parent, name, .. } => guard.check(&FsOperation::CreateDir {
                virtual_path: join_virtual_path(parent, name),
            }),
            FsMutation::CreateFile { parent, name, .. } => guard.check(&FsOperation::CreateFile {
                virtual_path: join_virtual_path(parent, name),
            }),
            FsMutation::WriteFile { virtual_path, .. }
            | FsMutation::RewriteFile { virtual_path, .. } => guard.check(&FsOperation::WriteFile {
                virtual_path: virtual_path.clone(),
            }),
            FsMutation::Truncate { virtual_path, .. } => guard.check(&FsOperation::Truncate {
                virtual_path: virtual_path.clone(),
            }),
            FsMutation::Rename { from, to, .. } => guard.check(&FsOperation::Rename {
                from: from.clone(),
                to: to.clone(),
            }),
            FsMutation::Delete { virtual_path, .. } => guard.check(&FsOperation::Delete {
                virtual_path: virtual_path.clone(),
                is_virus: self
                    .lookup_path(virtual_path)
                    .map(|node| node.is_virus)
                    .unwrap_or(false),
            }),
        }
    }

    fn refresh_runtime_metadata_after_mutation(
        &mut self,
        mutation: &FsMutation,
    ) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::CreateDir { parent, name, chain } => {
                let path = join_virtual_path(parent, name);
                let id = self.index.lookup_path(&path).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "created dir not in VFS")
                })?;
                if let Some(chain) = chain {
                    self.directory_store
                        .insert_directory(path, id, chain.clusters.clone());
                    self.fat.set_chain(&chain.clusters)?;
                    for cluster in &chain.clusters {
                        self.bitmap.mark_allocated(*cluster)?;
                        self.sector_owners.mark_range(
                            self.volume.layout().cluster_to_sector(*cluster),
                            SECTORS_PER_CLUSTER as u64,
                            SectorOwner::DirectoryData { node_id: id.0 },
                        )?;
                    }
                }
            }
            FsMutation::CreateFile {
                parent,
                name,
                chain,
                size,
                ..
            } => {
                let path = join_virtual_path(parent, name);
                let id = self.index.lookup_path(&path).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "created file not in VFS")
                })?;
                if let Some(chain) = chain {
                    self.fat.set_chain(&chain.clusters)?;
                    let mut remaining = *size;
                    let mut offset = 0_u64;
                    for cluster in &chain.clusters {
                        self.bitmap.mark_allocated(*cluster)?;
                        for i in 0..SECTORS_PER_CLUSTER as u64 {
                            if remaining == 0 {
                                break;
                            }
                            let valid_bytes = remaining.min(crate::exfat::layout::SECTOR_SIZE as u64);
                            self.sector_owners.mark_range(
                                self.volume.layout().cluster_to_sector(*cluster) + i,
                                1,
                                SectorOwner::FileData {
                                    node_id: id.0,
                                    file_offset: offset,
                                    valid_bytes: valid_bytes as u32,
                                },
                            )?;
                            remaining = remaining.saturating_sub(valid_bytes);
                            offset += valid_bytes;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn join_virtual_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}
