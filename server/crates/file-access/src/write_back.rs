//! 写回真实 U 盘。
//!
//! 处理 Windows 对虚拟卷的写操作，同步到 /mnt/usb_raw。

use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use tracing::warn;

use crate::exfat::volume::VirtualVolume;
use crate::types::SectorContent;

/// 写回管理器。
pub struct WriteBackManager {
    /// 真实 U 盘挂载路径（预留，后续版本用于合法文件写回）。
    #[allow(dead_code)]
    mount_path: PathBuf,
    /// 是否为只读模式。
    readonly: bool,
}

impl WriteBackManager {
    /// 创建写回管理器。
    pub fn new(mount_path: &Path, readonly: bool) -> Self {
        WriteBackManager {
            mount_path: mount_path.to_path_buf(),
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
                    warn!("拒绝写入被阻断文件的数据扇区: sector={}", sector);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("文件被策略阻断，禁止写入: sector={}", sector),
                    ));
                }
                self.write_file_data(&real_path, offset, data, valid_bytes)
            }
            SectorContent::Metadata(_) => {
                if volume.is_directory_sector(sector) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        format!("v1 不支持目录变更操作: sector={}", sector),
                    ));
                }
                Ok(())
            }
            SectorContent::Zero => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!("v1 不支持新文件创建: sector={}", sector),
                ))
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
        Ok(())
    }
}
