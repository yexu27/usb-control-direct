//! S01 编排器单元测试。
//!
//! 使用临时 SQLite 数据库验证事件路由和处理链行为。
//! 不依赖真实 USB 设备。

use std::sync::Arc;

use tokio::sync::mpsc;
use tempfile::tempdir;

use common::types::DeviceType;
use log_audit::AuditService;
use storage::Storage;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::orchestrator::{DeviceEvent, DeviceOrchestrator, NbdPool};
use whitelist::WhitelistManager;

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
    let audit = Arc::new(AuditService::new(
        Storage::open(&db_path).unwrap(),
        &db_path,
    ));
    let whitelist = Arc::new(
        WhitelistManager::new(Storage::open(&db_path).unwrap()).unwrap(),
    );

    let (tx, rx) = mpsc::unbounded_channel();
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit);

    tx.send(DeviceEvent::StorageAdded(test_storage_info("SN-NOT-IN-WHITELIST"))).unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn test_keyboard_added() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let audit = Arc::new(AuditService::new(
        Storage::open(&db_path).unwrap(),
        &db_path,
    ));
    let whitelist = Arc::new(
        WhitelistManager::new(Storage::open(&db_path).unwrap()).unwrap(),
    );

    let (tx, rx) = mpsc::unbounded_channel();
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit);

    tx.send(DeviceEvent::KeyboardAdded(test_keyboard_info())).unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn test_mouse_added() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let audit = Arc::new(AuditService::new(
        Storage::open(&db_path).unwrap(),
        &db_path,
    ));
    let whitelist = Arc::new(
        WhitelistManager::new(Storage::open(&db_path).unwrap()).unwrap(),
    );

    let (tx, rx) = mpsc::unbounded_channel();
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit);

    tx.send(DeviceEvent::MouseAdded(test_mouse_info())).unwrap();
    drop(tx);

    orchestrator.run().await;
}

#[tokio::test]
async fn test_unsupported_device_blocked() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let audit = Arc::new(AuditService::new(
        Storage::open(&db_path).unwrap(),
        &db_path,
    ));
    let whitelist = Arc::new(
        WhitelistManager::new(Storage::open(&db_path).unwrap()).unwrap(),
    );

    let (tx, rx) = mpsc::unbounded_channel();
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit);

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
    let audit = Arc::new(AuditService::new(
        Storage::open(&db_path).unwrap(),
        &db_path,
    ));
    let whitelist = Arc::new(
        WhitelistManager::new(Storage::open(&db_path).unwrap()).unwrap(),
    );

    let (tx, rx) = mpsc::unbounded_channel();
    let orchestrator = DeviceOrchestrator::new(rx, whitelist, audit);

    tx.send(DeviceEvent::DeviceRemoved("/sys/devices/test_remove".into())).unwrap();
    drop(tx);

    orchestrator.run().await;
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
