//! 写回真实 U 盘。
//!
//! 处理 Windows 对虚拟卷的写操作，同步到 /mnt/usb_raw。
//! Windows 写文件时先写数据簇再更新目录项（乱序写入），使用待写缓冲区暂存。

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

use crate::exfat::volume::VirtualVolume;
use crate::types::SectorContent;

/// 写回管理器。
pub struct WriteBackManager {
    /// 真实 U 盘挂载路径。
    mount_path: PathBuf,
    /// 待写数据缓冲区（扇区号 → 数据）。
    pending_data: HashMap<u64, Vec<u8>>,
    /// 是否为只读模式。
    readonly: bool,
}

impl WriteBackManager {
    /// 创建写回管理器。
    pub fn new(mount_path: &Path, readonly: bool) -> Self {
        WriteBackManager {
            mount_path: mount_path.to_path_buf(),
            pending_data: HashMap::new(),
            readonly,
        }
    }

    /// 处理写请求。
    ///
    /// 参数:
    ///   - sector: 扇区号。
    ///   - data: 写入的数据（512 字节）。
    ///   - volume: 虚拟卷引用（用于判断扇区类型）。
    ///
    /// 返回:
    ///   - Ok(()) 表示成功（或忽略），Err 表示写入失败。
    pub fn handle_write(
        &mut self,
        sector: u64,
        data: &[u8],
        volume: &VirtualVolume,
    ) -> Result<(), std::io::Error> {
        if self.readonly {
            return Ok(());
        }

        let content = volume.read_sector(sector);
        match content {
            SectorContent::FileData { real_path, offset, valid_bytes, blocked } => {
                if blocked {
                    debug!("拒绝写入被阻断文件的数据扇区: sector={}", sector);
                    return Ok(());
                }
                self.write_file_data(&real_path, offset, data, valid_bytes)
            }
            SectorContent::Metadata(_) => {
                if volume.is_directory_sector(sector) {
                    // 目录扇区写入：暂存，后续解析目录变化
                    self.pending_data.insert(sector, data.to_vec());
                    debug!("暂存目录扇区写入: sector={}", sector);
                }
                // 其他元数据扇区（boot / FAT / bitmap）忽略
                Ok(())
            }
            SectorContent::Zero => {
                // 可能是 Windows 写新文件到空闲簇，暂存
                self.pending_data.insert(sector, data.to_vec());
                Ok(())
            }
        }
    }

    /// 写入文件数据到真实文件。
    fn write_file_data(
        &self,
        real_path: &Path,
        offset: u64,
        data: &[u8],
        valid_bytes: u32,
    ) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new().write(true).open(real_path)?;
        file.seek(SeekFrom::Start(offset))?;
        let write_len = valid_bytes.min(data.len() as u32) as usize;
        file.write_all(&data[..write_len])?;
        file.flush()?;
        Ok(())
    }

    /// 刷新挂起的写入。
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        if !self.pending_data.is_empty() {
            warn!(
                "丢弃 {} 个未处理的待写扇区（mount_path={}）",
                self.pending_data.len(),
                self.mount_path.display()
            );
            self.pending_data.clear();
        }
        Ok(())
    }
}
