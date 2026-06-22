//! udev 监听与设备管理器。
//!
//! DeviceManager 是 S01 的核心编排器，管理所有已连接 USB 设备的生命周期。
//! udev 监听和事件循环的真实实现需要 udev crate（Linux only），
//! 本模块定义核心管理逻辑，udev 交互通过 trait 抽象。

use std::collections::HashMap;

use common::types::UsbDeviceState;
use tracing::{info, warn};

use crate::descriptor::UsbDeviceInfo;
use crate::error::UsbIdentifyError;
use crate::state_machine::{UsbDeviceEvent, UsbStateMachine};

/// 设备会话：跟踪单个 USB 设备的生命周期。
pub struct DeviceSession {
    /// 设备信息。
    pub info: UsbDeviceInfo,
    /// 状态机。
    pub state_machine: UsbStateMachine,
    /// 挂载路径（mount 成功后设置）。
    pub mount_point: Option<String>,
}

/// 设备管理器：管理所有已连接 USB 设备。
pub struct DeviceManager {
    /// 设备映射：sys_path → DeviceSession。
    sessions: HashMap<String, DeviceSession>,
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
            sessions: HashMap::new(),
        }
    }

    /// 获取当前所有连接设备信息（供 CMD_GET_CONNECTED_DEVICES 使用）。
    pub fn connected_devices(&self) -> Vec<&DeviceSession> {
        self.sessions
            .values()
            .filter(|s| s.state_machine.state() != UsbDeviceState::Removed)
            .collect()
    }

    /// 按序列号获取当前仍连接的设备会话。
    pub fn connected_device_by_serial(&self, serial_number: &str) -> Option<&DeviceSession> {
        self.connected_devices()
            .into_iter()
            .find(|session| session.info.serial_number == serial_number)
    }

    /// 处理设备插入事件。
    pub fn handle_device_added(&mut self, info: UsbDeviceInfo) -> Result<(), UsbIdentifyError> {
        let key = info.sys_path.clone();
        let mut sm = UsbStateMachine::new();
        sm.transition(UsbDeviceEvent::Inserted)?;

        self.sessions.insert(
            key,
            DeviceSession {
                info,
                state_machine: sm,
                mount_point: None,
            },
        );

        Ok(())
    }

    /// 处理设备拔出事件。
    pub fn handle_device_removed(&mut self, sys_path: &str) -> Option<DeviceSession> {
        if let Some(mut session) = self.sessions.remove(sys_path) {
            let _ = session.state_machine.transition(UsbDeviceEvent::Unplug);
            info!(sys_path = sys_path, "设备拔出，会话已清理");
            Some(session)
        } else {
            warn!(sys_path = sys_path, "设备拔出但无对应会话");
            None
        }
    }

    /// 获取指定设备的会话（可变引用）。
    pub fn get_session_mut(&mut self, sys_path: &str) -> Option<&mut DeviceSession> {
        self.sessions.get_mut(sys_path)
    }

    /// 获取指定设备的会话（不可变引用）。
    pub fn get_session(&self, sys_path: &str) -> Option<&DeviceSession> {
        self.sessions.get(sys_path)
    }

    /// 会话数量。
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::types::DeviceType;

    fn storage_info(sys_path: &str, serial_number: &str) -> UsbDeviceInfo {
        UsbDeviceInfo {
            sys_path: sys_path.to_string(),
            dev_path: Some("/dev/sda1".to_string()),
            serial_number: serial_number.to_string(),
            vid: "0951".to_string(),
            pid: "1666".to_string(),
            device_name: "USB Disk".to_string(),
            device_type: DeviceType::Storage,
            interface_class: 0x08,
            interface_subclass: 0x06,
            interface_protocol: 0x50,
            capacity_bytes: Some(32 * 1024 * 1024 * 1024),
        }
    }

    #[test]
    fn connected_device_by_serial_tracks_current_sessions() {
        let mut manager = DeviceManager::new();
        manager
            .handle_device_added(storage_info("/sys/a", "SN-A"))
            .unwrap();
        manager
            .handle_device_added(storage_info("/sys/b", "SN-B"))
            .unwrap();

        assert_eq!(
            manager
                .connected_device_by_serial("SN-B")
                .unwrap()
                .info
                .sys_path,
            "/sys/b"
        );

        manager.handle_device_removed("/sys/b");
        assert!(manager.connected_device_by_serial("SN-B").is_none());
        assert!(manager.connected_device_by_serial("SN-A").is_some());
    }
}
