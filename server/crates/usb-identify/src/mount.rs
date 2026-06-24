//! mount/umount 封装。
//!
//! 使用 nix crate 执行 mount/umount 系统调用。
//! 注意：实际 mount/umount 需要 root 权限，单元测试中使用 trait 抽象隔离。

use std::path::PathBuf;

use tracing::{debug, info, warn};

use crate::error::UsbIdentifyError;

/// USB 挂载目录前缀。
const MOUNT_BASE: &str = "/mnt/usb_raw";

/// mount 操作 trait，便于测试时 mock。
pub trait MountOperations: Send + Sync {
    /// 检查设备是否已挂载。
    fn is_mounted(&self, dev_path: &str) -> Result<bool, UsbIdentifyError>;

    /// 挂载设备到指定路径。
    fn mount(
        &self,
        dev_path: &str,
        mount_point: &str,
        fs_type: &str,
    ) -> Result<(), UsbIdentifyError>;

    /// 卸载指定路径。
    fn umount(&self, mount_point: &str) -> Result<(), UsbIdentifyError>;

    /// 检测文件系统类型。
    fn detect_fs_type(&self, dev_path: &str) -> Result<String, UsbIdentifyError>;
}

/// 真实的 mount 操作实现（需要 root 权限）。
pub struct RealMountOps;

impl MountOperations for RealMountOps {
    fn is_mounted(&self, dev_path: &str) -> Result<bool, UsbIdentifyError> {
        debug!(dev = %dev_path, "检查设备是否已挂载");
        let mounts = std::fs::read_to_string("/proc/mounts")
            .map_err(|e| UsbIdentifyError::Internal(format!("读取 /proc/mounts 失败: {}", e)))?;
        Ok(mounts.lines().any(|line| {
            line.split_whitespace()
                .next()
                .map(|dev| dev == dev_path)
                .unwrap_or(false)
        }))
    }

    fn mount(
        &self,
        dev_path: &str,
        mount_point: &str,
        fs_type: &str,
    ) -> Result<(), UsbIdentifyError> {
        std::fs::create_dir_all(mount_point).map_err(|e| {
            UsbIdentifyError::MountFailed(format!("创建挂载点 {} 失败: {}", mount_point, e))
        })?;

        let fs_type_opt = if fs_type.is_empty() || fs_type == "auto" {
            None::<&str>
        } else {
            Some(fs_type)
        };
        nix::mount::mount(
            Some(dev_path),
            mount_point,
            fs_type_opt,
            nix::mount::MsFlags::MS_NOEXEC | nix::mount::MsFlags::MS_NOSUID,
            None::<&str>,
        )
        .map_err(|e| {
            UsbIdentifyError::MountFailed(format!(
                "挂载 {} -> {} (fs={}) 失败: {}",
                dev_path, mount_point, fs_type, e
            ))
        })?;

        info!(dev = dev_path, mount_point = mount_point, fs_type = fs_type, "挂载设备成功");
        Ok(())
    }

    fn umount(&self, mount_point: &str) -> Result<(), UsbIdentifyError> {
        nix::mount::umount2(mount_point, nix::mount::MntFlags::MNT_DETACH).map_err(|e| {
            UsbIdentifyError::UmountFailed(format!("卸载 {} 失败: {}", mount_point, e))
        })?;

        info!(mount_point = mount_point, "卸载设备成功（懒卸载）");
        Ok(())
    }

    fn detect_fs_type(&self, dev_path: &str) -> Result<String, UsbIdentifyError> {
        let output = std::process::Command::new("blkid")
            .args(["-o", "value", "-s", "TYPE", dev_path])
            .output()
            .map_err(|e| {
                UsbIdentifyError::Internal(format!("执行 blkid 失败: {}", e))
            })?;

        if !output.status.success() {
            return Err(UsbIdentifyError::Internal(format!(
                "blkid 返回错误: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let fs_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if fs_type.is_empty() {
            return Err(UsbIdentifyError::Internal(
                "无法检测文件系统类型".into(),
            ));
        }

        debug!(dev = dev_path, fs_type = %fs_type, "检测到文件系统类型");
        Ok(fs_type)
    }
}

/// 为设备生成挂载路径。
///
/// 参数:
///   - `dev_name`: 设备名（如 sda1）。
///
/// 返回:
///   - `/mnt/usb_raw/sda1`。
pub fn mount_path_for(dev_name: &str) -> PathBuf {
    PathBuf::from(MOUNT_BASE).join(dev_name)
}

/// 从块设备路径提取设备名。
///
/// 参数:
///   - `dev_path`: 如 `/dev/sda1`。
///
/// 返回:
///   - `sda1`。
pub fn dev_name_from_path(dev_path: &str) -> &str {
    dev_path
        .rsplit('/')
        .next()
        .unwrap_or(dev_path)
}

/// mount 三步递进（同 demo 逻辑）:
/// 1. 内核自动探测（`None` fs type）
/// 2. ntfs-3g 命令
/// 3. 内核 ntfs 只读
pub fn mount_partition(
    dev_path: &str,
    mount_point: &str,
    read_only: bool,
) -> Result<(), UsbIdentifyError> {
    std::fs::create_dir_all(mount_point).map_err(|e| {
        UsbIdentifyError::MountFailed(format!("创建挂载点 {} 失败: {}", mount_point, e))
    })?;

    let flags = if read_only {
        nix::mount::MsFlags::MS_RDONLY | nix::mount::MsFlags::MS_NOEXEC | nix::mount::MsFlags::MS_NOSUID
    } else {
        nix::mount::MsFlags::MS_NOEXEC | nix::mount::MsFlags::MS_NOSUID
    };

    // 1. 内核自动探测
    if nix::mount::mount(Some(dev_path), mount_point, None::<&str>, flags, None::<&str>).is_ok()
    {
        info!(dev = dev_path, mount_point = mount_point, "自动检测挂载成功");
        return Ok(());
    }

    // 2. ntfs-3g
    let mut cmd = std::process::Command::new("ntfs-3g");
    if read_only {
        cmd.arg("-o").arg("ro");
    }
    cmd.arg(dev_path).arg(mount_point);
    match cmd.output() {
        Ok(out) if out.status.success() => {
            info!(dev = dev_path, mount_point = mount_point, read_only, "ntfs-3g 挂载成功");
            return Ok(());
        }
        Ok(out) => {
            debug!(dev = dev_path, stderr = %String::from_utf8_lossy(&out.stderr), "ntfs-3g 失败");
        }
        Err(e) => {
            debug!(dev = dev_path, ?e, "ntfs-3g 执行失败");
        }
    }

    // 3. 内核 ntfs 只读兜底
    if nix::mount::mount(
        Some(dev_path),
        mount_point,
        Some("ntfs"),
        nix::mount::MsFlags::MS_RDONLY | nix::mount::MsFlags::MS_NOEXEC | nix::mount::MsFlags::MS_NOSUID,
        None::<&str>,
    )
    .is_ok()
    {
        warn!(dev = dev_path, mount_point = mount_point, "回退到内核 ntfs 只读挂载");
        return Ok(());
    }

    Err(UsbIdentifyError::MountFailed(format!(
        "挂载 {} -> {} 失败（已尝试所有方式）",
        dev_path, mount_point
    )))
}
