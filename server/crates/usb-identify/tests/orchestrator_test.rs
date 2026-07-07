//! S01 编排器单元测试。
//!
//! 使用临时 SQLite 数据库验证事件路由和处理链行为。
//! 不依赖真实 USB 设备。

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex, RwLock};

use tempfile::tempdir;
use tokio::sync::mpsc;

use common::types::DeviceType;
use device_runtime::DeviceRuntimeRegistry;
use hid_access::hid_gadget::HidgNodes;
use log_audit::AuditService;
use storage::Storage;
use storage_test_support::initialize_database;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::monitor::DeviceManager;
use usb_identify::orchestrator::{DeviceEvent, DeviceOrchestrator};
use usb_identify::traits::{
    AuthorizedStorageDevice, StorageSessionController, StorageSessionError, StorageSessionHandle,
};
use whitelist::service::AddWhitelistRequest;
use whitelist::WhitelistManager;

#[derive(Default)]
struct MockStorageSessionController {
    started: AtomicUsize,
    stopped: AtomicUsize,
    stop_all_count: AtomicUsize,
    active: AtomicBool,
    last_started: StdMutex<Option<AuthorizedStorageDevice>>,
}

impl StorageSessionController for MockStorageSessionController {
    fn start_authorized_storage(
        &self,
        device: AuthorizedStorageDevice,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<StorageSessionHandle, StorageSessionError>>
                + Send
                + '_,
        >,
    > {
        self.started.fetch_add(1, Ordering::SeqCst);
        *self.last_started.lock().unwrap() = Some(device.clone());
        Box::pin(async move {
            Ok(StorageSessionHandle {
                session_id: format!("storage-{}", device.serial_number),
                parent_path: device.parent_path,
            })
        })
    }

    fn stop_by_parent(
        &self,
        _parent_path: String,
        _reason: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), StorageSessionError>> + Send + '_>,
    > {
        self.stopped.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }

    fn stop_all(
        &self,
        _reason: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), StorageSessionError>> + Send + '_>,
    > {
        self.stop_all_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }

    fn has_active_storage(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        let active = self.active.load(Ordering::SeqCst);
        Box::pin(async move { active })
    }
}

/// 测试用 HidgNodes（空路径——编排器路由测试不使用）。
fn test_hidg_nodes() -> HidgNodes {
    HidgNodes {
        keyboard: "/dev/null".into(),
        mouse: "/dev/null".into(),
    }
}

fn test_storage_info(serial: &str) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: format!("/sys/devices/test_{}", serial),
        dev_path: Some("/dev/sda1".into()),
        serial_number: serial.into(),
        vid: "0930".into(),
        pid: "6545".into(),
        device_name: format!("Test U盘 {}", serial),
        device_type: DeviceType::Storage,
        interface_class: 0x08,
        interface_subclass: 0x06,
        interface_protocol: 0x50,
        capacity_bytes: Some(16 * 1024 * 1024 * 1024),
    }
}

fn composite_storage_info(serial: &str, interface: &str) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: format!(
            "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.9/2-1.9:{}",
            interface
        ),
        dev_path: Some("/dev/sda1".into()),
        serial_number: serial.into(),
        vid: "0930".into(),
        pid: "6545".into(),
        device_name: format!("Test U盘 {}", serial),
        device_type: DeviceType::Storage,
        interface_class: 0x08,
        interface_subclass: 0x06,
        interface_protocol: 0x50,
        capacity_bytes: Some(16 * 1024 * 1024 * 1024),
    }
}

fn composite_unsupported_info(serial: &str, interface: &str) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: format!(
            "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.9/2-1.9:{}",
            interface
        ),
        dev_path: None,
        serial_number: serial.into(),
        vid: "0930".into(),
        pid: "6545".into(),
        device_name: format!("Composite Vendor {}", serial),
        device_type: DeviceType::Unsupported,
        interface_class: 0xff,
        interface_subclass: 0x42,
        interface_protocol: 0x01,
        capacity_bytes: None,
    }
}

fn setup_services(db_path: &std::path::Path) -> (Arc<AuditService>, Arc<WhitelistManager>) {
    initialize_database(db_path);
    let storage = Arc::new(Storage::open(db_path).unwrap());
    let audit = Arc::new(AuditService::new(Arc::clone(&storage), db_path));
    let whitelist = Arc::new(WhitelistManager::new(storage).unwrap());
    (audit, whitelist)
}

fn build_orchestrator(
    rx: mpsc::UnboundedReceiver<DeviceEvent>,
    whitelist: Arc<WhitelistManager>,
    audit: Arc<AuditService>,
    device_manager: Arc<RwLock<DeviceManager>>,
    storage_session: Arc<MockStorageSessionController>,
) -> DeviceOrchestrator {
    build_orchestrator_with_runtime(
        rx,
        whitelist,
        audit,
        device_manager,
        Arc::new(DeviceRuntimeRegistry::new()),
        storage_session,
    )
}

fn build_orchestrator_with_runtime(
    rx: mpsc::UnboundedReceiver<DeviceEvent>,
    whitelist: Arc<WhitelistManager>,
    audit: Arc<AuditService>,
    device_manager: Arc<RwLock<DeviceManager>>,
    runtime_registry: Arc<DeviceRuntimeRegistry>,
    storage_session: Arc<MockStorageSessionController>,
) -> DeviceOrchestrator {
    DeviceOrchestrator::new(
        rx,
        whitelist,
        audit,
        device_manager,
        runtime_registry,
        storage_session,
        test_hidg_nodes(),
    )
}

fn add_storage_whitelist(whitelist: &WhitelistManager, serial: &str, permission: i32) {
    whitelist
        .add(AddWhitelistRequest {
            serial_number: serial.into(),
            vid: Some("0930".into()),
            pid: Some("6545".into()),
            device_name: Some(format!("Test U盘 {}", serial)),
            capacity_bytes: Some(16 * 1024 * 1024 * 1024),
            device_type: "storage".into(),
            description: None,
            permission,
            add_method: 0,
        })
        .unwrap();
}

fn test_keyboard_info() -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: "/sys/devices/test_kb".into(),
        dev_path: Some("/dev/input/event3".into()),
        serial_number: "".into(),
        vid: "046D".into(),
        pid: "C31C".into(),
        device_name: "Test Keyboard".into(),
        device_type: DeviceType::Keyboard,
        interface_class: 0x03,
        interface_subclass: 0x01,
        interface_protocol: 0x01,
        capacity_bytes: None,
    }
}

fn test_mouse_info() -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: "/sys/devices/test_mouse".into(),
        dev_path: Some("/dev/input/event4".into()),
        serial_number: "".into(),
        vid: "046D".into(),
        pid: "C077".into(),
        device_name: "Test Mouse".into(),
        device_type: DeviceType::Mouse,
        interface_class: 0x03,
        interface_subclass: 0x01,
        interface_protocol: 0x02,
        capacity_bytes: None,
    }
}

#[tokio::test]
async fn test_storage_whitelist_denied() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let runtime_registry = Arc::new(DeviceRuntimeRegistry::new());
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator_with_runtime(
        rx,
        whitelist,
        audit,
        device_manager,
        Arc::clone(&runtime_registry),
        storage_session,
    );

    tx.send(DeviceEvent::StorageAdded(test_storage_info(
        "SN-NOT-IN-WHITELIST",
    )))
    .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(storage_session_assert.started.load(Ordering::SeqCst), 0);
    assert!(runtime_registry.list().is_empty());
}

#[tokio::test]
async fn whitelisted_storage_is_routed_to_storage_session_manager() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);
    add_storage_whitelist(&whitelist, "SN-ALLOW", 1);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let runtime_registry = Arc::new(DeviceRuntimeRegistry::new());
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator_with_runtime(
        rx,
        whitelist,
        audit,
        device_manager,
        Arc::clone(&runtime_registry),
        storage_session,
    );

    tx.send(DeviceEvent::StorageAdded(test_storage_info("SN-ALLOW")))
        .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(storage_session_assert.started.load(Ordering::SeqCst), 1);
    let started = storage_session_assert.last_started.lock().unwrap();
    let device = started.as_ref().unwrap();
    assert_eq!(device.serial_number, "SN-ALLOW");
    assert_eq!(device.dev_path, "/dev/sda1");
    assert_eq!(device.permission, 1);
    assert_eq!(device.runtime_id, "runtime__/sys/devices/test_SN-ALLOW");

    let snapshots = runtime_registry.list();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].serial_number, "SN-ALLOW");
    assert_eq!(snapshots[0].device_type, "storage");
    assert_eq!(snapshots[0].interface_type, "mass_storage");
    assert_eq!(snapshots[0].status, "accepted");
    assert_eq!(snapshots[0].stage, "admission");
}

#[tokio::test]
async fn test_keyboard_added() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let runtime_registry = Arc::new(DeviceRuntimeRegistry::new());
    let orchestrator = build_orchestrator_with_runtime(
        rx,
        whitelist,
        audit,
        device_manager,
        Arc::clone(&runtime_registry),
        Arc::new(MockStorageSessionController::default()),
    );

    tx.send(DeviceEvent::KeyboardAdded(test_keyboard_info()))
        .unwrap();
    drop(tx);

    orchestrator.run().await;

    let snapshots = runtime_registry.list();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].device_type, "keyboard");
    assert_eq!(snapshots[0].status, "failed");
    assert_eq!(snapshots[0].stage, "keyboard_evdev_bind");
    assert_eq!(snapshots[0].fail_code, "evdev_not_found");
}

#[tokio::test]
async fn test_mouse_added() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let runtime_registry = Arc::new(DeviceRuntimeRegistry::new());
    let orchestrator = build_orchestrator_with_runtime(
        rx,
        whitelist,
        audit,
        device_manager,
        Arc::clone(&runtime_registry),
        Arc::new(MockStorageSessionController::default()),
    );

    tx.send(DeviceEvent::MouseAdded(test_mouse_info())).unwrap();
    drop(tx);

    orchestrator.run().await;

    let snapshots = runtime_registry.list();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].device_type, "mouse");
    assert_eq!(snapshots[0].status, "failed");
    assert_eq!(snapshots[0].stage, "mouse_evdev_bind");
    assert_eq!(snapshots[0].fail_code, "evdev_not_found");
}

#[tokio::test]
async fn test_unsupported_device_blocked() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        device_manager,
        Arc::new(MockStorageSessionController::default()),
    );

    let info = UsbDeviceInfo {
        sys_path: "/sys/devices/test_unknown".into(),
        dev_path: None,
        serial_number: "".into(),
        vid: "0000".into(),
        pid: "0000".into(),
        device_name: "Unknown Device".into(),
        device_type: DeviceType::Unknown,
        interface_class: 0xFF,
        interface_subclass: 0xFF,
        interface_protocol: 0xFF,
        capacity_bytes: None,
    };
    tx.send(DeviceEvent::UnsupportedAdded(info, "未知设备类型".into()))
        .unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn test_device_removed() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        device_manager,
        Arc::new(MockStorageSessionController::default()),
    );

    tx.send(DeviceEvent::DeviceRemoved(
        "/sys/devices/test_remove".into(),
    ))
    .unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn shutdown_cleanup_delegates_to_storage_session_manager() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);
    let (_tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator(rx, whitelist, audit, device_manager, storage_session);

    orchestrator.shutdown_cleanup("service_shutdown").await;

    assert_eq!(
        storage_session_assert.stop_all_count.load(Ordering::SeqCst),
        1
    );
}

#[tokio::test]
async fn test_parent_device_removed_clears_registered_device() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    {
        let mut dm = device_manager.write().unwrap();
        let mut info = test_keyboard_info();
        info.sys_path = "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.0".into();
        dm.add_interface(info);
    }

    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        Arc::new(MockStorageSessionController::default()),
    );

    tx.send(DeviceEvent::DeviceRemoved(
        "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.0".into(),
    ))
    .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(device_manager.read().unwrap().count(), 0);
}

#[tokio::test]
async fn test_multi_interface_keyboard_registers_one_device() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        Arc::new(MockStorageSessionController::default()),
    );

    let mut kb0 = test_keyboard_info();
    kb0.sys_path = "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.0".into();

    let mut kb1 = test_keyboard_info();
    kb1.sys_path = "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.1".into();
    kb1.interface_protocol = 0x00;
    kb1.device_type = DeviceType::Unsupported;

    tx.send(DeviceEvent::KeyboardAdded(kb0)).unwrap();
    tx.send(DeviceEvent::UnsupportedAdded(
        kb1,
        "不支持的设备类型".into(),
    ))
    .unwrap();
    drop(tx);

    orchestrator.run().await;

    let dm = device_manager.read().unwrap();
    assert_eq!(dm.count(), 1);
    let record = dm
        .get_by_parent("/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1")
        .unwrap();
    assert_eq!(record.interface_count(), 2);
}

#[tokio::test]
async fn test_multi_interface_remove_waits_until_last_interface() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    {
        let mut dm = device_manager.write().unwrap();
        let mut kb0 = test_keyboard_info();
        kb0.sys_path = "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.0".into();
        dm.add_interface(kb0);

        let mut kb1 = test_keyboard_info();
        kb1.sys_path = "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.1".into();
        kb1.device_type = DeviceType::Unsupported;
        dm.add_interface(kb1);
    }

    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        Arc::new(MockStorageSessionController::default()),
    );

    tx.send(DeviceEvent::DeviceRemoved(
        "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.0".into(),
    ))
    .unwrap();
    tx.send(DeviceEvent::DeviceRemoved(
        "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.1".into(),
    ))
    .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(device_manager.read().unwrap().count(), 0);
}

#[tokio::test]
async fn unsupported_interface_first_does_not_block_later_whitelisted_storage() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);
    add_storage_whitelist(&whitelist, "SN-PHONE", 1);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        storage_session,
    );

    tx.send(DeviceEvent::UnsupportedAdded(
        composite_unsupported_info("SN-PHONE", "1.1"),
        "不支持的设备类型".into(),
    ))
    .unwrap();
    tx.send(DeviceEvent::StorageAdded(composite_storage_info(
        "SN-PHONE", "1.0",
    )))
    .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(storage_session_assert.started.load(Ordering::SeqCst), 1);
    let dm = device_manager.read().unwrap();
    let record = dm
        .get_by_parent("/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.9")
        .unwrap();
    assert_eq!(record.interface_count(), 2);
}

#[tokio::test]
async fn unsupported_interface_after_storage_does_not_stop_or_duplicate_storage() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);
    add_storage_whitelist(&whitelist, "SN-PHONE", 1);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        storage_session,
    );

    tx.send(DeviceEvent::StorageAdded(composite_storage_info(
        "SN-PHONE", "1.0",
    )))
    .unwrap();
    tx.send(DeviceEvent::UnsupportedAdded(
        composite_unsupported_info("SN-PHONE", "1.1"),
        "不支持的设备类型".into(),
    ))
    .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(storage_session_assert.started.load(Ordering::SeqCst), 1);
    assert_eq!(storage_session_assert.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(device_manager.read().unwrap().count(), 1);
}

#[tokio::test]
async fn removing_storage_interface_stops_storage_even_when_unsupported_interface_remains() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);
    add_storage_whitelist(&whitelist, "SN-PHONE", 1);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        storage_session,
    );

    let storage = composite_storage_info("SN-PHONE", "1.0");
    let vendor = composite_unsupported_info("SN-PHONE", "1.1");

    tx.send(DeviceEvent::StorageAdded(storage.clone())).unwrap();
    tx.send(DeviceEvent::UnsupportedAdded(
        vendor,
        "不支持的设备类型".into(),
    ))
    .unwrap();
    tx.send(DeviceEvent::DeviceRemoved(storage.sys_path))
        .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(storage_session_assert.started.load(Ordering::SeqCst), 1);
    assert_eq!(storage_session_assert.stopped.load(Ordering::SeqCst), 1);
    assert_eq!(device_manager.read().unwrap().count(), 1);
}

#[tokio::test]
async fn removing_unsupported_interface_does_not_stop_storage() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);
    add_storage_whitelist(&whitelist, "SN-PHONE", 1);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        storage_session,
    );

    let storage = composite_storage_info("SN-PHONE", "1.0");
    let vendor = composite_unsupported_info("SN-PHONE", "1.1");

    tx.send(DeviceEvent::StorageAdded(storage)).unwrap();
    tx.send(DeviceEvent::UnsupportedAdded(
        vendor.clone(),
        "不支持的设备类型".into(),
    ))
    .unwrap();
    tx.send(DeviceEvent::DeviceRemoved(vendor.sys_path))
        .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(storage_session_assert.started.load(Ordering::SeqCst), 1);
    assert_eq!(storage_session_assert.stopped.load(Ordering::SeqCst), 0);
    assert_eq!(device_manager.read().unwrap().count(), 1);
}

#[tokio::test]
async fn removing_all_composite_interfaces_clears_parent_and_stops_storage_once() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);
    add_storage_whitelist(&whitelist, "SN-PHONE", 1);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let storage_session = Arc::new(MockStorageSessionController::default());
    let storage_session_assert = Arc::clone(&storage_session);
    let orchestrator = build_orchestrator(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        storage_session,
    );

    let storage = composite_storage_info("SN-PHONE", "1.0");
    let vendor = composite_unsupported_info("SN-PHONE", "1.1");

    tx.send(DeviceEvent::StorageAdded(storage.clone())).unwrap();
    tx.send(DeviceEvent::UnsupportedAdded(
        vendor.clone(),
        "不支持的设备类型".into(),
    ))
    .unwrap();
    tx.send(DeviceEvent::DeviceRemoved(vendor.sys_path))
        .unwrap();
    tx.send(DeviceEvent::DeviceRemoved(storage.sys_path))
        .unwrap();
    drop(tx);

    orchestrator.run().await;

    assert_eq!(storage_session_assert.started.load(Ordering::SeqCst), 1);
    assert_eq!(storage_session_assert.stopped.load(Ordering::SeqCst), 1);
    assert_eq!(device_manager.read().unwrap().count(), 0);
}
