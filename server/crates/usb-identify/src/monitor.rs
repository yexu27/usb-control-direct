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

    /// 处理设备插入事件。
    pub fn handle_device_added(
        &mut self,
        info: UsbDeviceInfo,
    ) -> Result<(), UsbIdentifyError> {
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
    pub fn handle_device_removed(
        &mut self,
        sys_path: &str,
    ) -> Option<DeviceSession> {
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
