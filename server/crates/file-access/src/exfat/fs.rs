//! 受控虚拟 exFAT 文件系统 facade。

use std::io::{Read, Seek, SeekFrom, Write};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use tracing::debug;

use crate::exfat::allocator::ExfatAllocator;
use crate::exfat::directory_parser::parse_entry_sets;
use crate::exfat::layout::{SECTOR_SIZE, SECTORS_PER_CLUSTER};
use crate::exfat::volume::VirtualVolume;
use crate::policy::evaluate_access;
use crate::types::{AccessDecision, ControlledEntry, PolicySnapshot, SectorContent};
use crate::vfs::committer::RealFsCommitter;
use crate::vfs::index::node_to_controlled_entry;
use crate::vfs::journal::{FileMutation, WriteJournal};
use crate::vfs::operation_guard::{FsOperation, OperationGuard};
use crate::vfs::{NodeId, VfsIndex};

pub struct VirtualExfatFs {
    index: VfsIndex,
    snapshot: PolicySnapshot,
    volume: VirtualVolume,
    journal: Mutex<WriteJournal>,
    metadata_overlay: Mutex<HashMap<u64, Vec<u8>>>,
    data_overlay: Mutex<HashMap<u64, Vec<u8>>>,
    committer: RealFsCommitter,
    readonly: bool,
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
            data_overlay: Mutex::new(HashMap::new()),
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
                SectorContent::Zero => out.resize(out.len() + take, 0),
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
        self.commit_overlay_creates()?;
        self.journal.lock().unwrap().flush(&self.committer)
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

    fn commit_overlay_creates(&self) -> Result<(), std::io::Error> {
        let root_start = self.volume.layout().cluster_to_sector(2);
        let mut root_data = Vec::with_capacity(SECTS_PER_CLUSTER_BYTES);
        for i in 0..SECTORS_PER_CLUSTER as u64 {
            let sector = root_start + i;
            if let Some(data) = self.metadata_overlay.lock().unwrap().get(&sector) {
                root_data.extend_from_slice(data);
                continue;
            }
            match self.volume.read_sector(sector) {
                SectorContent::Metadata(data) => root_data.extend_from_slice(&data),
                _ => root_data.extend(vec![0u8; SECTOR_SIZE as usize]),
            }
        }

        let entries = parse_entry_sets(&root_data)?;
        for entry in entries {
            if entry.name.is_empty() || entry.is_deleted || entry.data_length == 0 {
                continue;
            }
            let virtual_path = format!("/{}", entry.name);
            let real_path = self.index.mount_root().join(&entry.name);
            if real_path.exists() {
                continue;
            }

            if entry.is_dir {
                self.create_dir(&virtual_path)?;
                continue;
            }

            self.operation_guard().check(&FsOperation::CreateFile {
                virtual_path: virtual_path.clone(),
            })?;
            let data = self.collect_overlay_file_data(entry.first_cluster, entry.data_length)?;
            self.journal
                .lock()
                .unwrap()
                .record(FileMutation::CreateFile {
                    virtual_path: virtual_path.clone(),
                });
            self.journal.lock().unwrap().record(FileMutation::Write {
                virtual_path,
                offset: 0,
                data,
            });
        }
        Ok(())
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
}

const SECTS_PER_CLUSTER_BYTES: usize = SECTORS_PER_CLUSTER as usize * SECTOR_SIZE as usize;

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
