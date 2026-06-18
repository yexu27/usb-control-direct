//! udev 设备事件监听。
//!
//! 使用 udev crate 监听 USB 设备热插拔事件，
//! 在事件回调中调用 DeviceManager 进行设备生命周期管理。

use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::task;
use tracing::{error, info, warn};

use crate::descriptor::{classify_device, parse_hex_u8, read_sysfs_attr, UsbDeviceInfo};
use crate::monitor::DeviceManager;

/// 启动 udev 监听循环。
///
/// 在 spawn_blocking 中运行 udev 事件轮询（阻塞操作），
/// 通过回调通知外部处理插入/拔出事件。
///
/// 参数:
///   - device_manager: 设备管理器（共享）。
///   - shutdown: 关闭信号接收端。
pub async fn start_udev_monitor(
    device_manager: Arc<RwLock<DeviceManager>>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    let dm = device_manager;

    let handle = task::spawn_blocking(move || {
        let socket = match udev::MonitorBuilder::new()
            .and_then(|b| b.match_subsystem("usb"))
            .and_then(|b| b.listen())
        {
            Ok(s) => s,
            Err(e) => {
                error!("udev 监听器创建失败: {}", e);
                return;
            }
        };

        info!("udev USB 设备监听已启动");

        loop {
            let mut iter = socket.iter();
            while let Some(event) = iter.next() {
                let action = match event.action() {
                    Some(a) => a.to_string_lossy().to_string(),
                    None => continue,
                };

                let sys_path = event.syspath().to_string_lossy().to_string();
                let devtype = event
                    .property_value("DEVTYPE")
                    .map(|v| v.to_string_lossy().to_string())
                    .unwrap_or_default();

                if devtype != "usb_interface" {
                    continue;
                }

                match action.as_str() {
                    "add" => {
                        info!(sys_path = %sys_path, "USB 设备插入事件");
                        if let Some(info) = parse_device_info(&event) {
                            let mut mgr = match dm.write() {
                                Ok(m) => m,
                                Err(e) => {
                                    error!("获取 DeviceManager 写锁失败: {}", e);
                                    continue;
                                }
                            };
                            if let Err(e) = mgr.handle_device_added(info) {
                                warn!("处理设备插入失败: {}", e);
                            }
                        }
                    }
                    "remove" => {
                        info!(sys_path = %sys_path, "USB 设备拔出事件");
                        let mut mgr = match dm.write() {
                            Ok(m) => m,
                            Err(e) => {
                                error!("获取 DeviceManager 写锁失败: {}", e);
                                continue;
                            }
                        };
                        mgr.handle_device_removed(&sys_path);
                    }
                    _ => {}
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    tokio::select! {
        _ = shutdown.recv() => {
            info!("udev 监听收到关闭信号");
        }
        result = handle => {
            if let Err(e) = result {
                error!("udev 监听线程异常退出: {}", e);
            }
        }
    }
}

/// 从 udev 事件解析设备信息。
fn parse_device_info(event: &udev::Event) -> Option<UsbDeviceInfo> {
    let syspath = event.syspath();

    let interface_class = read_sysfs_attr(syspath, "bInterfaceClass")
        .and_then(|s| parse_hex_u8(&s))
        .unwrap_or(0);
    let interface_subclass = read_sysfs_attr(syspath, "bInterfaceSubClass")
        .and_then(|s| parse_hex_u8(&s))
        .unwrap_or(0);
    let interface_protocol = read_sysfs_attr(syspath, "bInterfaceProtocol")
        .and_then(|s| parse_hex_u8(&s))
        .unwrap_or(0);

    let device_type = classify_device(interface_class, interface_subclass, interface_protocol);

    let parent = syspath.parent()?;
    let vid = read_sysfs_attr(parent, "idVendor").unwrap_or_default();
    let pid = read_sysfs_attr(parent, "idProduct").unwrap_or_default();
    let serial = read_sysfs_attr(parent, "serial").unwrap_or_default();
    let product = read_sysfs_attr(parent, "product").unwrap_or_default();

    Some(UsbDeviceInfo {
        sys_path: syspath.to_string_lossy().to_string(),
        dev_path: event.devnode().map(|p| p.to_string_lossy().to_string()),
        serial_number: serial,
        vid,
        pid,
        device_name: product,
        device_type,
        interface_class,
        interface_subclass,
        interface_protocol,
        capacity_bytes: None,
    })
}
