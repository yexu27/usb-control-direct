//! USB 设备管理器。
//!
//! 以父设备路径分组管理已连接 USB interface。
//! 准入、白名单和会话启动由 `DeviceOrchestrator` 负责，本模块只维护接口注册表。

use std::collections::HashMap;

use tracing::info;

use crate::descriptor::UsbDeviceInfo;

/// 单个 USB interface 记录。
#[derive(Debug, Clone)]
pub struct InterfaceRecord {
    pub info: UsbDeviceInfo,
    pub session_key: String,
    pub connected_at: i64,
}

/// 父 USB 设备记录。
#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub parent_path: String,
    interfaces: HashMap<String, InterfaceRecord>,
    pub connected_at: i64,
}

impl DeviceRecord {
    pub fn interface_count(&self) -> usize {
        self.interfaces.len()
    }

    pub fn interface(&self, sys_path: &str) -> Option<&InterfaceRecord> {
        self.interfaces.get(sys_path)
    }

    pub fn interfaces(&self) -> impl Iterator<Item = &InterfaceRecord> {
        self.interfaces.values()
    }
}

/// interface add 注册结果。
#[derive(Debug, Clone)]
pub struct InterfaceAddResult {
    pub parent_path: String,
    pub session_key: String,
    pub is_new_parent: bool,
    pub is_new_interface: bool,
}

/// interface remove 注册结果。
#[derive(Debug, Clone)]
pub struct InterfaceRemoveResult {
    pub parent_path: String,
    pub session_key: String,
    pub interface: InterfaceRecord,
    pub parent_removed: bool,
}

/// USB 设备管理器。
///
/// 维护已连接 USB interface 注册表，按 parent path 分组，但所有生命周期结果都按
/// interface sys_path 返回。
pub struct DeviceManager {
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
        Self {
            records: HashMap::new(),
        }
    }

    /// 添加 USB interface。
    pub fn add_interface(&mut self, info: UsbDeviceInfo) -> InterfaceAddResult {
        let parent_path = parent_device_path(&info.sys_path);
        let session_key = interface_session_key(&info.sys_path);
        let now = common::time::now_unix();
        let is_new_parent = !self.records.contains_key(&parent_path);

        let record = self.records.entry(parent_path.clone()).or_insert_with(|| {
            info!(
                parent = %parent_path,
                dev = %info.device_name,
                "USB 父设备注册"
            );
            DeviceRecord {
                parent_path: parent_path.clone(),
                interfaces: HashMap::new(),
                connected_at: now,
            }
        });

        let is_new_interface = !record.interfaces.contains_key(&info.sys_path);
        if is_new_interface {
            info!(
                parent = %parent_path,
                sys_path = %info.sys_path,
                dev = %info.device_name,
                type = ?info.device_type,
                "USB 接口注册"
            );
            record.interfaces.insert(
                info.sys_path.clone(),
                InterfaceRecord {
                    info,
                    session_key: session_key.clone(),
                    connected_at: now,
                },
            );
        }

        InterfaceAddResult {
            parent_path,
            session_key,
            is_new_parent,
            is_new_interface,
        }
    }

    /// 移除 USB interface。
    pub fn remove_interface(&mut self, sys_path: &str) -> Option<InterfaceRemoveResult> {
        let parent_path = parent_device_path(sys_path);
        let record = self.records.get_mut(&parent_path)?;
        let interface = record.interfaces.remove(sys_path)?;
        let session_key = interface.session_key.clone();
        let parent_removed = record.interfaces.is_empty();

        if parent_removed {
            self.records.remove(&parent_path);
            info!(parent = %parent_path, "USB 父设备移除");
        } else {
            info!(
                parent = %parent_path,
                remaining = record.interfaces.len(),
                "USB 接口移除，父设备仍有其它接口"
            );
        }

        Some(InterfaceRemoveResult {
            parent_path,
            session_key,
            interface,
            parent_removed,
        })
    }

    /// 根据父设备路径查询设备。
    pub fn get_by_parent(&self, parent_path: &str) -> Option<&DeviceRecord> {
        self.records.get(parent_path)
    }

    /// 根据序列号查找已连接接口信息。
    pub fn connected_device_by_serial(&self, serial: &str) -> Option<&UsbDeviceInfo> {
        self.records
            .values()
            .flat_map(|record| record.interfaces.values())
            .find(|interface| interface.info.serial_number == serial)
            .map(|interface| &interface.info)
    }

    /// 列出所有已连接父设备记录。
    pub fn list_all(&self) -> Vec<&DeviceRecord> {
        self.records.values().collect()
    }

    /// 已连接父设备数量。
    pub fn count(&self) -> usize {
        self.records.len()
    }
}

/// 生成接口级会话键。
pub fn interface_session_key(sys_path: &str) -> String {
    sys_path.to_string()
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
