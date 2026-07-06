//! NBD 设备管理器。
//!
//! 本模块负责启动前检查、旧连接恢复、NBD 启动和 ready 等待。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tracing::{info, warn};

use crate::block_backend::BlockBackend;

use super::device::{nbd_ioctl, NbdDevice, NBD_CLEAR_QUE, NBD_CLEAR_SOCK, NBD_DISCONNECT};
use super::sysfs::{
    nbd_name_from_device_path, parse_nbd_max_part, NbdPartitionScanStatus, NbdSysfs,
};

/// NBD 设备管理器。
#[derive(Debug, Clone)]
pub struct NbdDeviceManager {
    sysfs: NbdSysfs,
    dev_root: PathBuf,
    max_part_path: PathBuf,
}

impl Default for NbdDeviceManager {
    fn default() -> Self {
        Self::new("/sys/block", "/dev", "/sys/module/nbd/parameters/max_part")
    }
}

impl NbdDeviceManager {
    pub fn new(
        sys_block_root: impl AsRef<Path>,
        dev_root: impl AsRef<Path>,
        max_part_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            sysfs: NbdSysfs::new(sys_block_root),
            dev_root: dev_root.as_ref().to_path_buf(),
            max_part_path: max_part_path.as_ref().to_path_buf(),
        }
    }

    pub fn connected_devices_for_recovery(
        &self,
        pool_size: u32,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut devices = Vec::new();
        for idx in 0..pool_size {
            let name = format!("nbd{idx}");
            if self.sysfs.is_connected(&name).unwrap_or(false) {
                devices.push(self.dev_root.join(name));
            }
        }
        Ok(devices)
    }

    pub fn recover_pool(&self, pool_size: u32) {
        match self.connected_devices_for_recovery(pool_size) {
            Ok(devices) => {
                for device in devices {
                    match self.disconnect_device(&device) {
                        Ok(()) => info!(device = %device.display(), "启动恢复: 断开旧 NBD 连接"),
                        Err(e) => warn!(
                            device = %device.display(),
                            error = %e,
                            "启动恢复: 断开旧 NBD 连接失败"
                        ),
                    }
                }
            }
            Err(e) => warn!(error = %e, "启动恢复: 读取 NBD sysfs 失败"),
        }
    }

    pub async fn start(
        &self,
        nbd_index: u32,
        total_sectors: u64,
        readonly: bool,
        backend: Arc<dyn BlockBackend>,
    ) -> Result<NbdDevice, std::io::Error> {
        self.ensure_partition_scan_disabled()?;

        let device_path = self.dev_root.join(format!("nbd{nbd_index}"));
        let name = nbd_name_from_device_path(&device_path)?;
        if self.sysfs.is_connected(&name).unwrap_or(false) {
            self.disconnect_device(&device_path)?;
            self.sysfs
                .wait_disconnected(&name, Duration::from_secs(2))?;
        }

        let device = NbdDevice::start(device_path, total_sectors, readonly, backend)?;
        self.sysfs
            .wait_ready(&name, total_sectors, Duration::from_millis(500))?;
        Ok(device)
    }

    fn ensure_partition_scan_disabled(&self) -> Result<(), std::io::Error> {
        let value = std::fs::read_to_string(&self.max_part_path)?;
        match parse_nbd_max_part(&value)? {
            NbdPartitionScanStatus::Disabled => Ok(()),
            NbdPartitionScanStatus::Enabled(max_part) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("NBD max_part must be 0 for production mapping, got {max_part}"),
            )),
        }
    }

    fn disconnect_device(&self, device: &Path) -> Result<(), std::io::Error> {
        let _ = nbd_name_from_device_path(device)?;
        let nbd_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)?;
        use std::os::unix::io::AsRawFd;
        let nbd_fd = nbd_file.as_raw_fd();

        // 安全性: nbd_fd 来自刚打开的 NBD 设备文件，ioctl 参数不携带用户指针。
        unsafe {
            let _ = nbd_ioctl(nbd_fd, NBD_DISCONNECT, 0);
            let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
            let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
        }
        Ok(())
    }
}
