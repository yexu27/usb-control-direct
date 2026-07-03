use tokio::sync::mpsc;
use tracing::{error, info};

use crate::event_source::{
    device_event_from_info, is_usb_interface_devtype, parse_device_info_from_syspath,
};
use crate::orchestrator::DeviceEvent;

/// 启动时 USB 存量设备枚举器。
///
/// 只负责扫描当前已存在的 usb_interface，并转换成 `DeviceEvent`。
pub struct UsbEnumerator;

impl UsbEnumerator {
    /// 枚举当前系统中已存在的 USB interface 并发送内部事件。
    pub fn enumerate_and_send(tx: &mpsc::UnboundedSender<DeviceEvent>) {
        let mut enumerator = match udev::Enumerator::new() {
            Ok(enumerator) => enumerator,
            Err(e) => {
                error!("创建 udev Enumerator 失败: {}", e);
                return;
            }
        };

        if let Err(e) = enumerator.match_subsystem("usb") {
            error!("设置 udev 匹配子系统失败: {}", e);
            return;
        }

        for device in enumerator.scan_devices().into_iter().flatten() {
            let devtype = device
                .property_value("DEVTYPE")
                .map(|v| v.to_string_lossy().to_string())
                .unwrap_or_default();
            if !is_usb_interface_devtype(&devtype) {
                continue;
            }

            if let Some(info) = parse_device_info_from_syspath(device.syspath()) {
                let _ = tx.send(device_event_from_info(info));
            }
        }

        info!("USB 存量设备枚举完成");
    }
}
