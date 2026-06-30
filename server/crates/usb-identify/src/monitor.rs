//! USB 设备管理器。
//!
//! 以父设备路径（去掉 :N.M 接口后缀）为唯一键管理已连接 USB 设备，
//! 合并同一物理设备的多个接口到一条记录。

use std::collections::HashMap;

use common::types::DeviceType;
use tracing::{info, warn};

use crate::descriptor::UsbDeviceInfo;

/// 大容量存储设备接口类。
const CLASS_MASS_STORAGE: u8 = 0x08;
/// HID 接口类。
const CLASS_HID: u8 = 0x03;

/// 设备记录。
#[derive(Debug, Clone)]
pub struct DeviceRecord {
    /// 设备信息（首个接口）。
    pub info: UsbDeviceInfo,
    /// 该设备的所有接口 sys_path。
    pub interfaces: Vec<String>,
    /// 各接口的 class 值（与 interfaces 一一对应）。
    pub interface_classes: Vec<u8>,
    /// 首次连接时间（Unix 秒）。
    pub connected_at: i64,
}

impl DeviceRecord {
    /// 是否为存储类设备。
    pub fn is_storage(&self) -> bool {
        self.info.device_type == DeviceType::Storage
    }

    /// 是否疑似 BadUSB 设备（同时具备 Storage 和 HID 接口）。
    pub fn is_badusb(&self) -> bool {
        let has_storage = self.interface_classes.contains(&CLASS_MASS_STORAGE);
        let has_hid = self.interface_classes.contains(&CLASS_HID);
        has_storage && has_hid
    }
}

/// USB 设备管理器。
///
/// 维护已连接设备的注册表，增删查均以父设备路径为键。
pub struct DeviceManager {
    /// 父设备路径 → DeviceRecord。
    records: HashMap<String, DeviceRecord>,
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceManager {
    /// 创建设备管理器。
    pub fn new() -> Self {
        DeviceManager {
            records: HashMap::new(),
        }
    }

    /// 添加设备。按父设备路径归并同物理设备的多个接口。
    ///
    /// 如果父设备路径已存在，追加接口并保留首次记录的 info。
    /// 如果不存在，创建新记录。
    /// 多接口设备同时具备 Storage + HID 时输出 BadUSB 告警。
    pub fn add(&mut self, info: UsbDeviceInfo) {
        let parent_path = parent_device_path(&info.sys_path);
        let class = info.interface_class;
        let now = common::time::now_unix();

        if let Some(record) = self.records.get_mut(&parent_path) {
            if !record.interfaces.contains(&info.sys_path) {
                record.interfaces.push(info.sys_path.clone());
                record.interface_classes.push(class);
            }
            info!(
                parent = %parent_path,
                dev = %info.device_name,
                interfaces = ?record.interfaces,
                "设备接口追加"
            );
            if record.is_badusb() {
                warn!(
                    parent = %parent_path,
                    dev = %info.device_name,
                    "检测到 BadUSB 伪装设备（同时具备 Storage 和 HID 接口）"
                );
            }
        } else {
            info!(
                parent = %parent_path,
                dev = %info.device_name,
                type = ?info.device_type,
                "设备注册"
            );
            let sys_path = info.sys_path.clone();
            self.records.insert(
                parent_path,
                DeviceRecord {
                    info,
                    interfaces: vec![sys_path],
                    interface_classes: vec![class],
                    connected_at: now,
                },
            );
        }
    }

    /// 移除设备的指定接口。
    ///
    /// 返回被移除的设备记录（当且仅当该设备的所有接口都已移除时）。
    pub fn remove_interface(&mut self, sys_path: &str) -> Option<DeviceRecord> {
        let parent_path = parent_device_path(sys_path);

        let record = self.records.get_mut(&parent_path)?;
        if let Some(pos) = record.interfaces.iter().position(|iface| iface == sys_path) {
            record.interfaces.remove(pos);
            if pos < record.interface_classes.len() {
                record.interface_classes.remove(pos);
            }
        }

        if record.interfaces.is_empty() {
            let removed = self.records.remove(&parent_path);
            info!(
                parent = %parent_path,
                dev = ?removed.as_ref().map(|r| &r.info.device_name),
                "设备移除"
            );
            removed
        } else {
            info!(
                parent = %parent_path,
                remaining = ?record.interfaces,
                "设备接口移除（仍有其他接口）"
            );
            None
        }
    }

    /// 根据父设备路径查询设备。
    pub fn get_by_parent(&self, parent_path: &str) -> Option<&DeviceRecord> {
        self.records.get(parent_path)
    }

    /// 根据序列号查找已连接设备信息。
    pub fn connected_device_by_serial(&self, serial: &str) -> Option<&UsbDeviceInfo> {
        self.records.values().find(|r| r.info.serial_number == serial).map(|r| &r.info)
    }

    /// 根据序列号判断设备是否疑似 BadUSB（同时具备 Storage 和 HID 接口）。
    pub fn is_badusb_by_serial(&self, serial: &str) -> bool {
        self.records
            .values()
            .find(|r| r.info.serial_number == serial)
            .map(|r| r.is_badusb())
            .unwrap_or(false)
    }

    /// 列出所有已连接设备。
    pub fn list_all(&self) -> Vec<&DeviceRecord> {
        self.records.values().collect()
    }

    /// 已连接设备数量。
    pub fn count(&self) -> usize {
        self.records.len()
    }
}

/// 从接口 sys_path 提取父设备路径。
///
/// 规则：去掉最后一个 `:N.M` 后缀。真实 sysfs 接口路径形如
/// `/sys/.../2-1.1/2-1.1:1.0`，父设备路径应为 `/sys/.../2-1.1`。
/// 简化路径 `/sys/.../2-1.1:1.0` 仍返回 `/sys/.../2-1.1`。
pub fn parent_device_path(sys_path: &str) -> String {
    match sys_path.rsplit_once(':') {
        Some((without_suffix, suffix)) if suffix.chars().all(|c| c.is_ascii_digit() || c == '.') => {
            let path = std::path::Path::new(without_suffix);
            if let (Some(name), Some(parent)) = (path.file_name(), path.parent()) {
                if parent.file_name() == Some(name) {
                    return parent.to_string_lossy().to_string();
                }
            }
            without_suffix.to_string()
        }
        _ => sys_path.to_string(),
    }
}
