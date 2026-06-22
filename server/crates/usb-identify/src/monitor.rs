//! USB 设备管理器。
//!
//! 以父设备路径（去掉 :N.M 接口后缀）为唯一键管理已连接 USB 设备，
//! 合并同一物理设备的多个接口到一条记录。

use std::collections::HashMap;

use common::types::DeviceType;
use tracing::info;

use crate::descriptor::UsbDeviceInfo;

/// 设备记录。
#[derive(Debug, Clone)]
pub struct DeviceRecord {
    /// 设备信息。
    pub info: UsbDeviceInfo,
    /// 该设备的所有接口 sys_path。
    pub interfaces: Vec<String>,
    /// 首次连接时间（Unix 秒）。
    pub connected_at: i64,
}

impl DeviceRecord {
    /// 是否为存储类设备。
    pub fn is_storage(&self) -> bool {
        self.info.device_type == DeviceType::Storage
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
    pub fn add(&mut self, info: UsbDeviceInfo) {
        let parent_path = parent_device_path(&info.sys_path);
        let now = common::time::now_unix();

        if let Some(record) = self.records.get_mut(&parent_path) {
            if !record.interfaces.contains(&info.sys_path) {
                record.interfaces.push(info.sys_path.clone());
            }
            info!(
                parent = %parent_path,
                dev = %info.device_name,
                interfaces = ?record.interfaces,
                "设备接口追加"
            );
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
        record.interfaces.retain(|iface| iface != sys_path);

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

    /// 列出所有已连接设备。
    pub fn list_all(&self) -> Vec<&DeviceRecord> {
        self.records.values().collect()
    }

    /// 已连接设备数量。
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.records.len()
    }
}

/// 从接口 sys_path 提取父设备路径。
///
/// 规则：去掉最后一个 `:N.M` 后缀。
/// 示例: `/sys/.../2-1.1:1.0` → `/sys/.../2-1.1`
pub fn parent_device_path(sys_path: &str) -> String {
    match sys_path.rsplit_once(':') {
        Some((parent, suffix)) if suffix.chars().all(|c| c.is_ascii_digit() || c == '.') => {
            parent.to_string()
        }
        _ => sys_path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::types::DeviceType;

    fn make_info(sys_path: &str, device_type: DeviceType, name: &str) -> UsbDeviceInfo {
        UsbDeviceInfo {
            sys_path: sys_path.into(),
            dev_path: None,
            serial_number: name.into(),
            vid: "0000".into(),
            pid: "0000".into(),
            device_name: name.into(),
            device_type,
            interface_class: 0,
            interface_subclass: 0,
            interface_protocol: 0,
            capacity_bytes: None,
        }
    }

    #[test]
    fn parent_path_strips_interface_suffix() {
        assert_eq!(
            parent_device_path("/sys/.../2-1.1:1.0"),
            "/sys/.../2-1.1"
        );
        assert_eq!(
            parent_device_path("/sys/.../2-1.2:1.0"),
            "/sys/.../2-1.2"
        );
    }

    #[test]
    fn parent_path_preserves_non_interface() {
        assert_eq!(
            parent_device_path("/sys/.../usb2/2-1"),
            "/sys/.../usb2/2-1"
        );
    }

    #[test]
    fn multi_interface_merges_to_one_record() {
        let mut dm = DeviceManager::new();

        dm.add(make_info("/sys/.../2-1.1:1.0", DeviceType::Keyboard, "TestKB"));
        dm.add(make_info("/sys/.../2-1.1:1.1", DeviceType::Unsupported, "TestKB-Media"));

        assert_eq!(dm.count(), 1);
        let record = dm.get_by_parent("/sys/.../2-1.1").unwrap();
        assert_eq!(record.interfaces.len(), 2);
        assert_eq!(record.info.device_type, DeviceType::Keyboard);
    }

    #[test]
    fn remove_last_interface_removes_record() {
        let mut dm = DeviceManager::new();

        dm.add(make_info("/sys/.../2-1.2:1.0", DeviceType::Storage, "TestUSB"));
        assert_eq!(dm.count(), 1);

        let removed = dm.remove_interface("/sys/.../2-1.2:1.0");
        assert!(removed.is_some());
        assert_eq!(dm.count(), 0);
    }

    #[test]
    fn remove_one_interface_keeps_record_with_others() {
        let mut dm = DeviceManager::new();

        dm.add(make_info("/sys/.../2-1.1:1.0", DeviceType::Keyboard, "TestKB"));
        dm.add(make_info("/sys/.../2-1.1:1.1", DeviceType::Unsupported, "TestKB-Media"));
        assert_eq!(dm.count(), 1);

        let removed = dm.remove_interface("/sys/.../2-1.1:1.0");
        assert!(removed.is_none());
        assert_eq!(dm.count(), 1);

        let removed = dm.remove_interface("/sys/.../2-1.1:1.1");
        assert!(removed.is_some());
        assert_eq!(dm.count(), 0);
    }

    #[test]
    fn remove_unknown_path_returns_none() {
        let mut dm = DeviceManager::new();
        let removed = dm.remove_interface("/sys/.../nonexistent:1.0");
        assert!(removed.is_none());
    }

    #[test]
    fn list_all_returns_all_records() {
        let mut dm = DeviceManager::new();

        dm.add(make_info("/sys/.../2-1.1:1.0", DeviceType::Keyboard, "KB"));
        dm.add(make_info("/sys/.../2-1.2:1.0", DeviceType::Storage, "USB"));
        dm.add(make_info("/sys/.../2-1.3:1.0", DeviceType::Mouse, "MS"));

        assert_eq!(dm.list_all().len(), 3);
    }
}
