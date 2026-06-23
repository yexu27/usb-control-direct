//! S01 设备编排器。
//!
//! 通过 tokio mpsc channel 接收 udev 事件，按设备类型路由到对应处理链：
//! - Storage → 白名单查询 → mount → 扫描 → NBD 映射
//! - Keyboard → evdev 拦截（S02）
//! - Mouse → evdev 转发（S02）
//! - Unsupported → 记录 BLOCKED 日志
//!
//! 使用 NBD 设备号池（/dev/nbd0—/dev/nbd3）支持多 U 盘同时映射。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tokio::sync::{mpsc, watch};
use tracing::{info, warn};

use crate::mount::{dev_name_from_path, MountOperations, RealMountOps};

use common::time::now_unix;
use log_audit::AuditService;
use storage::model::UsbAuditLogInsert;
use whitelist::WhitelistManager;

use crate::descriptor::UsbDeviceInfo;
use crate::monitor::DeviceManager;

/// 活动设备会话——追踪后台 task 和运行时资源。
/// 与 DeviceManager 同键（parent device path），不同职责：
///   DeviceManager: 设备属性，协议层可查询
///   ActiveSession: 运行时资源，仅 Orchestrator 内部
struct ActiveSession {
    device_type: common::types::DeviceType,
    nbd_index: Option<u32>,
    mount_path: Option<PathBuf>,
    cancel_tx: watch::Sender<bool>,
}

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

/// 从 USB 接口 sysfs 路径查找对应的 evdev 设备节点。
///
/// 内核为 USB HID 设备创建 input 子设备：
///   /sys/devices/.../2-1.1:1.0/0003:.../input/input3/event3
fn find_evdev_path(usb_iface_syspath: &str) -> Option<std::path::PathBuf> {
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
                        let dev_path = std::path::PathBuf::from("/dev/input").join(&event_name);
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

/// 主编排器。
///
/// 持有所有服务引用，接收 udev 事件并按类型路由。
/// 设备状态管理委托给 `DeviceManager`。
pub struct DeviceOrchestrator {
    rx: mpsc::UnboundedReceiver<DeviceEvent>,
    whitelist: Arc<WhitelistManager>,
    audit: Arc<AuditService>,
    device_manager: Arc<RwLock<DeviceManager>>,

    // 新增: 下游服务
    scan_service: Arc<dyn crate::traits::Scanner>,
    file_access_engine: Arc<dyn crate::traits::DeviceMapper>,

    // 新增: I/O 资源
    mount_ops: RealMountOps,
    nbd_pool: NbdPool,
    hidg_nodes: hid_access::hid_gadget::HidgNodes,

    // 新增: 运行时追踪
    active_sessions: HashMap<String, ActiveSession>,
}

impl DeviceOrchestrator {
    /// 创建编排器。
    pub fn new(
        rx: mpsc::UnboundedReceiver<DeviceEvent>,
        whitelist: Arc<WhitelistManager>,
        audit: Arc<AuditService>,
        device_manager: Arc<RwLock<DeviceManager>>,
        scan_service: Arc<dyn crate::traits::Scanner>,
        file_access_engine: Arc<dyn crate::traits::DeviceMapper>,
        hidg_nodes: hid_access::hid_gadget::HidgNodes,
    ) -> Self {
        Self {
            rx,
            whitelist,
            audit,
            device_manager,
            scan_service,
            file_access_engine,
            mount_ops: RealMountOps,
            nbd_pool: NbdPool::new(),
            hidg_nodes,
            active_sessions: HashMap::new(),
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
        // ===== Phase 1: Event Loop 同步判断 =====
        let parent_path = crate::monitor::parent_device_path(&info.sys_path);
        if let Ok(mut dm) = self.device_manager.write() {
            dm.add(info.clone());
        }

        let serial = info.serial_number.clone();
        if serial.is_empty() {
            self.write_audit_storage(&info, "whitelist_denied", "blocked",
                Some("序列号为空"));
            return;
        }
        let whitelist_entry = match self.whitelist.is_whitelisted(&serial) {
            Some(e) => e,
            None => {
                self.write_audit_storage(&info, "whitelist_denied", "blocked",
                    Some("不在白名单"));
                return;
            }
        };

        let nbd_index = match self.nbd_pool.acquire() {
            Some(idx) => idx,
            None => {
                warn!("NBD 设备号池耗尽，拒绝映射");
                self.write_audit_storage(&info, "map_failed", "failed",
                    Some("NBD 设备号池耗尽"));
                return;
            }
        };

        // 注册 ActiveSession
        let (cancel_tx, mut cancel_rx) = watch::channel(false);
        self.active_sessions.insert(parent_path.clone(), ActiveSession {
            device_type: common::types::DeviceType::Storage,
            nbd_index: Some(nbd_index),
            mount_path: None,
            cancel_tx,
        });

        info!(serial = %serial, nbd = nbd_index, "Storage 设备开始编排");

        // 克隆 Arc 供 spawned task
        let scan_service = Arc::clone(&self.scan_service);
        let file_access_engine = Arc::clone(&self.file_access_engine);
        let audit = Arc::clone(&self.audit);
        let permission = whitelist_entry.permission;

        // ===== Phase 2: spawn 独立 task =====
        tokio::spawn(async move {
            let dev_path = match &info.dev_path {
                Some(p) if !p.is_empty() => p.clone(),
                _ => {
                    write_audit_fail(&audit, &info, "map_failed", "dev_path 为空");
                    return;
                }
            };
            let dev_name = dev_name_from_path(&dev_path);
            let mount_point = crate::mount::mount_path_for(dev_name);
            let mount_ops = RealMountOps;

            // 1. Mount
            let read_only = permission == 0;
            if let Err(e) = crate::mount::mount_partition(&dev_path, &mount_point.to_string_lossy(), read_only) {
                write_audit_fail(&audit, &info, "scan_failed", &e.to_string());
                return;
            }
            let mount_path_str = mount_point.to_string_lossy().to_string();

            // 2. Scan（可取消）
            let scan_result = tokio::select! {
                r = scan_service.scan(&mount_point, &serial, &info.device_name) => r,
                _ = cancel_rx.changed() => {
                    info!(serial = %serial, "扫描被取消（设备拔出）");
                    let _ = mount_ops.umount(&mount_path_str);
                    return;
                }
            };
            let scan_result = match scan_result {
                Ok(r) => r,
                Err(e) => {
                    write_audit_fail(&audit, &info, "scan_failed", &e.to_string());
                    let _ = mount_ops.umount(&mount_path_str);
                    return;
                }
            };

            // 3. Map（可取消）
            let map_ctx = crate::traits::MapContext {
                mount_path: mount_path_str.clone(),
                scan_result: scan_result.clone(),
                permission,
            };
            let map_result = tokio::select! {
                r = file_access_engine.map_device(map_ctx) => r,
                _ = cancel_rx.changed() => {
                    info!(serial = %serial, "映射被取消（设备拔出）");
                    let _ = mount_ops.umount(&mount_path_str);
                    return;
                }
            };
            match map_result {
                Ok(_session) => {
                    write_audit_storage_static(&audit, &info, "mapped", "mapped", None);
                    info!(serial = %serial, "U 盘映射成功");
                }
                Err(e) => {
                    write_audit_fail(&audit, &info, "map_failed", &e.to_string());
                    let _ = mount_ops.umount(&mount_path_str);
                }
            }
        });
    }

    /// 处理键盘设备。
    async fn handle_keyboard(&mut self, info: UsbDeviceInfo) {
        let parent_path = crate::monitor::parent_device_path(&info.sys_path);

        // DeviceManager 登记
        if let Ok(mut dm) = self.device_manager.write() {
            dm.add(info.clone());
        }

        // 发现 evdev 设备节点
        let evdev_path = match find_evdev_path(&info.sys_path) {
            Some(p) => p,
            None => {
                warn!(dev = %info.device_name, "键盘: 找不到对应 evdev 设备");
                return;
            }
        };

        // 注册 ActiveSession
        let (cancel_tx, _cancel_rx) = watch::channel(false);
        self.active_sessions.insert(parent_path.clone(), ActiveSession {
            device_type: common::types::DeviceType::Keyboard,
            nbd_index: None,
            mount_path: None,
            cancel_tx,
        });

        let hidg_kb = self.hidg_nodes.keyboard.clone();
        let device_name = info.device_name.clone();
        let info4audit = info.clone();
        let audit = Arc::clone(&self.audit);

        info!(dev = %device_name, evdev = %evdev_path.display(), "键盘: 启动拦截器");

        // spawn_blocking: evdev IO 是阻塞的，拔出时 read 报错自然退出
        tokio::task::spawn_blocking(move || {
            use hid_access::evdev_interceptor::KeyboardInterceptor;
            let mut interceptor = KeyboardInterceptor::new(hidg_kb);
            match interceptor.run(&evdev_path) {
                Ok(()) => info!(dev = %device_name, "键盘拦截器正常退出"),
                Err(e) => warn!(dev = %device_name, error = %e, "键盘拦截器异常退出"),
            }
            write_audit_generic_static(&audit, &info4audit, "device_removed",
                "success", "keyboard", "hid_keyboard", 0x03, 0x01, 0x01);
        });
    }

    /// 处理鼠标设备。
    async fn handle_mouse(&mut self, info: UsbDeviceInfo) {
        let parent_path = crate::monitor::parent_device_path(&info.sys_path);

        if let Ok(mut dm) = self.device_manager.write() {
            dm.add(info.clone());
        }

        let evdev_path = match find_evdev_path(&info.sys_path) {
            Some(p) => p,
            None => {
                warn!(dev = %info.device_name, "鼠标: 找不到对应 evdev 设备");
                return;
            }
        };

        let (cancel_tx, _cancel_rx) = watch::channel(false);
        self.active_sessions.insert(parent_path.clone(), ActiveSession {
            device_type: common::types::DeviceType::Mouse,
            nbd_index: None,
            mount_path: None,
            cancel_tx,
        });

        let hidg_mouse = self.hidg_nodes.mouse.clone();
        let device_name = info.device_name.clone();
        let info4audit = info.clone();
        let audit = Arc::clone(&self.audit);

        info!(dev = %device_name, evdev = %evdev_path.display(), "鼠标: 启动转发器");

        tokio::task::spawn_blocking(move || {
            use hid_access::mouse_forwarder::MouseForwarder;
            let mut forwarder = MouseForwarder::new(hidg_mouse);
            match forwarder.run(&evdev_path) {
                Ok(()) => info!(dev = %device_name, "鼠标转发器正常退出"),
                Err(e) => warn!(dev = %device_name, error = %e, "鼠标转发器异常退出"),
            }
            write_audit_generic_static(&audit, &info4audit, "device_removed",
                "success", "mouse", "hid_mouse", 0x03, 0x01, 0x02);
        });
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
        let parent_path = crate::monitor::parent_device_path(&sys_path);

        // DeviceManager: 移除接口。只有最后一个接口移除才返回 Some
        let removed = if let Ok(mut dm) = self.device_manager.write() {
            dm.remove_interface(&sys_path)
        } else {
            None
        };

        let Some(device_record) = removed else {
            return; // 还有其他接口，物理设备未完全拔出
        };

        info!(dev = %device_record.info.device_name, type = ?device_record.info.device_type, "设备完全拔出");

        // ActiveSession: 发取消信号 + 兜底清理
        if let Some(session) = self.active_sessions.remove(&parent_path) {
            let _ = session.cancel_tx.send(true); // 幂等

            // Storage 兜底清理
            if session.device_type == common::types::DeviceType::Storage {
                if let Some(mount_path) = &session.mount_path {
                    let _ = self.mount_ops.umount(&mount_path.to_string_lossy());
                }
                if let Some(idx) = session.nbd_index {
                    self.nbd_pool.release(idx);
                }
            }
        }

        // 记移除日志
        let (dev_type_str, iface_str) = match device_record.info.device_type {
            common::types::DeviceType::Storage => ("storage", "mass_storage"),
            common::types::DeviceType::Keyboard => ("keyboard", "hid_keyboard"),
            common::types::DeviceType::Mouse => ("mouse", "hid_mouse"),
            _ => ("unsupported", "unsupported"),
        };
        self.write_audit_generic(&device_record.info, "device_removed", "success",
            dev_type_str, iface_str,
            device_record.info.interface_class as i32,
            device_record.info.interface_subclass as i32,
            device_record.info.interface_protocol as i32,
        );
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

}

fn write_audit_storage_static(
    audit: &log_audit::AuditService,
    info: &UsbDeviceInfo,
    event_type: &str,
    result: &str,
    fail_reason: Option<&str>,
) {
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
    let _ = audit.log_usb_audit(&mut log);
}

fn write_audit_fail(
    audit: &log_audit::AuditService,
    info: &UsbDeviceInfo,
    event_type: &str,
    reason: &str,
) {
    write_audit_storage_static(audit, info, event_type, "failed", Some(reason));
}

fn write_audit_generic_static(
    audit: &log_audit::AuditService,
    info: &crate::descriptor::UsbDeviceInfo,
    event_type: &str,
    result: &str,
    device_type: &str,
    interface_type: &str,
    iface_class: i32,
    iface_subclass: i32,
    iface_proto: i32,
) {
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
    let _ = audit.log_usb_audit(&mut log);
}

/// 为设备生成挂载路径。
pub fn mount_path_for(info: &UsbDeviceInfo) -> PathBuf {
    let dev_name = info.dev_path.as_deref()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("unknown");
    PathBuf::from("/mnt/usb_raw").join(dev_name)
}
