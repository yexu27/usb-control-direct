//! 受控虚拟 exFAT 文件系统 facade。

use std::path::Path;
use std::sync::Mutex;

use tracing::debug;

use crate::exfat::layout::SECTOR_SIZE;
use crate::exfat::runtime_state::ExfatRuntimeState;
use crate::types::{ControlledEntry, PolicySnapshot};
use crate::vfs::mutation::{FsMutation, NodeKind};
use crate::vfs::{NodeId, VfsNodeKind};

pub struct VirtualExfatFs {
    runtime: Mutex<ExfatRuntimeState>,
}

impl VirtualExfatFs {
    pub fn build(
        mount_root: &Path,
        tree: &[ControlledEntry],
        snapshot: PolicySnapshot,
        source_size_bytes: u64,
    ) -> Result<Self, std::io::Error> {
        let runtime =
            ExfatRuntimeState::from_controlled_tree(mount_root, tree, snapshot, source_size_bytes)?;
        debug!(
            source_size_bytes,
            total_sectors = runtime.total_sectors(),
            "受控虚拟 exFAT 文件系统构建完成"
        );
        Ok(Self {
            runtime: Mutex::new(runtime),
        })
    }

    pub fn total_sectors(&self) -> u64 {
        self.runtime.lock().unwrap().total_sectors()
    }

    pub fn root_dir_offset_for_test(&self) -> u64 {
        self.cluster_offset_for_test(2)
    }

    pub fn cluster_offset_for_test(&self, cluster: u32) -> u64 {
        self.runtime.lock().unwrap().cluster_to_sector(cluster) * SECTOR_SIZE as u64
    }

    pub fn lookup_path(&self, path: &str) -> Option<NodeId> {
        self.runtime.lock().unwrap().lookup_node_id(path)
    }

    pub fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        self.runtime.lock().unwrap().read_at(offset, len)
    }

    pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        self.runtime.lock().unwrap().write_at(offset, data)
    }

    pub fn create_file(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let (parent, name) = split_virtual_path(virtual_path)?;
        self.runtime.lock().unwrap().commit_mutation(FsMutation::CreateFile {
            parent,
            name,
            size: 0,
            valid_data_len: 0,
            chain: None,
            data_patches: Vec::new(),
        })
    }

    pub fn create_dir(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let (parent, name) = split_virtual_path(virtual_path)?;
        self.runtime.lock().unwrap().commit_mutation(FsMutation::CreateDir {
            parent,
            name,
            chain: None,
        })
    }

    pub fn write_file(
        &self,
        virtual_path: &str,
        offset: u64,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        self.runtime.lock().unwrap().commit_mutation(FsMutation::WriteFile {
            virtual_path: virtual_path.to_string(),
            offset,
            data: data.to_vec(),
        })
    }

    pub fn truncate(&self, virtual_path: &str, len: u64) -> Result<(), std::io::Error> {
        self.runtime.lock().unwrap().commit_mutation(FsMutation::Truncate {
            virtual_path: virtual_path.to_string(),
            len,
        })
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<(), std::io::Error> {
        let kind = {
            let runtime = self.runtime.lock().unwrap();
            let node = runtime.lookup_path(from).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "rename source not found")
            })?;
            node_kind(node.kind)
        };
        self.runtime.lock().unwrap().commit_mutation(FsMutation::Rename {
            from: from.to_string(),
            to: to.to_string(),
            kind,
        })
    }

    pub fn delete_file(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let kind = {
            let runtime = self.runtime.lock().unwrap();
            let node = runtime.lookup_path(virtual_path).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "delete target not found")
            })?;
            node_kind(node.kind)
        };
        self.runtime.lock().unwrap().commit_mutation(FsMutation::Delete {
            virtual_path: virtual_path.to_string(),
            kind,
        })
    }

    pub fn flush(&self) -> Result<(), std::io::Error> {
        self.runtime.lock().unwrap().flush()
    }

    pub fn shutdown(&self) -> Result<(), std::io::Error> {
        self.runtime.lock().unwrap().shutdown()
    }
}

fn node_kind(kind: VfsNodeKind) -> NodeKind {
    match kind {
        VfsNodeKind::File => NodeKind::File,
        VfsNodeKind::Directory => NodeKind::Directory,
    }
}

fn split_virtual_path(path: &str) -> Result<(String, String), std::io::Error> {
    if !path.starts_with('/') || path == "/" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "virtual path must be an absolute non-root path",
        ));
    }
    let trimmed = path.trim_end_matches('/');
    let (parent, name) = trimmed.rsplit_once('/').ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid virtual path")
    })?;
    if name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "virtual path name is empty",
        ));
    }
    let parent = if parent.is_empty() { "/" } else { parent };
    Ok((parent.to_string(), name.to_string()))
}

impl crate::block_backend::BlockBackend for VirtualExfatFs {
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        VirtualExfatFs::read_at(self, offset, len)
    }

    fn write_at(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        VirtualExfatFs::write_at(self, offset, data)
    }

    fn flush(&self) -> Result<(), std::io::Error> {
        VirtualExfatFs::flush(self)
    }

    fn shutdown(&self) -> Result<(), std::io::Error> {
        VirtualExfatFs::shutdown(self)
    }
}
