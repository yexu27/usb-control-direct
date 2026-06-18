//! OTG Gadget 控制。
//!
//! 通过 configfs f_mass_storage 将 NBD 设备暴露给受控主机。
//! RK3568 使用 rockchip UDC。

use std::fs;
use std::path::{Path, PathBuf};

use tracing::{error, info};

/// Gadget configfs 基础路径。
const GADGET_BASE: &str = "/sys/kernel/config/usb_gadget";

/// 默认 gadget 名称。
const GADGET_NAME: &str = "usb_ctrl";

/// 默认 UDC 名称（RK3568）。
const DEFAULT_UDC: &str = "fcc00000.dwc3";

/// OTG Gadget 管理器。
pub struct GadgetManager {
    /// gadget 路径。
    gadget_path: PathBuf,
    /// UDC 名称。
    udc_name: String,
    /// 是否已启用。
    enabled: bool,
}

impl Default for GadgetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GadgetManager {
    /// 创建 Gadget 管理器。
    pub fn new() -> Self {
        GadgetManager {
            gadget_path: PathBuf::from(GADGET_BASE).join(GADGET_NAME),
            udc_name: DEFAULT_UDC.to_string(),
            enabled: false,
        }
    }

    /// 使用自定义 UDC 名称。
    pub fn with_udc(mut self, udc: &str) -> Self {
        self.udc_name = udc.to_string();
        self
    }

    /// 启用 OTG 映射。
    ///
    /// 参数:
    ///   - nbd_device: NBD 设备路径（如 /dev/nbd0）。
    ///   - readonly: 是否为只读映射。
    ///   - device_description: 设备描述，写入 configfs strings/0x409/product。
    pub fn enable(
        &mut self,
        nbd_device: &Path,
        readonly: bool,
        device_description: &str,
    ) -> Result<(), std::io::Error> {
        self.setup_gadget(device_description)?;
        self.configure_mass_storage(nbd_device, readonly)?;
        self.bind_udc()?;
        self.enabled = true;
        info!(
            "OTG Gadget 已启用: device={}, readonly={}",
            nbd_device.display(),
            readonly
        );
        Ok(())
    }

    /// 禁用 OTG 映射。
    pub fn disable(&mut self) -> Result<(), std::io::Error> {
        if !self.enabled {
            return Ok(());
        }
        self.unbind_udc()?;
        self.enabled = false;
        info!("OTG Gadget 已禁用");
        Ok(())
    }

    /// 设置 gadget 基础结构。
    fn setup_gadget(&self, device_description: &str) -> Result<(), std::io::Error> {
        let gadget = &self.gadget_path;

        if !gadget.exists() {
            fs::create_dir_all(gadget)?;
        }

        write_file(&gadget.join("idVendor"), "0x1d6b")?;
        write_file(&gadget.join("idProduct"), "0x0104")?;
        write_file(&gadget.join("bcdDevice"), "0x0100")?;
        write_file(&gadget.join("bcdUSB"), "0x0200")?;

        let strings = gadget.join("strings/0x409");
        if !strings.exists() {
            fs::create_dir_all(&strings)?;
        }
        write_file(&strings.join("serialnumber"), "USB_CTRL_001")?;
        write_file(&strings.join("manufacturer"), "USB Security")?;
        write_file(&strings.join("product"), device_description)?;

        let config = gadget.join("configs/c.1");
        if !config.exists() {
            fs::create_dir_all(&config)?;
        }
        write_file(&config.join("MaxPower"), "500")?;

        let config_strings = config.join("strings/0x409");
        if !config_strings.exists() {
            fs::create_dir_all(&config_strings)?;
        }
        write_file(&config_strings.join("configuration"), "Mass Storage")?;

        Ok(())
    }

    /// 配置 f_mass_storage。
    fn configure_mass_storage(
        &self,
        nbd_device: &Path,
        readonly: bool,
    ) -> Result<(), std::io::Error> {
        let function = self.gadget_path.join("functions/mass_storage.usb0");
        if !function.exists() {
            fs::create_dir_all(&function)?;
        }

        let lun = function.join("lun.0");
        if !lun.exists() {
            fs::create_dir_all(&lun)?;
        }

        write_file(&lun.join("cdrom"), "0")?;
        write_file(&lun.join("removable"), "1")?;
        write_file(&lun.join("nofua"), "1")?;
        write_file(&lun.join("ro"), if readonly { "1" } else { "0" })?;
        write_file(&lun.join("file"), &nbd_device.to_string_lossy())?;

        let link_target = self.gadget_path.join("configs/c.1/mass_storage.usb0");
        if !link_target.exists() {
            std::os::unix::fs::symlink(&function, &link_target)?;
        }

        Ok(())
    }

    /// 绑定 UDC。
    fn bind_udc(&self) -> Result<(), std::io::Error> {
        write_file(&self.gadget_path.join("UDC"), &self.udc_name)
    }

    /// 解绑 UDC。
    fn unbind_udc(&self) -> Result<(), std::io::Error> {
        write_file(&self.gadget_path.join("UDC"), "")
    }
}

impl Drop for GadgetManager {
    fn drop(&mut self) {
        if self.enabled {
            if let Err(e) = self.disable() {
                error!("GadgetManager drop 时禁用失败: {}", e);
            }
        }
    }
}

/// 写入 sysfs/configfs 文件。
fn write_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    fs::write(path, content)
}
