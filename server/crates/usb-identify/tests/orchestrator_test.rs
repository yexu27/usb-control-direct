//! S01 编排器单元测试。
//!
//! 使用临时 SQLite 数据库验证事件路由和处理链行为。
//! 不依赖真实 USB 设备。

use std::path::Path;
use std::sync::{Arc, RwLock};

use tokio::sync::mpsc;
use tempfile::tempdir;

use common::types::DeviceType;
use hid_access::hid_gadget::HidgNodes;
use log_audit::AuditService;
use storage::Storage;
use storage_test_support::initialize_database;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::monitor::DeviceManager;
use usb_identify::orchestrator::{DeviceEvent, DeviceOrchestrator, NbdPool};
use usb_identify::traits::{
    DeviceMapper, MapContext, MapError, MappedSession, ScanError, ScanResult, Scanner, UnmapError,
};
use whitelist::WhitelistManager;

/// 测试用空 Scanner——编排器路由测试不会触发实际扫描。
struct MockScanner;
impl Scanner for MockScanner {
    fn scan(
        &self,
        _mount_path: &Path,
        _device_sn: &str,
        _device_name: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ScanResult, ScanError>> + Send + '_>,
    > {
        Box::pin(async { Err(ScanError::Failed("mock: 未实现".into())) })
    }
    fn cancel(&self, _mount_path: &Path) {}
}

/// 测试用空 DeviceMapper——编排器路由测试不会触发实际映射。
struct MockDeviceMapper;
impl DeviceMapper for MockDeviceMapper {
    fn map_device(
        &self,
        _ctx: MapContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<MappedSession, MapError>> + Send + '_>,
    > {
        Box::pin(async { Err(MapError::BuildFailed("mock: 未实现".into())) })
    }
    fn unmap_device(
        &self,
        _session: MappedSession,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), UnmapError>> + Send + '_>,
    > {
        Box::pin(async { Err(UnmapError::Failed("mock: 未实现".into())) })
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

fn setup_services(
    db_path: &std::path::Path,
) -> (Arc<AuditService>, Arc<WhitelistManager>) {
    initialize_database(db_path);
    let storage = Arc::new(Storage::open(db_path).unwrap());
    let audit = Arc::new(AuditService::new(Arc::clone(&storage), db_path));
    let whitelist = Arc::new(WhitelistManager::new(storage).unwrap());
    (audit, whitelist)
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
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit, device_manager, Arc::new(MockScanner), Arc::new(MockDeviceMapper), test_hidg_nodes());

    tx.send(DeviceEvent::StorageAdded(test_storage_info("SN-NOT-IN-WHITELIST"))).unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn test_keyboard_added() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit, device_manager, Arc::new(MockScanner), Arc::new(MockDeviceMapper), test_hidg_nodes());

    tx.send(DeviceEvent::KeyboardAdded(test_keyboard_info())).unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn test_mouse_added() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit, device_manager, Arc::new(MockScanner), Arc::new(MockDeviceMapper), test_hidg_nodes());

    tx.send(DeviceEvent::MouseAdded(test_mouse_info())).unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn test_unsupported_device_blocked() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let (audit, whitelist) = setup_services(&db_path);

    let (tx, rx) = mpsc::unbounded_channel();
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit, device_manager, Arc::new(MockScanner), Arc::new(MockDeviceMapper), test_hidg_nodes());

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
    tx.send(DeviceEvent::UnsupportedAdded(info, "未知设备类型".into())).unwrap();
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
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit, device_manager, Arc::new(MockScanner), Arc::new(MockDeviceMapper), test_hidg_nodes());

    tx.send(DeviceEvent::DeviceRemoved("/sys/devices/test_remove".into())).unwrap();
    drop(tx);

    orchestrator.run().await;
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
        info.sys_path =
            "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.0".into();
        dm.add(info);
    }

    let orchestrator = DeviceOrchestrator::new(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        Arc::new(MockScanner),
        Arc::new(MockDeviceMapper),
        test_hidg_nodes(),
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
    let orchestrator = DeviceOrchestrator::new(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        Arc::new(MockScanner),
        Arc::new(MockDeviceMapper),
        test_hidg_nodes(),
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
    assert_eq!(record.interfaces.len(), 2);
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
        dm.add(kb0);

        let mut kb1 = test_keyboard_info();
        kb1.sys_path = "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.1".into();
        kb1.device_type = DeviceType::Unsupported;
        dm.add(kb1);
    }

    let orchestrator = DeviceOrchestrator::new(
        rx,
        whitelist,
        audit,
        Arc::clone(&device_manager),
        Arc::new(MockScanner),
        Arc::new(MockDeviceMapper),
        test_hidg_nodes(),
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

#[test]
fn test_nbd_pool_acquire_release() {
    let mut pool = NbdPool::new();

    let idx0 = pool.acquire().unwrap();
    let idx1 = pool.acquire().unwrap();
    assert_ne!(idx0, idx1);

    pool.release(idx0);
    let idx2 = pool.acquire().unwrap();
    assert_eq!(idx2, idx0);

    pool.acquire().unwrap(); // idx3
    pool.acquire().unwrap(); // last one
    assert!(pool.acquire().is_none());
}
