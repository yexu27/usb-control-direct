//! USB 描述符解析与设备分类。
//!
//! 通过读取 sysfs 属性识别 USB 设备类型。

use std::path::Path;

use common::types::DeviceType;

/// USB 接口类常量。
const CLASS_MASS_STORAGE: u8 = 0x08;
const CLASS_HID: u8 = 0x03;
const SUBCLASS_BOOT: u8 = 0x01;
const PROTOCOL_KEYBOARD: u8 = 0x01;
const PROTOCOL_MOUSE: u8 = 0x02;

/// 解析后的 USB 设备信息。
#[derive(Debug, Clone)]
pub struct UsbDeviceInfo {
    /// 设备路径（sysfs 路径或块设备路径）。
    pub sys_path: String,
    /// 块设备路径（如 /dev/sda1）。
    pub dev_path: Option<String>,
    /// 序列号。
    pub serial_number: String,
    /// VID（十六进制字符串）。
    pub vid: String,
    /// PID（十六进制字符串）。
    pub pid: String,
    /// 设备名称（产品字符串）。
    pub device_name: String,
    /// 设备类型。
    pub device_type: DeviceType,
    /// 接口类。
    pub interface_class: u8,
    /// 接口子类。
    pub interface_subclass: u8,
    /// 接口协议。
    pub interface_protocol: u8,
    /// 容量（字节）。
    pub capacity_bytes: Option<i64>,
}

/// 接口类型字符串（协议层使用）。
pub fn interface_type_str(info: &UsbDeviceInfo) -> &'static str {
    match info.device_type {
        DeviceType::Storage => "mass_storage",
        DeviceType::Keyboard => "hid_keyboard",
        DeviceType::Mouse => "hid_mouse",
        DeviceType::Unsupported => "unsupported",
        DeviceType::Unknown => "unknown",
    }
}

/// 准入状态字符串（协议层使用）。
///
/// 参数:
///   - `info`: 设备信息。
///   - `is_in_whitelist`: 是否在白名单中。
///   - `is_spoof`: 是否疑似伪装。
pub fn admission_status_str(info: &UsbDeviceInfo, is_in_whitelist: bool, is_spoof: bool) -> &'static str {
    if is_spoof {
        return "spoof_suspected";
    }
    match info.device_type {
        DeviceType::Storage => {
            if is_in_whitelist {
                "blocked"
            } else {
                "addable"
            }
        }
        DeviceType::Unsupported | DeviceType::Unknown => "unsupported",
        DeviceType::Keyboard | DeviceType::Mouse => "unsupported",
    }
}

/// 根据接口类/子类/协议号分类设备类型。
pub fn classify_device(
    interface_class: u8,
    interface_subclass: u8,
    interface_protocol: u8,
) -> DeviceType {
    match (interface_class, interface_subclass, interface_protocol) {
        (CLASS_MASS_STORAGE, _, _) => DeviceType::Storage,
        (CLASS_HID, SUBCLASS_BOOT, PROTOCOL_KEYBOARD) => DeviceType::Keyboard,
        (CLASS_HID, SUBCLASS_BOOT, PROTOCOL_MOUSE) => DeviceType::Mouse,
        (CLASS_HID, _, _) => DeviceType::Unsupported,
        _ => DeviceType::Unknown,
    }
}

/// 检测设备是否疑似伪装。
///
/// 描述符声明的接口类型与实际能力不一致时标记为伪装。
pub fn detect_spoof(info: &UsbDeviceInfo) -> bool {
    match info.device_type {
        DeviceType::Storage => {
            // 声称是存储设备但接口类不是 08
            info.interface_class != CLASS_MASS_STORAGE
        }
        DeviceType::Keyboard => {
            info.interface_class != CLASS_HID
                || info.interface_subclass != SUBCLASS_BOOT
                || info.interface_protocol != PROTOCOL_KEYBOARD
        }
        DeviceType::Mouse => {
            info.interface_class != CLASS_HID
                || info.interface_subclass != SUBCLASS_BOOT
                || info.interface_protocol != PROTOCOL_MOUSE
        }
        _ => false,
    }
}

/// 从 sysfs 属性文件读取值。
pub fn read_sysfs_attr(device_path: &Path, attr: &str) -> Option<String> {
    let path = device_path.join(attr);
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// 解析十六进制字符串为 u8。
pub fn parse_hex_u8(s: &str) -> Option<u8> {
    u8::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}
