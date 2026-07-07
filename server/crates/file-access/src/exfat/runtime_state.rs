//! Runtime exFAT state built from the controlled USB tree.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::exfat::bitmap_state::BitmapState;
use crate::exfat::directory_store::DirectoryStore;
use crate::exfat::fat_state::FatState;
use crate::exfat::layout::{
    BOOT_REGION_SECTORS, FIRST_CLUSTER, PARTITION_OFFSET_SECTORS, SECTOR_SIZE,
    SECTORS_PER_CLUSTER,
};
use crate::exfat::sector_owner::{SectorOwner, SectorOwnerMap};
use crate::exfat::transaction::{PendingTransaction, TransactionWrite};
use crate::exfat::transaction_resolver::TransactionResolver;
use crate::exfat::volume::{FileDataRangeInfo, VirtualVolume};
use crate::exfat::write_interpreter::WriteInterpreter;
use crate::types::{blocked_placeholder_bytes, ControlledEntry, PolicySnapshot};
use crate::vfs::committer::RealFsCommitter;
use crate::vfs::mutation::{FsMutation, NodeKind};
use crate::vfs::node::VfsFileView;
use crate::vfs::operation_guard::{FsOperation, OperationGuard};
use crate::vfs::VfsNodeKind;
use crate::vfs::{NodeId, VfsIndex, VfsNode};

#[derive(Debug, Clone)]
pub struct ExfatRuntimeState {
    index: VfsIndex,
    volume: VirtualVolume,
    directory_store: DirectoryStore,
    fat: FatState,
    bitmap: BitmapState,
    sector_owners: SectorOwnerMap,
    metadata_overlays: HashMap<u64, Vec<u8>>,
    snapshot: PolicySnapshot,
    committer: RealFsCommitter,
    pending_tx: PendingTransaction,
    next_tx_id: u64,
}

impl ExfatRuntimeState {
    pub fn from_controlled_tree(
        mount_root: &Path,
        entries: &[ControlledEntry],
        snapshot: PolicySnapshot,
        source_size_bytes: u64,
    ) -> Result<Self, std::io::Error> {
        let mut index = VfsIndex::from_controlled_tree(mount_root, entries, &snapshot)?;
        let volume = VirtualVolume::build_with_capacity(entries, &snapshot, source_size_bytes)?;
        let layout = volume.layout().clone();
        sync_vfs_clusters_from_volume(&mut index, &volume)?;
        let mut directory_store = DirectoryStore::default();
        let mut fat = FatState::new(layout.cluster_count);
        let mut bitmap = BitmapState::new(layout.cluster_count);
        let mut sector_owners = SectorOwnerMap::new(layout.total_sectors);

        sector_owners.register_cluster_range(
            FIRST_CLUSTER,
            layout.cluster_to_sector(FIRST_CLUSTER),
            layout.cluster_count,
            SECTORS_PER_CLUSTER as u64,
        );

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

        for info in volume.file_data_range_entries() {
            if let Some(node) = real_to_node.get(&normalize_path(&info.real_path)) {
                mark_file_range(&mut sector_owners, info, node.id.0)?;
            }
        }

        let state = Self {
            index,
            volume,
            directory_store,
            fat,
            bitmap,
            sector_owners,
            metadata_overlays: HashMap::new(),
            snapshot,
            committer: RealFsCommitter::new(mount_root.to_path_buf()),
            pending_tx: PendingTransaction::new(1),
            next_tx_id: 2,
        };
        state.validate_consistency()?;
        Ok(state)
    }

    pub fn lookup_path(&self, path: &str) -> Option<&VfsNode> {
        self.index.lookup_path(path).and_then(|id| self.index.node(id))
    }

    pub fn lookup_node_id(&self, path: &str) -> Option<NodeId> {
        self.index.lookup_path(path)
    }

    pub fn node(&self, node_id: NodeId) -> Option<&VfsNode> {
        self.index.node(node_id)
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

    fn file_node(&self, node_id: u64) -> Result<&VfsNode, std::io::Error> {
        self.index.node(NodeId(node_id)).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "file data owner not found in VFS")
        })
    }

    pub fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        self.validate_read_range(offset, len)?;
        let mut out = Vec::with_capacity(len);
        let mut current = offset;
        while out.len() < len {
            let sector = current / SECTOR_SIZE as u64;
            let sector_offset = (current % SECTOR_SIZE as u64) as usize;
            let take = (SECTOR_SIZE as usize - sector_offset).min(len - out.len());
            if let Some(data) = self.metadata_overlays.get(&sector) {
                out.extend_from_slice(&data[sector_offset..sector_offset + take]);
                current += take as u64;
                continue;
            }
            match self.sector_owner(sector) {
                SectorOwner::FileData {
                    node_id,
                    file_offset,
                    valid_bytes,
                } => {
                    let available = valid_bytes.saturating_sub(sector_offset as u32) as usize;
                    let read_len = take.min(available);
                    if read_len == 0 {
                        out.resize(out.len() + take, 0);
                    } else {
                        let node = self.file_node(node_id)?;
                        match &node.file_view {
                            Some(VfsFileView::BlockedPlaceholder { reason, .. }) => {
                                tracing::debug!(
                                    virtual_path = %node.virtual_path,
                                    reason = %reason,
                                    "阻断文件读取返回策略占位内容"
                                );
                                let data = read_placeholder_slice(file_offset, sector_offset, take);
                                out.extend_from_slice(&data);
                            }
                            _ => {
                                let mut file = std::fs::File::open(&node.real_path)?;
                                file.seek(SeekFrom::Start(file_offset + sector_offset as u64))?;
                                let mut data = vec![0u8; read_len];
                                let actual = file.read(&mut data)?;
                                out.extend_from_slice(&data[..actual]);
                                if actual < take {
                                    out.resize(out.len() + take - actual, 0);
                                }
                            }
                        }
                    }
                }
                SectorOwner::AllocatedZero {
                    node_id,
                    file_offset,
                } => {
                    let node = self.file_node(node_id)?;
                    match &node.file_view {
                        Some(VfsFileView::BlockedPlaceholder { reason, .. }) => {
                            tracing::debug!(
                                virtual_path = %node.virtual_path,
                                reason = %reason,
                                "阻断文件零填充区域读取返回 0"
                            );
                            out.resize(out.len() + take, 0);
                        }
                        _ => {
                            let mut file = std::fs::File::open(&node.real_path)?;
                            file.seek(SeekFrom::Start(file_offset + sector_offset as u64))?;
                            let mut data = vec![0u8; take];
                            let actual = file.read(&mut data)?;
                            out.extend_from_slice(&data[..actual]);
                            if actual < take {
                                out.resize(out.len() + take - actual, 0);
                            }
                        }
                    }
                }
                SectorOwner::OutOfRange => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "read out of virtual exFAT range",
                    ));
                }
                _ => match self.volume.read_sector(sector)? {
                    crate::types::SectorContent::Metadata(data) => {
                        out.extend_from_slice(&data[sector_offset..sector_offset + take]);
                    }
                    crate::types::SectorContent::FileData {
                        real_path,
                        offset,
                        valid_bytes,
                    } => {
                        let available = valid_bytes.saturating_sub(sector_offset as u32) as usize;
                        let read_len = take.min(available);
                        let mut file = std::fs::File::open(real_path)?;
                        file.seek(SeekFrom::Start(offset + sector_offset as u64))?;
                        let mut data = vec![0u8; read_len];
                        let actual = file.read(&mut data)?;
                        out.extend_from_slice(&data[..actual]);
                        if actual < take {
                            out.resize(out.len() + take - actual, 0);
                        }
                    }
                    crate::types::SectorContent::Zero => out.resize(out.len() + take, 0),
                },
            }
            current += take as u64;
        }
        Ok(out)
    }

    fn validate_read_range(&self, offset: u64, len: usize) -> Result<(), std::io::Error> {
        let end = offset.checked_add(len as u64).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "read range overflows virtual exFAT disk",
            )
        })?;
        if end > self.volume.layout().end_byte_exclusive() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "read range exceeds virtual exFAT disk",
            ));
        }
        Ok(())
    }

    pub(crate) fn directory_image(
        &self,
        path: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, std::io::Error> {
        let Some(clusters) = self.directory_store.directory_clusters(path) else {
            return Ok(None);
        };
        let Some(first_cluster) = clusters.first() else {
            return Ok(None);
        };
        let first_sector = self.volume.layout().cluster_to_sector(*first_cluster);
        let mut data = Vec::with_capacity(
            clusters.len() * SECTORS_PER_CLUSTER as usize * SECTOR_SIZE as usize,
        );
        for cluster in clusters {
            let cluster_sector = self.volume.layout().cluster_to_sector(*cluster);
            for sector_offset in 0..SECTORS_PER_CLUSTER as u64 {
                let sector = cluster_sector + sector_offset;
                if let Some(overlay) = self.metadata_overlays.get(&sector) {
                    data.extend_from_slice(overlay);
                    continue;
                }
                match self.volume.read_sector(sector)? {
                    crate::types::SectorContent::Metadata(sector_data) => {
                        data.extend_from_slice(&sector_data);
                    }
                    _ => data.resize(data.len() + SECTOR_SIZE as usize, 0),
                }
            }
        }
        Ok(Some((first_sector, data)))
    }

    pub fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        if !data.is_empty() && self.snapshot.permission == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "只读权限禁止写入",
            ));
        }
        if offset % SECTOR_SIZE as u64 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "NBD write offset is not sector aligned",
            ));
        }
        let start_sector = offset / SECTOR_SIZE as u64;
        let mut recorded_transaction_write = false;
        for (i, chunk) in data.chunks(SECTOR_SIZE as usize).enumerate() {
            let sector = start_sector + i as u64;
            let owner = self.sector_owner(sector);
            match owner.clone() {
                SectorOwner::BootRegion | SectorOwner::BackupBootRegion => {
                    self.metadata_overlays
                        .insert(sector, padded_sector(chunk));
                }
                SectorOwner::FileData {
                    node_id,
                    file_offset,
                    valid_bytes,
                    ..
                } => {
                    let Some(node) = self.index.node(NodeId(node_id)) else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "file data owner not found in VFS",
                        ));
                    };
                    let write_len = (valid_bytes as usize).min(chunk.len());
                    self.commit_mutation(FsMutation::WriteFile {
                        virtual_path: node.virtual_path.clone(),
                        offset: file_offset,
                        data: chunk[..write_len].to_vec(),
                    })?;
                }
                SectorOwner::AllocatedZero {
                    node_id,
                    file_offset,
                } => {
                    let Some(node) = self.index.node(NodeId(node_id)) else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "file data owner not found in VFS",
                        ));
                    };
                    let known_remaining = node.size.saturating_sub(file_offset) as usize;
                    let write_len = if node.size == 0 {
                        chunk.len()
                    } else {
                        known_remaining.min(chunk.len())
                    };
                    self.commit_mutation(FsMutation::WriteFile {
                        virtual_path: node.virtual_path.clone(),
                        offset: file_offset,
                        data: chunk[..write_len].to_vec(),
                    })?;
                }
                _ => {
                    WriteInterpreter::new().record_sector_write(
                        &mut self.pending_tx,
                        sector,
                        owner,
                        chunk,
                    )?;
                    recorded_transaction_write = true;
                }
            }
        }
        if recorded_transaction_write {
            let tx = self.pending_tx.clone();
            match self.try_commit_closed_transaction(&tx) {
                Ok(mutations) => {
                    if !mutations.is_empty() {
                        self.apply_metadata_overlays_from_transaction(&tx);
                        self.pending_tx = PendingTransaction::new(self.next_tx_id);
                        self.next_tx_id += 1;
                    } else {
                        self.pending_tx.retain_deferred_data_writes();
                        if self.pending_tx.is_empty() {
                            self.pending_tx = PendingTransaction::new(self.next_tx_id);
                            self.next_tx_id += 1;
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        tx_id = tx.id(),
                        error = %e,
                        "exFAT 写事务提交失败，丢弃未提交的虚拟 metadata"
                    );
                    self.pending_tx = PendingTransaction::new(self.next_tx_id);
                    self.next_tx_id += 1;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        if !self.pending_tx.writes().is_empty() {
            let tx = self.pending_tx.clone();
            match self.try_commit_closed_transaction(&tx) {
                Ok(mutations) => {
                    if !mutations.is_empty() {
                        self.apply_metadata_overlays_from_transaction(&tx);
                    }
                    self.pending_tx = PendingTransaction::new(self.next_tx_id);
                    self.next_tx_id += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        tx_id = tx.id(),
                        error = %e,
                        "exFAT flush 提交失败，丢弃未提交的虚拟 metadata"
                    );
                    self.pending_tx = PendingTransaction::new(self.next_tx_id);
                    self.next_tx_id += 1;
                    return Err(e);
                }
            }
        }
        self.committer.sync_mount_root()
    }

    pub fn shutdown(&mut self) -> Result<(), std::io::Error> {
        self.flush()
    }

    pub fn validate_consistency(&self) -> Result<(), std::io::Error> {
        for record in self.directory_store.records() {
            if self.index.node(record.node_id).is_none() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("directory {} points to missing VFS node", record.virtual_path),
                ));
            }
            for cluster in &record.clusters {
                if !self.bitmap.is_allocated(*cluster) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("directory {} cluster {} is not allocated", record.virtual_path, cluster),
                    ));
                }
                let owner = self.sector_owner(self.cluster_to_sector(*cluster));
                let valid_owner = if record.virtual_path == "/" {
                    matches!(owner, SectorOwner::RootDirectory)
                } else {
                    matches!(owner, SectorOwner::DirectoryData { node_id } if node_id == record.node_id.0)
                };
                if !valid_owner {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("directory {} cluster {} has invalid owner", record.virtual_path, cluster),
                    ));
                }
            }
        }

        for cluster in self.bitmap.allocated_clusters() {
            let owner = self.sector_owner(self.cluster_to_sector(cluster));
            if matches!(owner, SectorOwner::FreeCluster { .. }) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("allocated cluster {} still marked free", cluster),
                ));
            }
        }

        for (start, _, _) in self.sector_owners.explicit_ranges() {
            match self.sector_owner(start) {
                SectorOwner::DirectoryData { node_id }
                | SectorOwner::FileData { node_id, .. }
                | SectorOwner::AllocatedZero { node_id, .. } => {
                    if self.index.node(NodeId(node_id)).is_none() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("sector {start} references missing node {node_id}"),
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(())
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
        self.validate_consistency()?;
        Ok(())
    }

    fn apply_metadata_overlays_from_transaction(&mut self, tx: &PendingTransaction) {
        for write in tx.writes() {
            match write {
                TransactionWrite::Fat { sector, data }
                | TransactionWrite::Bitmap { sector, data }
                | TransactionWrite::Directory { sector, data, .. } => {
                    self.metadata_overlays.insert(*sector, padded_sector(data));
                }
                TransactionWrite::FileData { .. } | TransactionWrite::FreeCluster { .. } => {}
            }
        }
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

fn mark_file_range(
    sector_owners: &mut SectorOwnerMap,
    info: FileDataRangeInfo,
    node_id: u64,
) -> Result<(), std::io::Error> {
    sector_owners.mark_range(
        info.start_sector,
        info.sector_count,
        SectorOwner::FileDataRange {
            node_id,
            file_offset: info.offset,
            byte_len: info.byte_len,
        },
    )
}

fn normalize_path(path: &Path) -> PathBuf {
    path.components().collect()
}

fn sync_vfs_clusters_from_volume(
    index: &mut VfsIndex,
    volume: &VirtualVolume,
) -> Result<(), std::io::Error> {
    for (path, clusters) in volume.directory_cluster_entries() {
        index.set_first_cluster(&path, clusters.first().copied())?;
    }

    let real_to_path = index
        .iter_nodes()
        .map(|node| (normalize_path(&node.real_path), node.virtual_path.clone()))
        .collect::<HashMap<_, _>>();
    for info in volume.file_data_range_entries() {
        let Some(first_cluster) = volume.layout().sector_to_cluster(info.start_sector) else {
            continue;
        };
        if let Some(path) = real_to_path.get(&normalize_path(&info.real_path)) {
            index.set_first_cluster(path, Some(first_cluster))?;
        }
    }
    Ok(())
}

fn padded_sector(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; SECTOR_SIZE as usize];
    let copy_len = data.len().min(SECTOR_SIZE as usize);
    out[..copy_len].copy_from_slice(&data[..copy_len]);
    out
}

fn read_placeholder_slice(file_offset: u64, sector_offset: usize, take: usize) -> Vec<u8> {
    let placeholder = blocked_placeholder_bytes();
    let Some(start_u64) = file_offset.checked_add(sector_offset as u64) else {
        return vec![0u8; take];
    };
    let Ok(start) = usize::try_from(start_u64) else {
        return vec![0u8; take];
    };
    let mut out = Vec::with_capacity(take);
    if start < placeholder.len() {
        let end = (start + take).min(placeholder.len());
        out.extend_from_slice(&placeholder[start..end]);
    }
    out.resize(take, 0);
    out
}

fn mutation_name(mutation: &FsMutation) -> &'static str {
    match mutation {
        FsMutation::CreateDir { .. } => "create_dir",
        FsMutation::CreateFile { .. } => "create_file",
        FsMutation::WriteFile { .. } => "write_file",
        FsMutation::Truncate { .. } => "truncate",
        FsMutation::Rename { .. } => "rename",
        FsMutation::Delete { .. } => "delete",
        FsMutation::RewriteFile { .. } => "rewrite_file",
    }
}

impl ExfatRuntimeState {
    fn check_mutation(&self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::WriteFile { virtual_path, .. }
            | FsMutation::RewriteFile { virtual_path, .. }
            | FsMutation::Truncate { virtual_path, .. }
            | FsMutation::Delete { virtual_path, .. } => {
                self.deny_if_blocked_placeholder(virtual_path, mutation_name(mutation))?;
            }
            FsMutation::Rename { from, to, .. } => {
                self.deny_if_blocked_placeholder(from, "rename_from")?;
                self.deny_if_blocked_placeholder(to, "rename_to")?;
            }
            FsMutation::CreateDir { .. } | FsMutation::CreateFile { .. } => {}
        }

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
            }),
        }
    }

    fn deny_if_blocked_placeholder(
        &self,
        virtual_path: &str,
        operation: &str,
    ) -> Result<(), std::io::Error> {
        if let Some(node) = self.lookup_path(virtual_path) {
            if node.is_blocked_placeholder() {
                tracing::warn!(
                    virtual_path = %node.virtual_path,
                    reason = node.blocked_reason().unwrap_or("unknown"),
                    operation,
                    "阻断占位文件禁止变更"
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "阻断占位文件禁止变更",
                ));
            }
        }
        Ok(())
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
                    self.mark_file_chain(id, chain, *size)?;
                }
            }
            FsMutation::Rename { from, to, .. } => {
                self.directory_store.rename_subtree(from, to);
            }
            FsMutation::Delete { virtual_path, .. } => {
                let removed_clusters = self.directory_store.remove_subtree(virtual_path);
                for cluster in removed_clusters {
                    self.bitmap.mark_free(cluster)?;
                    self.sector_owners.mark_range(
                        self.volume.layout().cluster_to_sector(cluster),
                        SECTORS_PER_CLUSTER as u64,
                        SectorOwner::FreeCluster { cluster },
                    )?;
                }
                self.clear_stale_node_owners()?;
            }
            FsMutation::RewriteFile {
                virtual_path,
                chain,
                size,
                ..
            } => {
                let id = self.index.lookup_path(virtual_path).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "rewritten file not in VFS")
                })?;
                self.clear_file_node_owners(id)?;
                if let Some(chain) = chain {
                    self.mark_file_chain(id, chain, *size)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn mark_file_chain(
        &mut self,
        id: NodeId,
        chain: &crate::vfs::mutation::ClusterChain,
        size: u64,
    ) -> Result<(), std::io::Error> {
        self.fat.set_chain(&chain.clusters)?;
        let mut remaining = size;
        let mut offset = 0_u64;
        for cluster in &chain.clusters {
            self.bitmap.mark_allocated(*cluster)?;
            for i in 0..SECTORS_PER_CLUSTER as u64 {
                let sector = self.volume.layout().cluster_to_sector(*cluster) + i;
                let owner = if remaining == 0 {
                    SectorOwner::AllocatedZero {
                        node_id: id.0,
                        file_offset: offset,
                    }
                } else {
                    let valid_bytes = remaining.min(crate::exfat::layout::SECTOR_SIZE as u64);
                    remaining = remaining.saturating_sub(valid_bytes);
                    SectorOwner::FileData {
                        node_id: id.0,
                        file_offset: offset,
                        valid_bytes: valid_bytes as u32,
                    }
                };
                self.sector_owners.mark_range(sector, 1, owner)?;
                offset += crate::exfat::layout::SECTOR_SIZE as u64;
            }
        }
        Ok(())
    }

    fn clear_stale_node_owners(&mut self) -> Result<(), std::io::Error> {
        for (start, len, owner) in self.sector_owners.explicit_ranges() {
            let stale_node = match owner {
                SectorOwner::DirectoryData { node_id }
                | SectorOwner::FileData { node_id, .. }
                | SectorOwner::FileDataRange { node_id, .. }
                | SectorOwner::AllocatedZero { node_id, .. } => {
                    self.index.node(NodeId(node_id)).is_none()
                }
                _ => false,
            };
            if !stale_node {
                continue;
            }
            let replacement = if let Some(cluster) = self.volume.layout().sector_to_cluster(start) {
                self.free_allocated_clusters_for_range(start, len)?;
                SectorOwner::FreeClusterRange {
                    first_cluster: cluster,
                    first_sector: start,
                    sectors_per_cluster: SECTORS_PER_CLUSTER as u64,
                }
            } else {
                SectorOwner::Reserved
            };
            self.sector_owners.mark_range(start, len, replacement)?;
        }
        Ok(())
    }

    fn clear_file_node_owners(&mut self, id: NodeId) -> Result<(), std::io::Error> {
        for (start, len, owner) in self.sector_owners.explicit_ranges() {
            let owned_by_file = match owner {
                SectorOwner::FileData { node_id, .. }
                | SectorOwner::FileDataRange { node_id, .. }
                | SectorOwner::AllocatedZero { node_id, .. } => node_id == id.0,
                _ => false,
            };
            if !owned_by_file {
                continue;
            }
            let replacement = if let Some(cluster) = self.volume.layout().sector_to_cluster(start) {
                self.free_allocated_clusters_for_range(start, len)?;
                SectorOwner::FreeClusterRange {
                    first_cluster: cluster,
                    first_sector: start,
                    sectors_per_cluster: SECTORS_PER_CLUSTER as u64,
                }
            } else {
                SectorOwner::Reserved
            };
            self.sector_owners.mark_range(start, len, replacement)?;
        }
        Ok(())
    }

    fn free_allocated_clusters_for_range(
        &mut self,
        start: u64,
        len: u64,
    ) -> Result<(), std::io::Error> {
        for sector_offset in (0..len).step_by(SECTORS_PER_CLUSTER as usize) {
            if let Some(cluster) = self.volume.layout().sector_to_cluster(start + sector_offset) {
                if self.bitmap.is_allocated(cluster) {
                    self.bitmap.mark_free(cluster)?;
                }
            }
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
