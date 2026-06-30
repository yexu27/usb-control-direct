//! DeviceManager 单元测试。
//!
//! 验证父设备路径提取、多接口归并、逐接口移除。

use common::types::DeviceType;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::monitor::{parent_device_path, DeviceManager};

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
fn parent_path_strips_real_sysfs_interface_leaf() {
    assert_eq!(
        parent_device_path("/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1/2-1.1:1.0"),
        "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.1"
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
