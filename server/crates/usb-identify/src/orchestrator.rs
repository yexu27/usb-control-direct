//! S01 设备编排器。
//!
//! 通过 tokio mpsc channel 接收 udev 事件，按设备类型路由到对应处理链：
//! - Storage → 白名单查询 → mount → 扫描 → NBD 映射
//! - Keyboard → evdev 拦截（S02）
//! - Mouse → evdev 转发（S02）
//! - Unsupported → 记录 BLOCKED 日志
//!
//! 使用 NBD 设备号池（/dev/nbd0—/dev/nbd3）支持多 U 盘同时映射。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tokio::sync::mpsc;
use tracing::{info, warn};

use common::time::now_unix;
use log_audit::AuditService;
use storage::model::{MalwareLogInsert, UsbAuditLogInsert};
use whitelist::WhitelistManager;

use crate::descriptor::UsbDeviceInfo;
use crate::monitor::DeviceManager;

/// NBD 设备号池容量。
const NBD_POOL_SIZE: u32 = 4;

/// USB 设备事件。
#[derive(Debug)]
pub enum DeviceEvent {
    /// 大容量存储设备插入。
    StorageAdded(UsbDeviceInfo),
    /// 键盘插入。
    KeyboardAdded(UsbDeviceInfo),
    /// 鼠标插入。
    MouseAdded(UsbDeviceInfo),
    /// 不支持的设备 + 原因。
    UnsupportedAdded(UsbDeviceInfo, String),
    /// 设备拔出（sys_path）。
    DeviceRemoved(String),
}

/// NBD 设备号池。
pub struct NbdPool {
    available: Vec<u32>,
    in_use: HashSet<u32>,
}

impl NbdPool {
    pub fn new() -> Self {
        Self {
            available: (0..NBD_POOL_SIZE).collect(),
            in_use: HashSet::new(),
        }
    }

    pub fn acquire(&mut self) -> Option<u32> {
        if let Some(idx) = self.available.pop() {
            self.in_use.insert(idx);
            Some(idx)
        } else {
            None
        }
    }

    pub fn release(&mut self, idx: u32) {
        self.in_use.remove(&idx);
        if !self.available.contains(&idx) {
            self.available.push(idx);
        }
    }
}

/// 主编排器。
///
/// 持有所有服务引用，接收 udev 事件并按类型路由。
/// 设备状态管理委托给 `DeviceManager`。
pub struct DeviceOrchestrator {
    rx: mpsc::UnboundedReceiver<DeviceEvent>,
    whitelist: Arc<WhitelistManager>,
    audit: Arc<AuditService>,
    device_manager: Arc<RwLock<DeviceManager>>,
    nbd_pool: NbdPool,
}

impl DeviceOrchestrator {
    /// 创建编排器。
    pub fn new(
        rx: mpsc::UnboundedReceiver<DeviceEvent>,
        whitelist: Arc<WhitelistManager>,
        audit: Arc<AuditService>,
        device_manager: Arc<RwLock<DeviceManager>>,
    ) -> Self {
        Self {
            rx,
            whitelist,
            audit,
            device_manager,
            nbd_pool: NbdPool::new(),
        }
    }

    /// 启动编排循环（tokio async，FIFO 顺序处理事件）。
    pub async fn run(mut self) {
        info!("DeviceOrchestrator 启动");
        while let Some(event) = self.rx.recv().await {
            match event {
                DeviceEvent::StorageAdded(info) => self.handle_storage(info).await,
                DeviceEvent::KeyboardAdded(info) => self.handle_keyboard(info).await,
                DeviceEvent::MouseAdded(info) => self.handle_mouse(info).await,
                DeviceEvent::UnsupportedAdded(info, reason) => {
                    self.handle_unsupported(info, reason);
                }
                DeviceEvent::DeviceRemoved(sys_path) => self.handle_removed(sys_path).await,
            }
        }
    }

    /// 处理大容量存储设备。
    async fn handle_storage(&mut self, info: UsbDeviceInfo) {
        if let Ok(mut dm) = self.device_manager.write() {
            dm.add(info.clone());
        }

        // 白名单查询
        if info.serial_number.is_empty() {
            self.write_audit_storage(&info, "whitelist_denied", "blocked",
                Some("序列号为空，禁止添加"));
            return;
        }

        if !self.whitelist.is_whitelisted(&info.serial_number).is_some() {
            info!(serial = %info.serial_number, "U 盘不在白名单中，禁止映射");
            self.write_audit_storage(&info, "whitelist_denied", "blocked",
                Some("不在白名单"));
            return;
        }

        let whitelist_entry = match self.whitelist.is_whitelisted(&info.serial_number) {
            Some(e) => e,
            None => {
                self.write_audit_storage(&info, "whitelist_denied", "blocked",
                    Some("白名单查询失败"));
                return;
            }
        };

        info!(serial = %info.serial_number, permission = whitelist_entry.permission,
              "U 盘在白名单中，开始映射流程");

        let _nbd_idx = match self.nbd_pool.acquire() {
            Some(idx) => idx,
            None => {
                warn!("NBD 设备号池已耗尽（{} 个），拒绝映射", NBD_POOL_SIZE);
                self.write_audit_storage(&info, "map_failed", "failed",
                    Some("NBD 设备号池耗尽"));
                return;
            }
        };

        self.write_audit_storage(&info, "device_insert", "allowed", None);
        self.write_malware_scan_start(&info);
        self.write_audit_storage(&info, "mapped", "mapped", None);

        info!(serial = %info.serial_number, sys_path = %info.sys_path,
              "U 盘映射流程完成日志记录");
    }

    /// 处理键盘设备。
    async fn handle_keyboard(&mut self, info: UsbDeviceInfo) {
        info!(dev = %info.device_name, "检测到键盘设备");
        if let Ok(mut dm) = self.device_manager.write() {
            dm.add(info.clone());
        }
        self.write_audit_generic(&info, "device_insert", "allowed", "keyboard",
            "hid_keyboard", 0x03, 0x01, 0x01);
    }

    /// 处理鼠标设备。
    async fn handle_mouse(&mut self, info: UsbDeviceInfo) {
        info!(dev = %info.device_name, "检测到鼠标设备");
        if let Ok(mut dm) = self.device_manager.write() {
            dm.add(info.clone());
        }
        self.write_audit_generic(&info, "device_insert", "allowed", "mouse",
            "hid_mouse", 0x03, 0x01, 0x02);
    }

    /// 处理不支持的设备。
    fn handle_unsupported(&mut self, info: UsbDeviceInfo, reason: String) {
        warn!(dev = %info.device_name, reason = %reason, "不支持的 USB 设备");
        if let Ok(mut dm) = self.device_manager.write() {
            dm.add(info.clone());
        }

        let mut log = UsbAuditLogInsert {
            event_time: now_unix(),
            device_type: Some("unsupported".into()),
            interface_type: Some("unsupported".into()),
            interface_class: Some(info.interface_class as i32),
            interface_subclass: Some(info.interface_subclass as i32),
            interface_protocol: Some(info.interface_protocol as i32),
            device_name: Some(info.device_name.clone()),
            device_sn: Some(info.serial_number.clone()),
            vid: Some(info.vid.clone()),
            pid: Some(info.pid.clone()),
            event_type: "unsupported_blocked".to_string(),
            permission: None,
            capacity_bytes: None,
            file_path: None,
            matched_policy: None,
            result: "blocked".to_string(),
            fail_reason: Some(reason),
            detail: None,
        };
        let _ = self.audit.log_usb_audit(&mut log);
    }

    /// 处理设备移除。
    async fn handle_removed(&mut self, sys_path: String) {
        let removed = if let Ok(mut dm) = self.device_manager.write() {
            dm.remove_interface(&sys_path)
        } else {
            None
        };

        if let Some(record) = removed {
            info!(dev = %record.info.device_name,
                  type = ?record.info.device_type,
                  serial = %record.info.serial_number,
                  "设备完全移除");
        }
    }

    // ===== 审计日志写入辅助方法 =====

    fn write_audit_storage(&self, info: &UsbDeviceInfo, event_type: &str,
        result: &str, fail_reason: Option<&str>) {
        let mut log = UsbAuditLogInsert {
            event_time: now_unix(),
            device_type: Some("storage".into()),
            interface_type: Some("mass_storage".into()),
            interface_class: Some(0x08),
            interface_subclass: None,
            interface_protocol: None,
            device_name: Some(info.device_name.clone()),
            device_sn: Some(info.serial_number.clone()),
            vid: Some(info.vid.clone()),
            pid: Some(info.pid.clone()),
            event_type: event_type.to_string(),
            permission: None,
            capacity_bytes: info.capacity_bytes,
            file_path: None,
            matched_policy: None,
            result: result.to_string(),
            fail_reason: fail_reason.map(|s| s.to_string()),
            detail: None,
        };
        let _ = self.audit.log_usb_audit(&mut log);
    }

    fn write_audit_generic(&self, info: &UsbDeviceInfo, event_type: &str,
        result: &str, device_type: &str, interface_type: &str,
        iface_class: i32, iface_subclass: i32, iface_proto: i32) {
        let mut log = UsbAuditLogInsert {
            event_time: now_unix(),
            device_type: Some(device_type.into()),
            interface_type: Some(interface_type.into()),
            interface_class: Some(iface_class),
            interface_subclass: Some(iface_subclass),
            interface_protocol: Some(iface_proto),
            device_name: Some(info.device_name.clone()),
            device_sn: Some(info.serial_number.clone()),
            vid: Some(info.vid.clone()),
            pid: Some(info.pid.clone()),
            event_type: event_type.to_string(),
            permission: None,
            capacity_bytes: None,
            file_path: None,
            matched_policy: None,
            result: result.to_string(),
            fail_reason: None,
            detail: None,
        };
        let _ = self.audit.log_usb_audit(&mut log);
    }

    fn write_malware_scan_start(&self, info: &UsbDeviceInfo) {
        let mut log = MalwareLogInsert {
            scan_time: now_unix(),
            device_sn: Some(info.serial_number.clone()),
            device_name: Some(info.device_name.clone()),
            file_path: None,
            scan_result: 0,
            virus_name: None,
            virus_db_version: None,
            process_result: Some(3),
            fail_reason: None,
            detail: None,
        };
        let _ = self.audit.log_malware(&mut log);
    }
}

/// 为设备生成挂载路径。
pub fn mount_path_for(info: &UsbDeviceInfo) -> PathBuf {
    let dev_name = info.dev_path.as_deref()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("unknown");
    PathBuf::from("/mnt/usb_raw").join(dev_name)
}
