//! storage 启动恢复。
//!
//! 本模块统一清理本项目管理范围内的 gadget LUN、NBD 旧连接和 raw mount 残留。

use std::path::{Path, PathBuf};

use tracing::{info, warn};

use crate::nbd::{read_nbd_partition_scan_status, NbdDeviceManager, NbdPartitionScanStatus};
use crate::raw_mount::{mount_entries_from, planned_usb_raw_unmounts, MountOperations};
use crate::FileAccessError;

/// storage 启动恢复配置。
#[derive(Debug, Clone)]
pub struct StartupRecoveryConfig {
    pub mount_base: PathBuf,
    pub nbd_pool_size: u32,
    pub lun_file: PathBuf,
}

impl StartupRecoveryConfig {
    /// 生产环境启动恢复配置。
    pub fn production(lun_file: PathBuf) -> Self {
        Self {
            mount_base: PathBuf::from("/mnt/usb_raw"),
            nbd_pool_size: 4,
            lun_file,
        }
    }
}

/// storage 启动恢复报告。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StartupRecoveryReport {
    pub cleared_lun: bool,
    pub disconnected_nbd: usize,
    pub recovered_mounts: usize,
}

/// 清空 mass storage LUN backing file。
pub fn clear_lun_backing_file(lun_file: &Path) -> Result<(), FileAccessError> {
    std::fs::write(lun_file, "\n").map_err(|e| {
        FileAccessError::RecoveryFailed(format!(
            "清空 LUN backing {} 失败: {}",
            lun_file.display(),
            e
        ))
    })
}

/// 基于 mount table 文本恢复 raw mount 残留。
pub fn recover_raw_mounts_under(
    mount_ops: &dyn MountOperations,
    mount_base: &Path,
    mounts_text: &str,
) -> Result<StartupRecoveryReport, FileAccessError> {
    let entries = mount_entries_from(mounts_text);
    let mount_base = mount_base.to_string_lossy();
    let mut report = StartupRecoveryReport::default();

    for target in planned_usb_raw_unmounts(&entries, &mount_base) {
        match mount_ops.umount(&target) {
            Ok(()) => {
                report.recovered_mounts += 1;
                info!(mount_point = %target, "启动恢复: 清理残留 U 盘挂载");
            }
            Err(e) => warn!(
                mount_point = %target,
                error = %e,
                "启动恢复: 清理残留 U 盘挂载失败"
            ),
        }
    }

    Ok(report)
}

/// 执行 storage 启动恢复。
pub fn run_startup_recovery(
    config: &StartupRecoveryConfig,
    mount_ops: &dyn MountOperations,
    nbd_manager: &NbdDeviceManager,
) -> Result<StartupRecoveryReport, FileAccessError> {
    let mut report = StartupRecoveryReport::default();

    clear_lun_backing_file(&config.lun_file)?;
    report.cleared_lun = true;

    let connected = nbd_manager
        .connected_devices_for_recovery(config.nbd_pool_size)
        .map_err(|e| FileAccessError::RecoveryFailed(format!("读取 NBD 恢复状态失败: {e}")))?;
    report.disconnected_nbd = connected.len();
    nbd_manager.recover_pool(config.nbd_pool_size);

    let mounts = std::fs::read_to_string("/proc/mounts")
        .map_err(|e| FileAccessError::RecoveryFailed(format!("读取 /proc/mounts 失败: {e}")))?;
    let mount_report = recover_raw_mounts_under(mount_ops, &config.mount_base, &mounts)?;
    report.recovered_mounts = mount_report.recovered_mounts;

    match read_nbd_partition_scan_status() {
        Ok(NbdPartitionScanStatus::Disabled) => info!("NBD 本机分区扫描已关闭"),
        Ok(NbdPartitionScanStatus::Enabled(max_part)) => warn!(
            max_part,
            "NBD 本机分区扫描已开启，生产镜像应配置 nbd.max_part=0，避免 nbdXpY 事件风暴"
        ),
        Err(e) => warn!(error = %e, "读取 NBD 分区扫描配置失败"),
    }

    Ok(report)
}
