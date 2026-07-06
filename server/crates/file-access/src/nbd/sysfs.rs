//! NBD sysfs 状态读取与运行环境检查。
//!
//! 本模块只读取 Linux NBD 运行时状态，不做 ioctl、不管理 fd，也不理解业务会话。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// NBD 本机分区扫描状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbdPartitionScanStatus {
    Disabled,
    Enabled(u32),
}

#[derive(Debug, Clone)]
pub struct NbdSysfs {
    root: PathBuf,
}

impl Default for NbdSysfs {
    fn default() -> Self {
        Self::new("/sys/block")
    }
}

impl NbdSysfs {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn is_connected(&self, name: &str) -> Result<bool, std::io::Error> {
        let value = std::fs::read_to_string(self.root.join(name).join("pid"))?;
        let value = value.trim();
        Ok(!value.is_empty() && value != "0")
    }

    pub fn wait_ready(
        &self,
        name: &str,
        expected_sectors: u64,
        timeout: Duration,
    ) -> Result<(), std::io::Error> {
        let nbd_sys = self.root.join(name);
        let deadline = Instant::now() + timeout;
        let mut stable_matches = 0;

        loop {
            let pid_ready = std::fs::read_to_string(nbd_sys.join("pid"))
                .map(|value| {
                    let value = value.trim();
                    !value.is_empty() && value != "0"
                })
                .unwrap_or(false);
            let size_ready = std::fs::read_to_string(nbd_sys.join("size"))
                .ok()
                .and_then(|value| value.trim().parse::<u64>().ok())
                .map(|size| size == expected_sectors)
                .unwrap_or(false);

            if pid_ready && size_ready {
                stable_matches += 1;
                if stable_matches >= 2 {
                    return Ok(());
                }
            } else {
                stable_matches = 0;
            }

            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "NBD device {name} not ready: expected size {expected_sectors} sectors"
                    ),
                ));
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn wait_disconnected(&self, name: &str, timeout: Duration) -> Result<(), std::io::Error> {
        let deadline = Instant::now() + timeout;

        loop {
            let pid_connected = self.is_connected(name).unwrap_or(false);
            if !pid_connected {
                return Ok(());
            }

            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("NBD device {name} still connected"),
                ));
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }
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
