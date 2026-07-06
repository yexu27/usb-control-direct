//! NBD sysfs 状态读取与运行环境检查。
//!
//! 本模块只读取 Linux NBD 运行时状态，不做 ioctl、不管理 fd，也不理解业务会话。

use std::path::Path;

/// NBD 本机分区扫描状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbdPartitionScanStatus {
    Disabled,
    Enabled(u32),
}

pub fn nbd_name_from_device_path(path: &Path) -> Result<String, std::io::Error> {
    let name = path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid NBD path: {}", path.display()),
        )
    })?;
    let name = name.to_string_lossy();
    let Some(rest) = name.strip_prefix("nbd") else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("not an NBD device: {}", path.display()),
        ));
    };
    if rest.is_empty() || !rest.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("NBD partition is not allowed: {}", path.display()),
        ));
    }
    Ok(name.to_string())
}

pub fn parse_nbd_max_part(value: &str) -> Result<NbdPartitionScanStatus, std::io::Error> {
    let parsed = value.trim().parse::<u32>().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid nbd max_part value: {}", e),
        )
    })?;

    if parsed == 0 {
        Ok(NbdPartitionScanStatus::Disabled)
    } else {
        Ok(NbdPartitionScanStatus::Enabled(parsed))
    }
}

pub fn read_nbd_partition_scan_status() -> Result<NbdPartitionScanStatus, std::io::Error> {
    let value = std::fs::read_to_string("/sys/module/nbd/parameters/max_part")?;
    parse_nbd_max_part(&value)
}

pub fn ensure_partition_scan_disabled() -> Result<(), std::io::Error> {
    match read_nbd_partition_scan_status()? {
        NbdPartitionScanStatus::Disabled => Ok(()),
        NbdPartitionScanStatus::Enabled(max_part) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("NBD max_part={max_part}, expected 0 to prevent nbdXpY event storm"),
        )),
    }
}
