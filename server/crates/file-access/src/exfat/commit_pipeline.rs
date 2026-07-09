//! exFAT filesystem mutation commit pipeline.

use crate::exfat::layout::SECTORS_PER_CLUSTER;
use crate::exfat::metadata_overlay::MetadataOverlay;
use crate::exfat::metadata_renderer::MetadataRenderer;
use crate::exfat::metadata_state::ExfatMetadataState;
use crate::exfat::policy_rejection::RecoverablePolicyRejection;
use crate::exfat::sector_owner::SectorOwner;
use crate::exfat::volume::VirtualVolume;
use crate::types::PolicySnapshot;
use crate::vfs::committer::RealFsCommitter;
use crate::vfs::mutation::{FsMutation, NodeKind};
use crate::vfs::operation_guard::{FsOperation, OperationGuard};
use crate::vfs::{NodeId, VfsIndex};

pub(crate) struct CommitPipeline<'a> {
    index: &'a mut VfsIndex,
    metadata: &'a mut ExfatMetadataState,
    metadata_overlay: &'a mut MetadataOverlay,
    volume: &'a VirtualVolume,
    committer: &'a RealFsCommitter,
    snapshot: PolicySnapshot,
}

impl<'a> CommitPipeline<'a> {
    pub(crate) fn new(
        index: &'a mut VfsIndex,
        metadata: &'a mut ExfatMetadataState,
        metadata_overlay: &'a mut MetadataOverlay,
        volume: &'a VirtualVolume,
        committer: &'a RealFsCommitter,
        snapshot: PolicySnapshot,
    ) -> Self {
        Self {
            index,
            metadata,
            metadata_overlay,
            volume,
            committer,
            snapshot,
        }
    }

    pub(crate) fn commit(&mut self, mutation: FsMutation) -> Result<(), std::io::Error> {
        self.check_mutation(&mutation)?;
        self.commit_real_fs(&mutation).map_err(|err| {
            std::io::Error::new(
                err.kind(),
                format!("exFAT real filesystem commit failed; session must fail-close: {err}"),
            )
        })?;
        self.index.apply_mutation(&mutation)?;
        self.refresh_metadata_after_mutation(&mutation)?;
        self.validate_metadata()?;
        self.apply_committed_metadata_for_mutation(&mutation)
    }

    fn check_mutation(&self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::WriteFile { virtual_path, .. }
            | FsMutation::RewriteFile { virtual_path, .. }
            | FsMutation::Truncate { virtual_path, .. } => {
                self.deny_modify_blocked_placeholder(virtual_path, mutation_name(mutation))?;
            }
            FsMutation::Rename { from, .. } => {
                self.deny_modify_blocked_placeholder(from, "rename_from")?;
            }
            FsMutation::CreateDir { .. }
            | FsMutation::CreateFile { .. }
            | FsMutation::Delete { .. } => {}
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
            | FsMutation::RewriteFile { virtual_path, .. } => {
                guard.check(&FsOperation::WriteFile {
                    virtual_path: virtual_path.clone(),
                })
            }
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

    fn deny_modify_blocked_placeholder(
        &self,
        virtual_path: &str,
        operation: &str,
    ) -> Result<(), std::io::Error> {
        if let Some(id) = self.index.lookup_path(virtual_path) {
            if let Some(node) = self.index.node(id) {
                if node.is_blocked_placeholder() {
                    let reason = node.blocked_reason().unwrap_or("unknown").to_string();
                    tracing::warn!(
                        virtual_path = %node.virtual_path,
                        reason = %reason,
                        operation,
                        "策略命中文件禁止修改"
                    );
                    return Err(RecoverablePolicyRejection::blocked_placeholder(
                        node.virtual_path.clone(),
                        operation,
                        reason,
                    )
                    .into_io_error());
                }
            }
        }
        Ok(())
    }

    fn commit_real_fs(&self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::CreateDir { parent, name, .. } => {
                self.committer.create_dir(&join_virtual_path(parent, name))
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
                Ok(())
            }
            FsMutation::WriteFile {
                virtual_path,
                offset,
                data,
            } => {
                self.committer.write_at(virtual_path, *offset, data)?;
                self.committer.flush_file(virtual_path)
            }
            FsMutation::Truncate { virtual_path, len } => {
                self.committer.truncate(virtual_path, *len)?;
                self.committer.flush_file(virtual_path)
            }
            FsMutation::Rename { from, to, .. } => self.committer.rename(from, to),
            FsMutation::Delete { virtual_path, kind } => {
                let real_path = self
                    .index
                    .lookup_path(virtual_path)
                    .and_then(|id| self.index.node(id))
                    .map(|node| node.real_path.clone())
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("delete target not found in VFS: {virtual_path}"),
                        )
                    })?;
                match kind {
                    NodeKind::File => self.committer.delete_file_at_real_path(&real_path),
                    NodeKind::Directory => self.committer.delete_dir_at_real_path(&real_path),
                }
            }
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
                self.committer.flush_file(virtual_path)
            }
        }
    }

    fn refresh_metadata_after_mutation(
        &mut self,
        mutation: &FsMutation,
    ) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::CreateDir {
                parent,
                name,
                chain,
            } => {
                let path = join_virtual_path(parent, name);
                let id = self.index.lookup_path(&path).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "created dir not in VFS")
                })?;
                if let Some(chain) = chain {
                    self.metadata.set_directory_chain(
                        self.volume.layout(),
                        id,
                        path,
                        chain.clusters.clone(),
                    )?;
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
                    self.metadata
                        .mark_file_chain(self.volume.layout(), id, chain, *size)?;
                }
            }
            FsMutation::Rename { from, to, .. } => {
                self.metadata.rename_subtree(from, to);
            }
            FsMutation::Delete { virtual_path, .. } => {
                self.metadata
                    .remove_subtree(self.volume.layout(), virtual_path)?;
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
                    self.metadata
                        .mark_file_chain(self.volume.layout(), id, chain, *size)?;
                }
            }
            FsMutation::WriteFile { .. } | FsMutation::Truncate { .. } => {}
        }
        Ok(())
    }

    fn validate_metadata(&self) -> Result<(), std::io::Error> {
        self.metadata.validate(self.index, self.volume.layout())?;
        for (start, _, _) in self.metadata.explicit_ranges() {
            match self.metadata.owner_of(start) {
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

    fn clear_stale_node_owners(&mut self) -> Result<(), std::io::Error> {
        for (start, len, owner) in self.metadata.explicit_ranges() {
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
            self.metadata.mark_range(start, len, replacement)?;
        }
        Ok(())
    }

    fn clear_file_node_owners(&mut self, id: NodeId) -> Result<(), std::io::Error> {
        for (start, len, owner) in self.metadata.explicit_ranges() {
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
            self.metadata.mark_range(start, len, replacement)?;
        }
        Ok(())
    }

    fn free_allocated_clusters_for_range(
        &mut self,
        start: u64,
        len: u64,
    ) -> Result<(), std::io::Error> {
        for sector_offset in (0..len).step_by(SECTORS_PER_CLUSTER as usize) {
            if let Some(cluster) = self
                .volume
                .layout()
                .sector_to_cluster(start + sector_offset)
            {
                if self.metadata.is_allocated(cluster) {
                    self.metadata
                        .mark_cluster_free(self.volume.layout(), cluster)?;
                }
            }
        }
        Ok(())
    }

    fn apply_committed_metadata_for_mutation(
        &mut self,
        mutation: &FsMutation,
    ) -> Result<(), std::io::Error> {
        let renderer = MetadataRenderer;
        let mut updates = renderer.render_fat_and_bitmap(self.metadata, self.volume.layout());
        match mutation {
            FsMutation::CreateDir { parent, .. } | FsMutation::CreateFile { parent, .. } => {
                updates.extend(renderer.render_directory(
                    parent,
                    self.index,
                    self.metadata,
                    self.volume.layout(),
                )?);
            }
            FsMutation::WriteFile { .. } => {}
            FsMutation::Truncate { virtual_path, .. }
            | FsMutation::RewriteFile { virtual_path, .. }
            | FsMutation::Delete { virtual_path, .. } => {
                if let Some(parent) = parent_path(virtual_path) {
                    updates.extend(renderer.render_directory(
                        &parent,
                        self.index,
                        self.metadata,
                        self.volume.layout(),
                    )?);
                }
            }
            FsMutation::Rename { from, to, .. } => {
                if let Some(parent) = parent_path(from) {
                    updates.extend(renderer.render_directory(
                        &parent,
                        self.index,
                        self.metadata,
                        self.volume.layout(),
                    )?);
                }
                if let Some(parent) = parent_path(to) {
                    updates.extend(renderer.render_directory(
                        &parent,
                        self.index,
                        self.metadata,
                        self.volume.layout(),
                    )?);
                }
            }
        }
        let updates = renderer.merge_updates(updates);
        self.metadata_overlay.apply_committed(&updates)
    }
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

fn join_virtual_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}

fn parent_path(path: &str) -> Option<String> {
    if path == "/" {
        return None;
    }
    let trimmed = path.trim_end_matches('/');
    let (parent, _) = trimmed.rsplit_once('/')?;
    Some(if parent.is_empty() {
        "/".to_string()
    } else {
        parent.to_string()
    })
}
