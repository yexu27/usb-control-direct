//! Runtime exFAT state built from the controlled USB tree.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::block_backend::BlockWriteOutcome;
use crate::exfat::bitmap_state::BitmapState;
use crate::exfat::commit_pipeline::CommitPipeline;
use crate::exfat::directory_store::DirectoryStore;
use crate::exfat::fat_state::FatState;
use crate::exfat::layout::{
    BOOT_REGION_SECTORS, FIRST_CLUSTER, PARTITION_OFFSET_SECTORS, SECTORS_PER_CLUSTER, SECTOR_SIZE,
};
use crate::exfat::metadata_overlay::MetadataOverlay;
use crate::exfat::metadata_state::ExfatMetadataState;
use crate::exfat::policy_rejection::RecoverablePolicyRejection;
use crate::exfat::sector_owner::{SectorOwner, SectorOwnerMap};
use crate::exfat::transaction::{PendingTransaction, ResolveStatus};
use crate::exfat::transaction_resolver::TransactionResolver;
use crate::exfat::volume::{FileDataRangeInfo, VirtualVolume};
use crate::exfat::write_interpreter::WriteInterpreter;
use crate::types::{blocked_placeholder_bytes, ControlledEntry, PolicySnapshot};
use crate::vfs::committer::RealFsCommitter;
use crate::vfs::mutation::{ClusterChain, FsMutation, NodeKind};
use crate::vfs::node::VfsFileView;
use crate::vfs::VfsNodeKind;
use crate::vfs::{NodeId, VfsIndex, VfsNode};

#[derive(Debug, Clone)]
pub struct ExfatRuntimeState {
    index: VfsIndex,
    volume: VirtualVolume,
    metadata: ExfatMetadataState,
    metadata_overlay: MetadataOverlay,
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
            if matches!(
                owner,
                SectorOwner::FreeCluster { .. } | SectorOwner::Reserved
            ) {
                sector_owners.mark_range(sector, 1, SectorOwner::UpcaseTable)?;
            }
        }

        for info in volume.file_data_range_entries() {
            if let Some(node) = real_to_node.get(&normalize_path(&info.real_path)) {
                mark_file_range(&mut sector_owners, info, node.id.0)?;
            }
        }

        let mut metadata = ExfatMetadataState::new(fat, bitmap, directory_store, sector_owners);
        for info in volume.file_data_range_entries() {
            let Some(node) = real_to_node.get(&normalize_path(&info.real_path)) else {
                continue;
            };
            let Some(first_cluster) = volume.layout().sector_to_cluster(info.start_sector) else {
                continue;
            };
            let cluster_count = file_cluster_count(node.size);
            if cluster_count == 0 {
                continue;
            }
            let clusters = (0..cluster_count)
                .map(|offset| first_cluster + offset)
                .collect::<Vec<_>>();
            metadata.set_file_chain(
                &layout,
                node.id,
                &ClusterChain {
                    first_cluster,
                    clusters,
                },
                node.size,
            )?;
        }

        let state = Self {
            index,
            volume,
            metadata,
            metadata_overlay: MetadataOverlay::new(),
            snapshot,
            committer: RealFsCommitter::new(mount_root.to_path_buf()),
            pending_tx: PendingTransaction::new(1),
            next_tx_id: 2,
        };
        state.validate_consistency()?;
        Ok(state)
    }

    pub fn lookup_path(&self, path: &str) -> Option<&VfsNode> {
        self.index
            .lookup_path(path)
            .and_then(|id| self.index.node(id))
    }

    pub fn lookup_node_id(&self, path: &str) -> Option<NodeId> {
        self.index.lookup_path(path)
    }

    pub fn node(&self, node_id: NodeId) -> Option<&VfsNode> {
        self.index.node(node_id)
    }

    pub fn directory_store(&self) -> &DirectoryStore {
        self.metadata.directory_store()
    }

    pub fn sector_owner(&self, sector: u64) -> SectorOwner {
        self.metadata.owner_of(sector)
    }

    pub fn total_sectors(&self) -> u64 {
        self.volume.total_sectors()
    }

    pub fn cluster_to_sector(&self, cluster: u32) -> u64 {
        self.volume.layout().cluster_to_sector(cluster)
    }

    pub(crate) fn cluster_size(&self) -> u32 {
        self.volume.layout().cluster_size()
    }

    pub(crate) fn metadata_chain_from(
        &self,
        first_cluster: u32,
    ) -> Result<Vec<u32>, std::io::Error> {
        self.metadata.chain_from(first_cluster)
    }

    fn file_node(&self, node_id: u64) -> Result<&VfsNode, std::io::Error> {
        self.index.node(NodeId(node_id)).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file data owner not found in VFS",
            )
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
            if let Some(data) = self.metadata_overlay.read_sector(sector) {
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
        let Some(clusters) = self.metadata.directory_clusters(path) else {
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
                if let Some(overlay) = self.metadata_overlay.read_sector(sector) {
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

    pub fn write_at(
        &mut self,
        offset: u64,
        data: &[u8],
    ) -> Result<BlockWriteOutcome, std::io::Error> {
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
                    // Windows may update virtual volume status metadata; this is not a file mutation.
                    self.metadata_overlay
                        .apply_committed_sector(sector, chunk)?;
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
                    let virtual_path = node.virtual_path.clone();
                    if let Some(outcome) = self.commit_mutation_for_write_request(
                        self.pending_tx.id(),
                        FsMutation::WriteFile {
                            virtual_path,
                            offset: file_offset,
                            data: chunk[..write_len].to_vec(),
                        },
                    )? {
                        return Ok(outcome);
                    }
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
                    let virtual_path = node.virtual_path.clone();
                    if let Some(outcome) = self.commit_mutation_for_write_request(
                        self.pending_tx.id(),
                        FsMutation::WriteFile {
                            virtual_path,
                            offset: file_offset,
                            data: chunk[..write_len].to_vec(),
                        },
                    )? {
                        return Ok(outcome);
                    }
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
            match self.resolve_pending_transaction(&tx) {
                Ok(ResolveStatus::Complete(resolved)) => {
                    if !resolved.mutations.is_empty() {
                        for mutation in resolved.mutations {
                            if let Some(outcome) =
                                self.commit_mutation_for_write_request(tx.id(), mutation)?
                            {
                                return Ok(outcome);
                            }
                        }
                        self.reset_pending_transaction();
                    } else {
                        self.pending_tx.retain_deferred_data_writes();
                        if self.pending_tx.is_empty() {
                            self.reset_pending_transaction();
                        }
                    }
                }
                Ok(ResolveStatus::Incomplete(_)) => {
                    self.pending_tx = tx;
                }
                Ok(ResolveStatus::Invalid(err)) => {
                    tracing::warn!(
                        tx_id = tx.id(),
                        error = ?err,
                        "exFAT 写事务无效，丢弃未提交的虚拟 metadata"
                    );
                    if err.is_recoverable_policy_rejection() {
                        let reason = format!("{err:?}");
                        self.reset_pending_transaction_after_policy_rejection(tx.id());
                        tracing::warn!(
                            tx_id = tx.id(),
                            reason = %reason,
                            "blocked placeholder 写事务被解析层拒绝，已恢复 canonical metadata 并吸收块写入"
                        );
                        return Ok(BlockWriteOutcome::PolicyRejectedAndRestored { reason });
                    } else {
                        self.reset_pending_transaction();
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("{err:?}"),
                    ));
                }
                Err(e) => {
                    tracing::warn!(
                        tx_id = tx.id(),
                        error = %e,
                        "exFAT 写事务提交失败，丢弃未提交的虚拟 metadata"
                    );
                    self.reset_pending_transaction();
                    return Err(e);
                }
            }
        }
        Ok(BlockWriteOutcome::Committed)
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        if !self.pending_tx.writes().is_empty() {
            let tx = self.pending_tx.clone();
            match self.resolve_pending_transaction(&tx) {
                Ok(ResolveStatus::Complete(resolved)) => {
                    if !resolved.mutations.is_empty() {
                        for mutation in resolved.mutations {
                            if self.commit_mutation_for_flush(tx.id(), mutation)? {
                                return self.committer.sync_mount_root();
                            }
                        }
                    }
                    self.reset_pending_transaction();
                }
                Ok(ResolveStatus::Incomplete(_)) => {
                    self.reset_pending_transaction();
                }
                Ok(ResolveStatus::Invalid(err)) => {
                    tracing::warn!(
                        tx_id = tx.id(),
                        error = ?err,
                        "exFAT flush 事务无效，丢弃未提交的虚拟 metadata"
                    );
                    if err.is_recoverable_policy_rejection() {
                        self.reset_pending_transaction_after_policy_rejection(tx.id());
                        tracing::warn!(
                            tx_id = tx.id(),
                            error = ?err,
                            "blocked placeholder flush 事务被解析层拒绝，已恢复 canonical metadata 并吸收 flush"
                        );
                        return Ok(());
                    } else {
                        self.reset_pending_transaction();
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("{err:?}"),
                    ));
                }
                Err(e) => {
                    tracing::warn!(
                        tx_id = tx.id(),
                        error = %e,
                        "exFAT flush 提交失败，丢弃未提交的虚拟 metadata"
                    );
                    self.reset_pending_transaction();
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
        self.metadata.validate(&self.index, self.volume.layout())?;
        for (start, _, _) in self.metadata.explicit_ranges() {
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

    fn resolve_pending_transaction(
        &self,
        tx: &PendingTransaction,
    ) -> Result<ResolveStatus, std::io::Error> {
        TransactionResolver::new().resolve_closed(tx, self)
    }

    fn reset_pending_transaction(&mut self) {
        self.pending_tx = PendingTransaction::new(self.next_tx_id);
        self.next_tx_id += 1;
    }

    fn reset_pending_transaction_after_policy_rejection(&mut self, tx_id: u64) {
        tracing::warn!(
            tx_id,
            "blocked placeholder 写事务被策略拒绝，已隔离失败事务，后续写入将从新事务开始"
        );
        self.reset_pending_transaction();
    }

    fn reset_pending_transaction_after_commit_error(&mut self, tx_id: u64, err: &std::io::Error) {
        tracing::warn!(
            tx_id,
            error = %err,
            "exFAT 写事务提交失败，已隔离失败事务，后续写入将从新事务开始"
        );
        self.reset_pending_transaction();
    }

    fn commit_mutation_for_write_request(
        &mut self,
        tx_id: u64,
        mutation: FsMutation,
    ) -> Result<Option<BlockWriteOutcome>, std::io::Error> {
        match self.commit_resolved_mutation(mutation) {
            Ok(()) => Ok(None),
            Err(e) => {
                if let Some(rejection) = RecoverablePolicyRejection::from_io_error(&e) {
                    let reason = rejection.to_outcome_reason();
                    self.reset_pending_transaction_after_commit_error(tx_id, &e);
                    tracing::warn!(
                        tx_id,
                        reason = %reason,
                        "blocked placeholder 写事务被策略拒绝，已恢复 canonical metadata 并吸收块写入"
                    );
                    return Ok(Some(BlockWriteOutcome::PolicyRejectedAndRestored {
                        reason,
                    }));
                }
                self.reset_pending_transaction_after_commit_error(tx_id, &e);
                Err(e)
            }
        }
    }

    fn commit_mutation_for_flush(
        &mut self,
        tx_id: u64,
        mutation: FsMutation,
    ) -> Result<bool, std::io::Error> {
        match self.commit_resolved_mutation(mutation) {
            Ok(()) => Ok(false),
            Err(e) => {
                if let Some(rejection) = RecoverablePolicyRejection::from_io_error(&e) {
                    let reason = rejection.to_outcome_reason();
                    self.reset_pending_transaction_after_commit_error(tx_id, &e);
                    tracing::warn!(
                        tx_id,
                        reason = %reason,
                        "blocked placeholder flush 事务被策略拒绝，已恢复 canonical metadata 并吸收 flush"
                    );
                    return Ok(true);
                }
                self.reset_pending_transaction_after_commit_error(tx_id, &e);
                Err(e)
            }
        }
    }

    fn commit_resolved_mutation(&mut self, mutation: FsMutation) -> Result<(), std::io::Error> {
        CommitPipeline::new(
            &mut self.index,
            &mut self.metadata,
            &mut self.metadata_overlay,
            &self.volume,
            &self.committer,
            self.snapshot.clone(),
        )
        .commit(mutation)
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

fn file_cluster_count(file_size: u64) -> u32 {
    file_size.div_ceil((SECTOR_SIZE * SECTORS_PER_CLUSTER) as u64) as u32
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
