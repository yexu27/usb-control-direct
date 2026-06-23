//! udev 设备事件监听。
//!
//! 使用 udev crate 监听 USB 设备热插拔事件，
//! 通过 mpsc channel 发送 DeviceEvent 给主编排器处理。

use tokio::sync::mpsc;
use tokio::task;
use tracing::{error, info};

use crate::descriptor::{classify_device, parse_hex_u8, read_sysfs_attr, UsbDeviceInfo};
use crate::orchestrator::DeviceEvent;

/// 启动 udev 监听循环。
///
/// 在 spawn_blocking 中运行 udev 事件轮询（阻塞操作）。
///
/// 参数:
///   - tx: 事件发送端。
///   - shutdown: 关闭信号接收端。
pub async fn start_udev_monitor(
    tx: mpsc::UnboundedSender<DeviceEvent>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) {
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
                            let event = match info.device_type {
                                common::types::DeviceType::Storage => {
                                    DeviceEvent::StorageAdded(info)
                                }
                                common::types::DeviceType::Keyboard => {
                                    DeviceEvent::KeyboardAdded(info)
                                }
                                common::types::DeviceType::Mouse => {
                                    DeviceEvent::MouseAdded(info)
                                }
                                _ => DeviceEvent::UnsupportedAdded(
                                    info,
                                    "不支持的 USB 设备类型".into(),
                                ),
                            };
                            let _ = tx.send(event);
                        }
                    }
                    "remove" => {
                        info!(sys_path = %sys_path, "USB 设备拔出事件");
                        let _ = tx.send(DeviceEvent::DeviceRemoved(sys_path));
                    }
                    _ => {}
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
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

/// 启动时枚举存量设备并通过 channel 发送。
pub fn enumerate_and_send(tx: mpsc::UnboundedSender<DeviceEvent>) {
    use udev::Enumerator;

    let mut enumerator = match Enumerator::new() {
        Ok(e) => e,
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
        if devtype != "usb_interface" {
            continue;
        }

        if let Some(info) = parse_device_info_from_device(&device) {
            let event = match info.device_type {
                common::types::DeviceType::Storage => DeviceEvent::StorageAdded(info),
                common::types::DeviceType::Keyboard => DeviceEvent::KeyboardAdded(info),
                common::types::DeviceType::Mouse => DeviceEvent::MouseAdded(info),
                _ => DeviceEvent::UnsupportedAdded(info, "不支持的设备类型".into()),
            };
            let _ = tx.send(event);
        }
    }
}

/// 从 USB 父设备 sysfs 目录查找块设备路径和容量。
///
/// 遍历 parent_path 下所有 host*/target*/block/sd* 子路径，
/// 提取 /dev/sdX 和扇区数 × 512 = 字节容量。
fn find_block_device(parent_path: &std::path::Path) -> Option<(String, i64)> {
    let entries = std::fs::read_dir(parent_path).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("host") {
            continue;
        }
        if let Some(result) = find_block_in_host(&path) {
            return Some(result);
        }
    }
    None
}

/// 在 host 目录下递归查找 block/sd* 设备。
fn find_block_in_host(host_path: &std::path::Path) -> Option<(String, i64)> {
    let entries = std::fs::read_dir(host_path).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "block" {
            return find_sd_device(&path);
        }
        if let Some(result) = find_block_in_host(&path) {
            return Some(result);
        }
    }
    None
}

/// 在 block/ 目录下找到 sd* 设备分区。
///
/// 整盘 sdX 是子目录（内含 sdX1/sdX2），分区 sdXN 有 size 和 dev 文件。
/// 递归扫描，取第一个分区。
fn find_sd_device(block_path: &std::path::Path) -> Option<(String, i64)> {
    let entries = std::fs::read_dir(block_path).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("sd") {
            continue;
        }
        let path = entry.path();

        // 分区 sdXN: 直接读 size + dev
        if path.join("dev").exists() {
            let size = std::fs::read_to_string(path.join("size"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);
            let dev = format!("/dev/{}", name);
            // 读 partition 文件判断: 0=整盘, 非0=分区
            let is_partition = std::fs::read_to_string(path.join("partition"))
                .ok()
                .map(|s| s.trim() != "0")
                .unwrap_or_else(|| name.chars().any(|c| c.is_ascii_digit() && c != '0'));
            if is_partition {
                return Some((dev, size as i64 * 512));
            }
        }

        // 整盘子目录: 递归查找分区 sdX1/sdX2
        if path.is_dir() {
            if let Some(result) = find_sd_device(&path) {
                return Some(result);
            }
        }
    }
    None
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

    let (dev_path, capacity_bytes) = if device_type == common::types::DeviceType::Storage {
        find_block_device(syspath)
            .map(|(dev, cap)| (Some(dev), Some(cap)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    Some(UsbDeviceInfo {
        sys_path: syspath.to_string_lossy().to_string(),
        dev_path,
        serial_number: serial,
        vid,
        pid,
        device_name: product,
        device_type,
        interface_class,
        interface_subclass,
        interface_protocol,
        capacity_bytes,
    })
}

/// 从 udev::Device 解析设备信息（用于启动枚举，无 Event 对象）。
fn parse_device_info_from_device(device: &udev::Device) -> Option<UsbDeviceInfo> {
    let syspath = device.syspath();

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

    let (dev_path, capacity_bytes) = if device_type == common::types::DeviceType::Storage {
        find_block_device(syspath)
            .map(|(dev, cap)| (Some(dev), Some(cap)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    Some(UsbDeviceInfo {
        sys_path: syspath.to_string_lossy().to_string(),
        dev_path,
        serial_number: serial,
        vid,
        pid,
        device_name: product,
        device_type,
        interface_class,
        interface_subclass,
        interface_protocol,
        capacity_bytes,
    })
}
