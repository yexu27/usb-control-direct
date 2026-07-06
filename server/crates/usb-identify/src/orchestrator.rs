//! S01 设备编排器。
//!
//! 通过 tokio mpsc channel 接收 udev 事件，按设备类型路由到对应处理链：
//! - Storage -> 白名单查询 -> storage session 路由
//! - Keyboard -> evdev 拦截（S02）
//! - Mouse -> evdev 转发（S02）
//! - Unsupported -> 记录 BLOCKED 日志
//!
//! 热插拔生命周期按 USB interface 事件归并到物理父设备路径。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tokio::sync::{mpsc, watch, Mutex};
use tracing::{debug, error, info, warn};

use crate::descriptor::UsbDeviceInfo;
use crate::monitor::{DeviceManager, DeviceRecord};
use crate::traits::{AuthorizedStorageDevice, StorageSessionController};

use common::audit_const::event_type;
use log_audit::AuditService;
use storage::model::UsbAuditLogInsert;
use whitelist::WhitelistManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Storage,
    Keyboard,
    Mouse,
}

/// 活动设备会话，追踪后台 task 和运行时资源。
struct ActiveSession {
    info: UsbDeviceInfo,
    kind: SessionKind,
    cancel_tx: watch::Sender<bool>,
    audit_detail: String,
}

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
    /// 设备拔出（USB interface sys_path）。
    DeviceRemoved(String),
}

/// 从 USB 接口 sysfs 路径查找对应的 evdev 设备节点。
///
/// 内核为 USB HID 设备创建 input 子设备：
/// /sys/devices/.../2-1.1:1.0/0003:.../input/input3/event3
fn find_evdev_path(usb_iface_syspath: &str) -> Option<PathBuf> {
    use std::fs;

    let iface_dir = std::path::Path::new(usb_iface_syspath);
    if !iface_dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(iface_dir).ok()?;
    for entry in entries.flatten() {
        let input_dir = entry.path().join("input");
        if !input_dir.is_dir() {
            continue;
        }
        let input_entries = fs::read_dir(&input_dir).ok()?;
        for input_entry in input_entries.flatten() {
            let name = input_entry.file_name().to_string_lossy().to_string();
            if name.starts_with("input") {
                let event_entries = fs::read_dir(input_entry.path()).ok()?;
                for event_entry in event_entries.flatten() {
                    let event_name = event_entry.file_name().to_string_lossy().to_string();
                    if event_name.starts_with("event") {
                        let dev_path = PathBuf::from("/dev/input").join(&event_name);
                        if dev_path.exists() {
                            return Some(dev_path);
                        }
                    }
                }
            }
        }
    }
    None
}

fn find_evdev_path_with_retry(usb_iface_syspath: &str) -> Option<PathBuf> {
    for _ in 0..20 {
        if let Some(path) = find_evdev_path(usb_iface_syspath) {
            return Some(path);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    None
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

    storage_session: Arc<dyn StorageSessionController>,
    hidg_nodes: hid_access::hid_gadget::HidgNodes,

    active_sessions: Arc<Mutex<HashMap<String, ActiveSession>>>,
}

#[derive(Clone)]
pub struct DeviceOrchestratorCleanupHandle {
    storage_session: Arc<dyn StorageSessionController>,
    active_sessions: Arc<Mutex<HashMap<String, ActiveSession>>>,
}

impl DeviceOrchestratorCleanupHandle {
    /// 清理当前所有活动会话，用于服务停止。
    pub async fn shutdown_cleanup(&self, reason: &str) {
        if let Err(e) = self.storage_session.stop_all(reason.to_string()).await {
            warn!(error = %e, reason = %reason, "停服务清理: storage session 清理失败");
        }

        let sessions = {
            let mut active = self.active_sessions.lock().await;
            active
                .drain()
                .map(|(_, session)| session)
                .collect::<Vec<_>>()
        };

        for session in sessions {
            let _ = session.cancel_tx.send(true);
            info!(
                kind = ?session.kind,
                reason = %reason,
                "停服务清理: 会话清理完成"
            );
        }
    }
}

impl DeviceOrchestrator {
    /// 创建编排器。
    pub fn new(
        rx: mpsc::UnboundedReceiver<DeviceEvent>,
        whitelist: Arc<WhitelistManager>,
        audit: Arc<AuditService>,
        device_manager: Arc<RwLock<DeviceManager>>,
        storage_session: Arc<dyn StorageSessionController>,
        hidg_nodes: hid_access::hid_gadget::HidgNodes,
    ) -> Self {
        Self {
            rx,
            whitelist,
            audit,
            device_manager,
            storage_session,
            hidg_nodes,
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn cleanup_handle(&self) -> DeviceOrchestratorCleanupHandle {
        DeviceOrchestratorCleanupHandle {
            storage_session: Arc::clone(&self.storage_session),
            active_sessions: Arc::clone(&self.active_sessions),
        }
    }

    /// 清理当前所有活动会话，用于服务停止。
    pub async fn shutdown_cleanup(&self, reason: &str) {
        self.cleanup_handle().shutdown_cleanup(reason).await;
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

    async fn register_session(&self, parent_path: String, session: ActiveSession) {
        self.active_sessions
            .lock()
            .await
            .insert(parent_path, session);
    }

    fn add_device_record(&self, info: UsbDeviceInfo) -> bool {
        let parent_path = crate::monitor::parent_device_path(&info.sys_path);
        let mut is_new = false;
        if let Ok(mut dm) = self.device_manager.write() {
            is_new = dm.get_by_parent(&parent_path).is_none();
            dm.add(info);
        }
        is_new
    }

    /// 处理大容量存储设备。
    async fn handle_storage(&mut self, info: UsbDeviceInfo) {
        let parent_path = crate::monitor::parent_device_path(&info.sys_path);
        let is_new_device = self.add_device_record(info.clone());
        if !is_new_device {
            debug!(
                parent = %parent_path,
                sys_path = %info.sys_path,
                "Storage 重复接口事件已归并，跳过重复编排"
            );
            return;
        }

        let serial = info.serial_number.clone();
        if serial.is_empty() {
            warn!(dev = %info.device_name, "U 盘序列号为空，跳过");
            return;
        }
        let whitelist_entry = match self.whitelist.is_whitelisted(&serial) {
            Some(e) => e,
            None => {
                debug!(serial = %serial, "U 盘不在白名单中");
                let mut log = build_audit_log(&info, event_type::INSERT_SUCCESS);
                log.detail = Some("未授权设备".into());
                if let Err(e) = self.audit.log_usb_audit(&mut log) {
                    error!(error = %e, "审计日志写入失败");
                }
                let (cancel_tx, _) = watch::channel(false);
                self.register_session(
                    parent_path,
                    ActiveSession {
                        info,
                        kind: SessionKind::Storage,
                        cancel_tx,
                        audit_detail: "未授权设备".into(),
                    },
                )
                .await;
                return;
            }
        };

        debug!(serial = %serial, permission = %whitelist_entry.permission, "U 盘在白名单中");

        let mut log = build_audit_log(&info, event_type::INSERT_SUCCESS);
        log.permission = Some(whitelist_entry.permission);
        log.detail = Some("授权设备".into());
        if let Err(e) = self.audit.log_usb_audit(&mut log) {
            error!(error = %e, "审计日志写入失败");
        }

        if self.storage_session.has_active_storage().await {
            warn!(
                serial = %serial,
                dev = %info.device_name,
                "当前 RK mass storage LUN 已有活跃 U 盘映射，拒绝同时映射第二个 U 盘"
            );
            return;
        }

        let Some(dev_path) = info.dev_path.clone().filter(|path| !path.is_empty()) else {
            warn!(serial = %serial, dev = %info.device_name, "dev_path 为空，跳过映射");
            return;
        };

        let (cancel_tx, _cancel_rx) = watch::channel(false);
        self.register_session(
            parent_path.clone(),
            ActiveSession {
                info: info.clone(),
                kind: SessionKind::Storage,
                cancel_tx,
                audit_detail: "授权设备".into(),
            },
        )
        .await;

        let device = AuthorizedStorageDevice {
            parent_path,
            sys_path: info.sys_path.clone(),
            dev_path,
            serial_number: serial.clone(),
            vid: info.vid.clone(),
            pid: info.pid.clone(),
            device_name: info.device_name.clone(),
            capacity_bytes: info.capacity_bytes,
            permission: whitelist_entry.permission,
        };

        match self.storage_session.start_authorized_storage(device).await {
            Ok(handle) => {
                info!(
                    serial = %serial,
                    session = %handle.session_id,
                    "Storage session 已接受，等待后台映射完成"
                );
            }
            Err(e) => {
                warn!(
                    serial = %serial,
                    dev = %info.device_name,
                    error = %e,
                    "Storage session 启动失败"
                );
            }
        }
    }

    /// 处理键盘设备。
    async fn handle_keyboard(&mut self, info: UsbDeviceInfo) {
        let parent_path = crate::monitor::parent_device_path(&info.sys_path);
        let is_new_device = self.add_device_record(info.clone());
        if !is_new_device {
            debug!(
                parent = %parent_path,
                sys_path = %info.sys_path,
                "HID 重复接口事件已归并，跳过重复拦截启动"
            );
            return;
        }

        let mut log = build_audit_log(&info, event_type::INSERT_SUCCESS);
        log.detail = Some("键盘".into());
        if let Err(e) = self.audit.log_usb_audit(&mut log) {
            error!(error = %e, "审计日志写入失败");
        }

        let evdev_path = match find_evdev_path_with_retry(&info.sys_path) {
            Some(p) => p,
            None => {
                warn!(dev = %info.device_name, sys_path = %info.sys_path, "键盘: 找不到对应 evdev 设备");
                return;
            }
        };

        let (cancel_tx, cancel_rx) = watch::channel(false);
        self.register_session(
            parent_path,
            ActiveSession {
                info: info.clone(),
                kind: SessionKind::Keyboard,
                cancel_tx,
                audit_detail: "键盘".into(),
            },
        )
        .await;

        let hidg_kb = self.hidg_nodes.keyboard.clone();
        let device_name = info.device_name.clone();

        info!(dev = %device_name, evdev = %evdev_path.display(), "键盘: 启动拦截器");

        tokio::task::spawn_blocking(move || {
            use hid_access::evdev_interceptor::{KeyboardInterceptor, KeyboardRunResult};
            let mut interceptor = KeyboardInterceptor::new(hidg_kb);
            match interceptor.run_with_cancel(&evdev_path, cancel_rx) {
                Ok(KeyboardRunResult::VerifiedThenRemoved) => {
                    info!(dev = %device_name, "键盘拦截器正常退出");
                }
                Ok(KeyboardRunResult::RemovedDuringVerify) => {
                    info!(dev = %device_name, "键盘验证阶段设备拔出");
                }
                Err(e) => {
                    warn!(dev = %device_name, sys_path = %info.sys_path, error = %e, "键盘拦截器异常退出");
                }
            }
        });
    }

    /// 处理鼠标设备。
    async fn handle_mouse(&mut self, info: UsbDeviceInfo) {
        let parent_path = crate::monitor::parent_device_path(&info.sys_path);
        let is_new_device = self.add_device_record(info.clone());
        if !is_new_device {
            debug!(
                parent = %parent_path,
                sys_path = %info.sys_path,
                "HID 重复接口事件已归并，跳过重复转发启动"
            );
            return;
        }

        let mut log = build_audit_log(&info, event_type::INSERT_SUCCESS);
        log.detail = Some("鼠标".into());
        if let Err(e) = self.audit.log_usb_audit(&mut log) {
            error!(error = %e, "审计日志写入失败");
        }

        let evdev_path = match find_evdev_path_with_retry(&info.sys_path) {
            Some(p) => p,
            None => {
                warn!(dev = %info.device_name, sys_path = %info.sys_path, "鼠标: 找不到对应 evdev 设备");
                return;
            }
        };

        let (cancel_tx, cancel_rx) = watch::channel(false);
        self.register_session(
            parent_path,
            ActiveSession {
                info: info.clone(),
                kind: SessionKind::Mouse,
                cancel_tx,
                audit_detail: "鼠标".into(),
            },
        )
        .await;

        let hidg_mouse = self.hidg_nodes.mouse.clone();
        let device_name = info.device_name.clone();

        info!(dev = %device_name, evdev = %evdev_path.display(), "鼠标: 启动转发器");

        tokio::task::spawn_blocking(move || {
            use hid_access::mouse_forwarder::MouseForwarder;
            let mut forwarder = MouseForwarder::new(hidg_mouse);
            match forwarder.run_with_cancel(&evdev_path, cancel_rx) {
                Ok(()) => info!(dev = %device_name, "鼠标转发器正常退出"),
                Err(e) => warn!(dev = %device_name, error = %e, "鼠标转发器异常退出"),
            }
        });
    }

    /// 处理不支持的设备。
    fn handle_unsupported(&mut self, info: UsbDeviceInfo, reason: String) {
        warn!(dev = %info.device_name, reason = %reason, "不支持的 USB 设备");
        let _ = self.add_device_record(info);
    }

    async fn cleanup_session_by_parent(
        &mut self,
        parent_path: &str,
        device_record: &DeviceRecord,
        reason: &str,
    ) {
        let session = self.active_sessions.lock().await.remove(parent_path);

        if let Some(session) = session {
            let _ = session.cancel_tx.send(true);

            if session.kind == SessionKind::Storage {
                if let Err(e) = self
                    .storage_session
                    .stop_by_parent(parent_path.to_string(), reason.to_string())
                    .await
                {
                    warn!(
                        parent = %parent_path,
                        error = %e,
                        reason = %reason,
                        "Storage session 清理失败"
                    );
                }
            }

            let mut log = build_audit_log(&session.info, event_type::DEVICE_REMOVE);
            log.detail = Some(session.audit_detail.clone());
            if let Err(e) = self.audit.log_usb_audit(&mut log) {
                error!(error = %e, reason = %reason, "审计日志写入失败");
            }

            info!(
                dev = %session.info.device_name,
                kind = ?session.kind,
                reason = %reason,
                "设备会话清理完成"
            );
            return;
        }

        let should_audit = matches!(
            device_record.info.device_type,
            common::types::DeviceType::Storage
                | common::types::DeviceType::Keyboard
                | common::types::DeviceType::Mouse
        );
        if should_audit {
            if device_record.info.device_type == common::types::DeviceType::Storage {
                if let Err(e) = self
                    .storage_session
                    .stop_by_parent(parent_path.to_string(), reason.to_string())
                    .await
                {
                    warn!(
                        parent = %parent_path,
                        error = %e,
                        reason = %reason,
                        "Storage session 清理失败"
                    );
                }
            }
            let mut log = build_audit_log(&device_record.info, event_type::DEVICE_REMOVE);
            log.detail = Some("设备拔出".into());
            if let Err(e) = self.audit.log_usb_audit(&mut log) {
                error!(error = %e, reason = %reason, "审计日志写入失败");
            }
        }
    }

    /// 处理设备移除。
    async fn handle_removed(&mut self, sys_path: String) {
        let parent_path = crate::monitor::parent_device_path(&sys_path);

        let removed = if let Ok(mut dm) = self.device_manager.write() {
            dm.remove_interface(&sys_path)
        } else {
            None
        };

        let Some(device_record) = removed else {
            return;
        };

        info!(dev = %device_record.info.device_name, type = ?device_record.info.device_type, "设备完全拔出");

        self.cleanup_session_by_parent(&parent_path, &device_record, "usb_remove")
            .await;
    }
}

/// 设备类型字符串（审计日志用）。
fn device_type_str(device_type: common::types::DeviceType) -> &'static str {
    match device_type {
        common::types::DeviceType::Storage => "storage",
        common::types::DeviceType::Keyboard => "keyboard",
        common::types::DeviceType::Mouse => "mouse",
        common::types::DeviceType::Unsupported => "unsupported",
        common::types::DeviceType::Unknown => "unknown",
    }
}

/// 构建 USB 审计日志记录。
///
/// 从 UsbDeviceInfo 提取设备属性，填充 UsbAuditLogInsert 的公共字段。
/// 调用方通过修改返回值的可选字段（permission、detail）补充业务信息。
fn build_audit_log(info: &UsbDeviceInfo, event_type: &str) -> UsbAuditLogInsert {
    UsbAuditLogInsert {
        event_time: 0,
        device_type: Some(device_type_str(info.device_type).into()),
        interface_type: Some(crate::descriptor::interface_type_str(info).into()),
        interface_class: Some(info.interface_class as i32),
        interface_subclass: Some(info.interface_subclass as i32),
        interface_protocol: Some(info.interface_protocol as i32),
        device_name: Some(info.device_name.clone()),
        device_sn: Some(info.serial_number.clone()),
        vid: Some(info.vid.clone()),
        pid: Some(info.pid.clone()),
        event_type: event_type.into(),
        permission: None,
        capacity_bytes: info.capacity_bytes,
        detail: None,
    }
}
