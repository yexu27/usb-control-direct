use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::descriptor::{classify_device, parse_hex_u8, read_sysfs_attr, UsbDeviceInfo};
use crate::orchestrator::DeviceEvent;

/// 判断 udev devtype 是否为 USB interface。
pub fn is_usb_interface_devtype(devtype: &str) -> bool {
    devtype == "usb_interface"
}

/// 判断 USB udev 事件是否需要转发给编排器。
///
/// 插拔事件必须是接口级事件，因为设备分类和生命周期都按 interface
/// 归并到同一个父 USB 设备。
pub fn should_forward_usb_event(action: &str, devtype: &str) -> bool {
    matches!(action, "add" | "remove") && is_usb_interface_devtype(devtype)
}

/// 将已解析的 USB 设备信息转换成项目内部事件。
///
/// 该函数只表达 interface 类型到 `DeviceEvent` 的转换，不做业务准入。
pub fn device_event_from_info(info: UsbDeviceInfo) -> DeviceEvent {
    match info.device_type {
        common::types::DeviceType::Storage => DeviceEvent::StorageAdded(info),
        common::types::DeviceType::Keyboard => DeviceEvent::KeyboardAdded(info),
        common::types::DeviceType::Mouse => DeviceEvent::MouseAdded(info),
        _ => DeviceEvent::UnsupportedAdded(info, "不支持的设备类型".into()),
    }
}

pub(crate) fn parse_device_info_from_syspath(syspath: &Path) -> Option<UsbDeviceInfo> {
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
        find_block_device_with_retry(syspath)
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

fn find_block_device(interface_path: &Path) -> Option<(String, i64)> {
    let entries = std::fs::read_dir(interface_path).ok()?;
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

fn find_block_device_with_retry(interface_path: &Path) -> Option<(String, i64)> {
    for _ in 0..20 {
        if let Some(found) = find_block_device(interface_path) {
            return Some(found);
        }
        thread::sleep(Duration::from_millis(100));
    }
    None
}

fn find_block_in_host(host_path: &Path) -> Option<(String, i64)> {
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

fn find_sd_device(block_path: &Path) -> Option<(String, i64)> {
    let entries = std::fs::read_dir(block_path).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("sd") {
            continue;
        }
        let path = entry.path();

        if path.join("dev").exists() {
            let size = std::fs::read_to_string(path.join("size"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);
            let dev = format!("/dev/{}", name);
            let is_partition = std::fs::read_to_string(path.join("partition"))
                .ok()
                .map(|s| s.trim() != "0")
                .unwrap_or_else(|| name.chars().any(|c| c.is_ascii_digit() && c != '0'));
            if is_partition {
                return Some((dev, size as i64 * 512));
            }
        }

        if path.is_dir() {
            if let Some(result) = find_sd_device(&path) {
                return Some(result);
            }
        }
    }
    None
}
