//! 服务启动恢复工具。
//!
//! Recovery 只清理本项目明确管理范围内的资源，避免误伤系统其他挂载或 NBD 用途。

use std::path::{Path, PathBuf};

use tracing::{info, warn};

use crate::error::UsbIdentifyError;
use crate::mount::{mount_entries_from, planned_usb_raw_unmounts, MountOperations};

/// 返回项目 NBD 池对应的设备路径。
pub fn nbd_devices_for_pool(pool_size: u32) -> Vec<String> {
    (0..pool_size)
        .map(|idx| format!("/dev/nbd{idx}"))
        .collect()
}

/// 在指定 sysfs 根目录下读取 NBD pid。
pub fn read_nbd_pid_under(
    sys_block_root: &Path,
    nbd_name: &str,
) -> Result<Option<u32>, UsbIdentifyError> {
    let pid_path = sys_block_root.join(nbd_name).join("pid");
    let value = match std::fs::read_to_string(&pid_path) {
        Ok(value) => value,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(UsbIdentifyError::Internal(format!(
                "读取 {} 失败: {}",
                pid_path.display(),
                e
            )));
        }
    };

    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return Ok(None);
    }

    trimmed.parse::<u32>().map(Some).map_err(|e| {
        UsbIdentifyError::Internal(format!("解析 {} 内容失败: {}", pid_path.display(), e))
    })
}

/// 判断 NBD 是否需要断开。
pub fn should_disconnect_nbd_under(
    sys_block_root: &Path,
    nbd_name: &str,
) -> Result<bool, UsbIdentifyError> {
    Ok(read_nbd_pid_under(sys_block_root, nbd_name)?.is_some())
}

/// 清空 mass_storage LUN backing file。
pub fn clear_lun_backing_file(lun_file: &Path) -> Result<(), UsbIdentifyError> {
    std::fs::write(lun_file, "\n").map_err(|e| {
        UsbIdentifyError::Internal(format!(
            "清空 LUN backing {} 失败: {}",
            lun_file.display(),
            e
        ))
    })
}

/// 启动恢复配置。
#[derive(Debug, Clone)]
pub struct StartupRecoveryConfig {
    pub mount_base: PathBuf,
    pub sys_block_root: PathBuf,
    pub nbd_pool_size: u32,
    pub lun_file: PathBuf,
}

impl StartupRecoveryConfig {
    pub fn production(lun_file: PathBuf) -> Self {
        Self {
            mount_base: PathBuf::from("/mnt/usb_raw"),
            sys_block_root: PathBuf::from("/sys/block"),
            nbd_pool_size: 4,
            lun_file,
        }
    }
}

/// 执行启动恢复中的原始 U 盘挂载清理。
pub fn recover_raw_mounts(
    mount_ops: &dyn MountOperations,
    mount_base: &Path,
) -> Result<(), UsbIdentifyError> {
    let mounts = std::fs::read_to_string("/proc/mounts")
        .map_err(|e| UsbIdentifyError::Internal(format!("读取 /proc/mounts 失败: {}", e)))?;
    let entries = mount_entries_from(&mounts);
    let mount_base = mount_base.to_string_lossy();

    for target in planned_usb_raw_unmounts(&entries, &mount_base) {
        match mount_ops.umount(&target) {
            Ok(()) => info!(mount_point = %target, "启动恢复: 清理残留 U 盘挂载"),
            Err(e) => warn!(
                mount_point = %target,
                error = %e,
                "启动恢复: 清理残留 U 盘挂载失败"
            ),
        }
    }

    Ok(())
}
