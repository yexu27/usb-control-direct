//! raw U 盘挂载管理。
//!
//! 本模块只负责真实 U 盘分区挂载、卸载和挂载表检查，不处理 USB 识别、
//! 病毒扫描、虚拟介质、NBD 或 gadget。

use std::path::PathBuf;

use tracing::{debug, info, warn};

use crate::error::FileAccessError;

const MOUNT_BASE: &str = "/mnt/usb_raw";

/// `/proc/mounts` 中的一条挂载记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    pub source: String,
    pub target: String,
    pub fs_type: String,
}

impl MountEntry {
    fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        fs_type: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            fs_type: fs_type.into(),
        }
    }
}

/// 挂载前置检查错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountPreflightError {
    SourceAlreadyMounted { source: String, target: String },
    MountPointOccupied { source: String, target: String },
}

impl std::fmt::Display for MountPreflightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MountPreflightError::SourceAlreadyMounted { source, target } => {
                write!(f, "设备已挂载: {source} -> {target}")
            }
            MountPreflightError::MountPointOccupied { source, target } => {
                write!(f, "挂载点已被其它设备占用: {source} -> {target}")
            }
        }
    }
}

impl std::error::Error for MountPreflightError {}

/// 解析 `/proc/mounts` 内容。
pub fn mount_entries_from(contents: &str) -> Vec<MountEntry> {
    contents
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let source = parts.next()?;
            let target = parts.next()?;
            let fs_type = parts.next()?;
            Some(MountEntry::new(source, target, fs_type))
        })
        .collect()
}

/// 选择本服务原始 U 盘挂载目录下的残留挂载点。
pub fn planned_usb_raw_unmounts(entries: &[MountEntry], mount_base: &str) -> Vec<String> {
    let normalized = mount_base.trim_end_matches('/');
    let prefix = format!("{normalized}/");
    entries
        .iter()
        .filter(|entry| entry.target == normalized || entry.target.starts_with(&prefix))
        .map(|entry| entry.target.clone())
        .collect()
}

/// 判断挂载点是否仍在 mount table 中。
pub fn mount_target_exists_from(entries: &[MountEntry], mount_point: &str) -> bool {
    entries.iter().any(|entry| entry.target == mount_point)
}

/// 判断挂载点当前是否仍处于挂载状态。
pub fn mount_target_exists(mount_point: &str) -> Result<bool, FileAccessError> {
    Ok(mount_target_exists_from(
        &current_mount_entries()?,
        mount_point,
    ))
}

/// 基于已解析 mount table 检查挂载目标是否可用。
pub fn ensure_mount_available_from(
    entries: &[MountEntry],
    dev_path: &str,
    mount_point: &str,
) -> Result<(), MountPreflightError> {
    if let Some(entry) = entries.iter().find(|entry| entry.source == dev_path) {
        return Err(MountPreflightError::SourceAlreadyMounted {
            source: entry.source.clone(),
            target: entry.target.clone(),
        });
    }

    if let Some(entry) = entries.iter().find(|entry| entry.target == mount_point) {
        return Err(MountPreflightError::MountPointOccupied {
            source: entry.source.clone(),
            target: entry.target.clone(),
        });
    }

    Ok(())
}

fn current_mount_entries() -> Result<Vec<MountEntry>, FileAccessError> {
    let mounts = std::fs::read_to_string("/proc/mounts")
        .map_err(|e| FileAccessError::MountFailed(format!("读取 /proc/mounts 失败: {e}")))?;
    Ok(mount_entries_from(&mounts))
}

/// mount 操作 trait，便于测试时 mock。
pub trait MountOperations: Send + Sync {
    fn is_mounted(&self, dev_path: &str) -> Result<bool, FileAccessError>;

    fn mount(
        &self,
        dev_path: &str,
        mount_point: &str,
        fs_type: &str,
    ) -> Result<(), FileAccessError>;

    fn umount(&self, mount_point: &str) -> Result<(), FileAccessError>;

    fn detect_fs_type(&self, dev_path: &str) -> Result<String, FileAccessError>;
}

/// 真实的 mount 操作实现。
#[derive(Debug, Clone, Copy, Default)]
pub struct RealMountOps;

impl MountOperations for RealMountOps {
    fn is_mounted(&self, dev_path: &str) -> Result<bool, FileAccessError> {
        debug!(dev = %dev_path, "检查设备是否已挂载");
        let mounts = std::fs::read_to_string("/proc/mounts")
            .map_err(|e| FileAccessError::MountFailed(format!("读取 /proc/mounts 失败: {e}")))?;
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
    ) -> Result<(), FileAccessError> {
        std::fs::create_dir_all(mount_point).map_err(|e| {
            FileAccessError::MountFailed(format!("创建挂载点 {mount_point} 失败: {e}"))
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
            FileAccessError::MountFailed(format!(
                "挂载 {dev_path} -> {mount_point} (fs={fs_type}) 失败: {e}"
            ))
        })?;

        info!(dev = dev_path, mount_point = mount_point, fs_type = fs_type, "挂载设备成功");
        Ok(())
    }

    fn umount(&self, mount_point: &str) -> Result<(), FileAccessError> {
        nix::mount::umount2(mount_point, nix::mount::MntFlags::MNT_DETACH).map_err(|e| {
            FileAccessError::UmountFailed(format!("卸载 {mount_point} 失败: {e}"))
        })?;

        info!(mount_point = mount_point, "卸载设备成功（懒卸载）");
        Ok(())
    }

    fn detect_fs_type(&self, dev_path: &str) -> Result<String, FileAccessError> {
        let output = std::process::Command::new("blkid")
            .args(["-o", "value", "-s", "TYPE", dev_path])
            .output()
            .map_err(|e| FileAccessError::MountFailed(format!("执行 blkid 失败: {e}")))?;

        if !output.status.success() {
            return Err(FileAccessError::MountFailed(format!(
                "blkid 返回错误: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let fs_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if fs_type.is_empty() {
            return Err(FileAccessError::MountFailed("无法检测文件系统类型".into()));
        }

        debug!(dev = dev_path, fs_type = %fs_type, "检测到文件系统类型");
        Ok(fs_type)
    }
}

/// 为设备生成挂载路径。
pub fn mount_path_for(dev_name: &str) -> PathBuf {
    PathBuf::from(MOUNT_BASE).join(dev_name)
}

/// 从块设备路径提取设备名。
pub fn dev_name_from_path(dev_path: &str) -> &str {
    dev_path.rsplit('/').next().unwrap_or(dev_path)
}

/// mount 三步递进:
/// 1. 内核自动探测
/// 2. ntfs-3g 命令
/// 3. 内核 ntfs 只读
pub fn mount_partition(
    dev_path: &str,
    mount_point: &str,
    read_only: bool,
) -> Result<(), FileAccessError> {
    std::fs::create_dir_all(mount_point).map_err(|e| {
        FileAccessError::MountFailed(format!("创建挂载点 {mount_point} 失败: {e}"))
    })?;

    let entries = current_mount_entries()?;
    ensure_mount_available_from(&entries, dev_path, mount_point)
        .map_err(|e| FileAccessError::MountFailed(format!("挂载前检查失败: {e}")))?;

    let flags = if read_only {
        nix::mount::MsFlags::MS_RDONLY
            | nix::mount::MsFlags::MS_NOEXEC
            | nix::mount::MsFlags::MS_NOSUID
    } else {
        nix::mount::MsFlags::MS_NOEXEC | nix::mount::MsFlags::MS_NOSUID
    };

    if nix::mount::mount(Some(dev_path), mount_point, None::<&str>, flags, None::<&str>).is_ok() {
        info!(dev = dev_path, mount_point = mount_point, "自动检测挂载成功");
        return Ok(());
    }

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

    if nix::mount::mount(
        Some(dev_path),
        mount_point,
        Some("ntfs"),
        nix::mount::MsFlags::MS_RDONLY
            | nix::mount::MsFlags::MS_NOEXEC
            | nix::mount::MsFlags::MS_NOSUID,
        None::<&str>,
    )
    .is_ok()
    {
        warn!(dev = dev_path, mount_point = mount_point, "回退到内核 ntfs 只读挂载");
        return Ok(());
    }

    Err(FileAccessError::MountFailed(format!(
        "挂载 {dev_path} -> {mount_point} 失败（已尝试所有方式）"
    )))
}
