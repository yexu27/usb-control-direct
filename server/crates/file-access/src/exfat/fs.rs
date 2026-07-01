//! 受控虚拟 exFAT 文件系统 facade。

use std::collections::{BTreeSet, HashMap};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Mutex;

use tracing::debug;

use crate::exfat::allocator::ExfatAllocator;
use crate::exfat::diff::diff_directory_snapshots;
use crate::exfat::dir_snapshot::DirectorySnapshot;
use crate::exfat::layout::{SECTOR_SIZE, SECTORS_PER_CLUSTER};
use crate::exfat::volume::VirtualVolume;
use crate::policy::evaluate_access;
use crate::types::{AccessDecision, ControlledEntry, PolicySnapshot, SectorContent};
use crate::vfs::committer::RealFsCommitter;
use crate::vfs::index::node_to_controlled_entry;
use crate::vfs::journal::{FileMutation, WriteJournal};
use crate::vfs::mutation::{FileDataPatch, FsMutation, NodeKind};
use crate::vfs::operation_guard::{FsOperation, OperationGuard};
use crate::vfs::{NodeId, VfsIndex};

pub struct VirtualExfatFs {
    index: VfsIndex,
    snapshot: PolicySnapshot,
    volume: VirtualVolume,
    journal: Mutex<WriteJournal>,
    metadata_overlay: Mutex<HashMap<u64, Vec<u8>>>,
    dirty_metadata_sectors: Mutex<BTreeSet<u64>>,
    data_overlay: Mutex<HashMap<u64, Vec<u8>>>,
    runtime_directory_sector_paths: Mutex<HashMap<u64, String>>,
    runtime_directory_path_clusters: Mutex<HashMap<String, Vec<u32>>>,
    runtime_file_sector_paths: Mutex<HashMap<u64, RuntimeFileSector>>,
    committer: RealFsCommitter,
    readonly: bool,
}

#[derive(Debug, Clone)]
struct RuntimeFileSector {
    virtual_path: String,
    file_offset: u64,
    valid_bytes: u32,
}

impl VirtualExfatFs {
    pub fn build(
        mount_root: &Path,
        tree: &[ControlledEntry],
        snapshot: PolicySnapshot,
        source_size_bytes: u64,
    ) -> Result<Self, std::io::Error> {
        let readonly = snapshot.permission == 0;
        let index = VfsIndex::from_controlled_tree(mount_root, tree)?;
        let allocator = ExfatAllocator::build(&index, source_size_bytes)?;
        let volume = VirtualVolume::build_with_capacity(&tree, &snapshot, source_size_bytes);
        debug!(
            source_size_bytes,
            metadata_memory_bytes = allocator.estimated_memory_bytes(),
            "受控虚拟 exFAT 文件系统构建完成"
        );
        Ok(VirtualExfatFs {
            index,
            snapshot,
            volume,
            journal: Mutex::new(WriteJournal::new()),
            metadata_overlay: Mutex::new(HashMap::new()),
            dirty_metadata_sectors: Mutex::new(BTreeSet::new()),
            data_overlay: Mutex::new(HashMap::new()),
            runtime_directory_sector_paths: Mutex::new(HashMap::new()),
            runtime_directory_path_clusters: Mutex::new(HashMap::new()),
            runtime_file_sector_paths: Mutex::new(HashMap::new()),
            committer: RealFsCommitter::new(mount_root.to_path_buf()),
            readonly,
        })
    }

    pub fn total_sectors(&self) -> u64 {
        self.volume.total_sectors()
    }

    pub fn root_dir_offset_for_test(&self) -> u64 {
        self.volume.layout().cluster_to_sector(2) * SECTOR_SIZE as u64
    }

    pub fn cluster_offset_for_test(&self, cluster: u32) -> u64 {
        self.volume.layout().cluster_to_sector(cluster) * SECTOR_SIZE as u64
    }

    pub fn lookup_path(&self, path: &str) -> Option<NodeId> {
        self.index.lookup_path(path)
    }

    pub fn read_file(
        &self,
        node_id: NodeId,
        offset: u64,
        len: usize,
    ) -> Result<Vec<u8>, std::io::Error> {
        let node = self.index.node(node_id).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "virtual node not found")
        })?;
        if node.is_virus {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "病毒文件禁止访问",
            ));
        }

        let decision_entry = node_to_controlled_entry(node);
        if let AccessDecision::Deny(reason) = evaluate_access(&decision_entry, &self.snapshot) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                reason,
            ));
        }

        let mut file = std::fs::OpenOptions::new().read(true).open(&node.real_path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; len];
        let read_len = file.read(&mut buf)?;
        buf.truncate(read_len);
        Ok(buf)
    }

    pub fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut out = Vec::with_capacity(len);
        let mut current = offset;
        while out.len() < len {
            let sector = current / SECTOR_SIZE as u64;
            let sector_offset = (current % SECTOR_SIZE as u64) as usize;
            let take = (SECTOR_SIZE as usize - sector_offset).min(len - out.len());
            if let Some(data) = self.metadata_overlay.lock().unwrap().get(&sector) {
                out.extend_from_slice(&data[sector_offset..sector_offset + take]);
                current += take as u64;
                continue;
            }
            if let Some(data) = self.data_overlay.lock().unwrap().get(&sector) {
                out.extend_from_slice(&data[sector_offset..sector_offset + take]);
                current += take as u64;
                continue;
            }
            match self.volume.read_sector(sector) {
                SectorContent::Metadata(data) => {
                    out.extend_from_slice(&data[sector_offset..sector_offset + take]);
                }
                SectorContent::FileData {
                    real_path,
                    offset,
                    valid_bytes,
                    blocked,
                } => {
                    if blocked {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            "文件被策略阻断，禁止读取",
                        ));
                    }
                    let available = valid_bytes.saturating_sub(sector_offset as u32) as usize;
                    let read_len = take.min(available);
                    let mut file = std::fs::File::open(real_path)?;
                    file.seek(SeekFrom::Start(offset + sector_offset as u64))?;
                    let mut data = vec![0u8; read_len];
                    file.read_exact(&mut data)?;
                    out.extend_from_slice(&data);
                    if read_len < take {
                        out.resize(out.len() + take - read_len, 0);
                    }
                }
                SectorContent::Zero => {
                    if let Some(runtime) = self.runtime_file_sector_for_sector(sector) {
                        let path = self.committer.real_path_for_virtual(&runtime.virtual_path)?;
                        let mut file = std::fs::File::open(path)?;
                        file.seek(SeekFrom::Start(runtime.file_offset + sector_offset as u64))?;
                        let mut data = vec![0u8; take];
                        let read_len = file.read(&mut data)?;
                        out.extend_from_slice(&data[..read_len]);
                        if read_len < take {
                            out.resize(out.len() + take - read_len, 0);
                        }
                    } else {
                        out.resize(out.len() + take, 0);
                    }
                }
            }
            current += take as u64;
        }
        Ok(out)
    }

    pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        if self.readonly {
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
        for (i, chunk) in data.chunks(SECTOR_SIZE as usize).enumerate() {
            let sector = start_sector + i as u64;
            if sector == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "禁止修改 exFAT boot sector",
                ));
            }
            if self.directory_path_for_sector(sector)?.is_some() {
                let mut sector_data = match self.volume.read_sector(sector) {
                    SectorContent::Metadata(data) => data,
                    _ => vec![0u8; SECTOR_SIZE as usize],
                };
                sector_data[..chunk.len()].copy_from_slice(chunk);
                self.metadata_overlay
                    .lock()
                    .unwrap()
                    .insert(sector, sector_data);
                self.dirty_metadata_sectors.lock().unwrap().insert(sector);
                continue;
            }
            if let Some(runtime) = self.runtime_file_sector_for_sector(sector) {
                self.operation_guard().check(&FsOperation::WriteFile {
                    virtual_path: runtime.virtual_path.clone(),
                })?;
                let write_offset = runtime.file_offset;
                let write_len = chunk.len().min(runtime.valid_bytes as usize);
                self.write_file(&runtime.virtual_path, write_offset, &chunk[..write_len])?;
                continue;
            }
            match self.volume.read_sector(sector) {
                SectorContent::FileData {
                    real_path,
                    offset,
                    valid_bytes,
                    blocked,
                } => {
                    if blocked {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            "文件被策略阻断，禁止写入",
                        ));
                    }
                    let write_len = chunk.len().min(valid_bytes as usize);
                    self.write_real_file_sector(&real_path, offset, &chunk[..write_len])?;
                }
                SectorContent::Metadata(_) => {
                    let mut sector_data = match self.volume.read_sector(sector) {
                        SectorContent::Metadata(data) => data,
                        _ => vec![0u8; SECTOR_SIZE as usize],
                    };
                    sector_data[..chunk.len()].copy_from_slice(chunk);
                    self.metadata_overlay
                        .lock()
                        .unwrap()
                        .insert(sector, sector_data);
                    self.dirty_metadata_sectors.lock().unwrap().insert(sector);
                }
                SectorContent::Zero => {
                    let mut sector_data = vec![0u8; SECTOR_SIZE as usize];
                    sector_data[..chunk.len()].copy_from_slice(chunk);
                    self.data_overlay.lock().unwrap().insert(sector, sector_data);
                }
            }
        }
        Ok(())
    }

    pub fn create_file(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        self.operation_guard().check(&FsOperation::CreateFile {
            virtual_path: virtual_path.to_string(),
        })?;
        self.journal
            .lock()
            .unwrap()
            .record(FileMutation::CreateFile {
                virtual_path: virtual_path.to_string(),
            });
        Ok(())
    }

    pub fn create_dir(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        self.operation_guard().check(&FsOperation::CreateDir {
            virtual_path: virtual_path.to_string(),
        })?;
        self.journal
            .lock()
            .unwrap()
            .record(FileMutation::CreateDir {
                virtual_path: virtual_path.to_string(),
            });
        Ok(())
    }

    pub fn write_file(
        &self,
        virtual_path: &str,
        offset: u64,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        self.operation_guard().check(&FsOperation::WriteFile {
            virtual_path: virtual_path.to_string(),
        })?;
        self.journal.lock().unwrap().record(FileMutation::Write {
            virtual_path: virtual_path.to_string(),
            offset,
            data: data.to_vec(),
        });
        Ok(())
    }

    pub fn truncate(&self, virtual_path: &str, len: u64) -> Result<(), std::io::Error> {
        self.operation_guard().check(&FsOperation::Truncate {
            virtual_path: virtual_path.to_string(),
        })?;
        self.journal
            .lock()
            .unwrap()
            .record(FileMutation::Truncate {
                virtual_path: virtual_path.to_string(),
                len,
            });
        Ok(())
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<(), std::io::Error> {
        self.operation_guard().check(&FsOperation::Rename {
            from: from.to_string(),
            to: to.to_string(),
        })?;
        self.journal.lock().unwrap().record(FileMutation::Rename {
            from: from.to_string(),
            to: to.to_string(),
        });
        Ok(())
    }

    pub fn delete_file(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let is_virus = self
            .lookup_path(virtual_path)
            .and_then(|id| self.index.node(id))
            .map(|node| node.is_virus)
            .unwrap_or(false);
        self.operation_guard().check(&FsOperation::Delete {
            virtual_path: virtual_path.to_string(),
            is_virus,
        })?;
        self.journal
            .lock()
            .unwrap()
            .record(FileMutation::DeleteFile {
                virtual_path: virtual_path.to_string(),
            });
        Ok(())
    }

    pub fn flush(&self) -> Result<(), std::io::Error> {
        self.commit_overlay_mutations()?;
        self.journal.lock().unwrap().flush(&self.committer)?;
        self.dirty_metadata_sectors.lock().unwrap().clear();
        self.data_overlay.lock().unwrap().clear();
        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), std::io::Error> {
        self.flush()
    }

    fn operation_guard(&self) -> OperationGuard {
        OperationGuard::new(self.snapshot.clone())
    }

    fn write_real_file_sector(
        &self,
        real_path: &Path,
        offset: u64,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        let mut file = std::fs::OpenOptions::new().write(true).open(real_path)?;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(data)?;
        file.flush()?;
        Ok(())
    }

    fn commit_overlay_mutations(&self) -> Result<(), std::io::Error> {
        loop {
            let snapshots = self.collect_dirty_directory_snapshots()?;
            if snapshots.is_empty() {
                break;
            }
            let mut mutations = Vec::new();
            for (old, new) in snapshots {
                mutations.extend(diff_directory_snapshots(&old, &new)?);
            }
            for mutation in mutations {
                self.apply_mutation_with_policy(self.attach_data_patches(mutation)?)?;
            }
            self.promote_directory_data_overlay_to_metadata()?;
        }
        Ok(())
    }

    fn collect_dirty_directory_snapshots(
        &self,
    ) -> Result<Vec<(DirectorySnapshot, DirectorySnapshot)>, std::io::Error> {
        let dirty = {
            let mut dirty = self.dirty_metadata_sectors.lock().unwrap();
            std::mem::take(&mut *dirty).into_iter().collect::<Vec<_>>()
        };
        let mut paths = BTreeSet::new();
        for sector in dirty {
            if let Some(path) = self.directory_path_for_sector(sector)? {
                paths.insert(path);
            }
        }

        let mut snapshots = Vec::new();
        for path in paths {
            let original = self.read_directory_snapshot(&path, false)?;
            let overlaid = self.read_directory_snapshot(&path, true)?;
            snapshots.push((original, overlaid));
        }
        Ok(snapshots)
    }

    fn read_directory_snapshot(
        &self,
        path: &str,
        with_overlay: bool,
    ) -> Result<DirectorySnapshot, std::io::Error> {
        let clusters = self.directory_clusters_for_path(path)?;
        let mut data = Vec::new();
        let overlay = self.metadata_overlay.lock().unwrap();
        for cluster in clusters {
            let start_sector = self.volume.layout().cluster_to_sector(cluster);
            for i in 0..SECTORS_PER_CLUSTER as u64 {
                let sector = start_sector + i;
                if with_overlay {
                    if let Some(sector_data) = overlay.get(&sector) {
                        data.extend_from_slice(sector_data);
                        continue;
                    }
                }
                match self.volume.read_sector(sector) {
                    SectorContent::Metadata(sector_data) => data.extend_from_slice(&sector_data),
                    _ => data.extend(vec![0u8; SECTOR_SIZE as usize]),
                }
            }
        }
        DirectorySnapshot::parse(path, &data)
    }

    fn attach_data_patches(&self, mutation: FsMutation) -> Result<FsMutation, std::io::Error> {
        match mutation {
            FsMutation::CreateFile {
                parent,
                name,
                size,
                valid_data_len,
                chain,
                ..
            } => {
                let data_patches = if let Some(chain) = &chain {
                    if size == 0 {
                        Vec::new()
                    } else {
                        vec![FileDataPatch {
                            virtual_path: join_virtual_path(&parent, &name),
                            offset: 0,
                            data: self.collect_overlay_file_data(chain.first_cluster, size)?,
                        }]
                    }
                } else {
                    Vec::new()
                };
                Ok(FsMutation::CreateFile {
                    parent,
                    name,
                    size,
                    valid_data_len,
                    chain,
                    data_patches,
                })
            }
            other => Ok(other),
        }
    }

    fn apply_mutation_with_policy(&self, mutation: FsMutation) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::CreateFile {
                parent,
                name,
                chain,
                data_patches,
                size,
                ..
            } => {
                let virtual_path = join_virtual_path(&parent, &name);
                self.operation_guard()
                    .check(&FsOperation::CreateFile { virtual_path: virtual_path.clone() })?;
                if let Some(chain) = &chain {
                    self.register_runtime_file(&virtual_path, &chain.clusters, size);
                }
                let mut journal = self.journal.lock().unwrap();
                journal.record(FileMutation::CreateFile {
                    virtual_path: virtual_path.clone(),
                });
                for patch in data_patches {
                    journal.record(FileMutation::Write {
                        virtual_path: patch.virtual_path,
                        offset: patch.offset,
                        data: patch.data,
                    });
                }
                Ok(())
            }
            FsMutation::CreateDir { parent, name, chain } => {
                let virtual_path = join_virtual_path(&parent, &name);
                self.operation_guard()
                    .check(&FsOperation::CreateDir { virtual_path: virtual_path.clone() })?;
                if let Some(chain) = &chain {
                    self.register_runtime_directory(&virtual_path, &chain.clusters);
                }
                self.journal
                    .lock()
                    .unwrap()
                    .record(FileMutation::CreateDir { virtual_path });
                Ok(())
            }
            FsMutation::WriteFile {
                virtual_path,
                offset,
                data,
            } => {
                self.operation_guard()
                    .check(&FsOperation::WriteFile { virtual_path: virtual_path.clone() })?;
                self.journal
                    .lock()
                    .unwrap()
                    .record(FileMutation::Write { virtual_path, offset, data });
                Ok(())
            }
            FsMutation::Truncate { virtual_path, len } => {
                self.operation_guard()
                    .check(&FsOperation::Truncate { virtual_path: virtual_path.clone() })?;
                self.journal
                    .lock()
                    .unwrap()
                    .record(FileMutation::Truncate { virtual_path, len });
                Ok(())
            }
            FsMutation::Rename { from, to, .. } => {
                self.operation_guard()
                    .check(&FsOperation::Rename { from: from.clone(), to: to.clone() })?;
                self.journal
                    .lock()
                    .unwrap()
                    .record(FileMutation::Rename { from, to });
                Ok(())
            }
            FsMutation::Delete { virtual_path, kind } => {
                self.operation_guard().check(&FsOperation::Delete {
                    virtual_path: virtual_path.clone(),
                    is_virus: false,
                })?;
                let mutation = match kind {
                    NodeKind::File => FileMutation::DeleteFile { virtual_path },
                    NodeKind::Directory => FileMutation::DeleteDir { virtual_path },
                };
                self.journal.lock().unwrap().record(mutation);
                Ok(())
            }
        }
    }

    fn collect_overlay_file_data(
        &self,
        first_cluster: u32,
        len: u64,
    ) -> Result<Vec<u8>, std::io::Error> {
        let start_sector = self.volume.layout().cluster_to_sector(first_cluster);
        let total_sectors = len.div_ceil(SECTOR_SIZE as u64);
        let mut data = Vec::with_capacity(len as usize);
        let overlay = self.data_overlay.lock().unwrap();
        for i in 0..total_sectors {
            let sector = start_sector + i;
            if let Some(sector_data) = overlay.get(&sector) {
                data.extend_from_slice(sector_data);
            } else {
                data.extend(vec![0u8; SECTOR_SIZE as usize]);
            }
        }
        data.truncate(len as usize);
        Ok(data)
    }

    fn directory_path_for_sector(
        &self,
        sector: u64,
    ) -> Result<Option<String>, std::io::Error> {
        if let Some(path) = self.volume.directory_path_for_sector(sector)? {
            return Ok(Some(path));
        }
        Ok(self
            .runtime_directory_sector_paths
            .lock()
            .unwrap()
            .get(&sector)
            .cloned())
    }

    fn directory_clusters_for_path(&self, path: &str) -> Result<Vec<u32>, std::io::Error> {
        if let Some(clusters) = self.volume.directory_clusters_for_path(path) {
            return Ok(clusters.to_vec());
        }
        self.runtime_directory_path_clusters
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "directory path not mapped")
            })
    }

    fn register_runtime_directory(&self, path: &str, clusters: &[u32]) {
        if clusters.is_empty() {
            return;
        }
        self.runtime_directory_path_clusters
            .lock()
            .unwrap()
            .insert(path.to_string(), clusters.to_vec());
        let mut sector_paths = self.runtime_directory_sector_paths.lock().unwrap();
        for cluster in clusters {
            let start = self.volume.layout().cluster_to_sector(*cluster);
            for i in 0..SECTORS_PER_CLUSTER as u64 {
                sector_paths.insert(start + i, path.to_string());
            }
        }
    }

    fn register_runtime_file(&self, path: &str, clusters: &[u32], size: u64) {
        if clusters.is_empty() {
            return;
        }
        let mut sector_paths = self.runtime_file_sector_paths.lock().unwrap();
        for (cluster_idx, cluster) in clusters.iter().enumerate() {
            let start = self.volume.layout().cluster_to_sector(*cluster);
            for i in 0..SECTORS_PER_CLUSTER as u64 {
                let file_offset = cluster_idx as u64
                    * SECTORS_PER_CLUSTER as u64
                    * SECTOR_SIZE as u64
                    + i * SECTOR_SIZE as u64;
                let valid_bytes = size
                    .saturating_sub(file_offset)
                    .min(SECTOR_SIZE as u64) as u32;
                sector_paths.insert(
                    start + i,
                    RuntimeFileSector {
                        virtual_path: path.to_string(),
                        file_offset,
                        valid_bytes,
                    },
                );
            }
        }
    }

    fn runtime_file_sector_for_sector(&self, sector: u64) -> Option<RuntimeFileSector> {
        self.runtime_file_sector_paths
            .lock()
            .unwrap()
            .get(&sector)
            .cloned()
    }

    fn promote_directory_data_overlay_to_metadata(&self) -> Result<(), std::io::Error> {
        let sectors = self.data_overlay.lock().unwrap().keys().copied().collect::<Vec<_>>();
        let mut promoted = Vec::new();
        for sector in sectors {
            if self.directory_path_for_sector(sector)?.is_some() {
                promoted.push(sector);
            }
        }
        if promoted.is_empty() {
            return Ok(());
        }

        let mut data_overlay = self.data_overlay.lock().unwrap();
        let mut metadata_overlay = self.metadata_overlay.lock().unwrap();
        let mut dirty = self.dirty_metadata_sectors.lock().unwrap();
        for sector in promoted {
            if let Some(data) = data_overlay.remove(&sector) {
                metadata_overlay.insert(sector, data);
                dirty.insert(sector);
            }
        }
        Ok(())
    }
}

fn join_virtual_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", parent, name)
    }
}

impl crate::nbd::NbdBackend for VirtualExfatFs {
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        VirtualExfatFs::read_at(self, offset, len)
    }

    fn write_at(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        VirtualExfatFs::write_at(self, offset, data)
    }

    fn flush(&self) -> Result<(), std::io::Error> {
        VirtualExfatFs::flush(self)
    }
}
