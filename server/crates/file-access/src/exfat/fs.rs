//! 受控虚拟 exFAT 文件系统 facade。

use std::path::Path;
use std::sync::Mutex;

use tracing::debug;

use crate::block_backend::BlockWriteOutcome;
use crate::exfat::layout::SECTOR_SIZE;
use crate::exfat::runtime_state::ExfatRuntimeState;
use crate::types::{ControlledEntry, PolicySnapshot};
use crate::vfs::NodeId;

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

    pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<BlockWriteOutcome, std::io::Error> {
        self.runtime.lock().unwrap().write_at(offset, data)
    }

    pub fn flush(&self) -> Result<(), std::io::Error> {
        self.runtime.lock().unwrap().flush()
    }

    pub fn shutdown(&self) -> Result<(), std::io::Error> {
        self.runtime.lock().unwrap().shutdown()
    }
}

impl crate::block_backend::BlockBackend for VirtualExfatFs {
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        VirtualExfatFs::read_at(self, offset, len)
    }

    fn write_at(&self, offset: u64, data: &[u8]) -> Result<BlockWriteOutcome, std::io::Error> {
        VirtualExfatFs::write_at(self, offset, data)
    }

    fn flush(&self) -> Result<(), std::io::Error> {
        VirtualExfatFs::flush(self)
    }

    fn shutdown(&self) -> Result<(), std::io::Error> {
        VirtualExfatFs::shutdown(self)
    }
}
