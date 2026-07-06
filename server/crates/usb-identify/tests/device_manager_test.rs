//! DeviceManager 单元测试。
//!
//! 验证父设备路径提取、接口级注册、接口级移除和父设备最终清理。

use common::types::DeviceType;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::monitor::{parent_device_path, DeviceManager, InterfaceRemoveResult};

fn make_info(
    sys_path: &str,
    device_type: DeviceType,
    class: u8,
    subclass: u8,
    protocol: u8,
    name: &str,
) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: sys_path.into(),
        dev_path: None,
        serial_number: name.into(),
        vid: "0000".into(),
        pid: "0000".into(),
        device_name: name.into(),
        device_type,
        interface_class: class,
        interface_subclass: subclass,
        interface_protocol: protocol,
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
fn composite_device_keeps_each_interface_info() {
    let mut dm = DeviceManager::new();
    let storage = make_info(
        "/sys/.../2-1.1:1.0",
        DeviceType::Storage,
        0x08,
        0x06,
        0x50,
        "PhoneStorage",
    );
    let vendor = make_info(
        "/sys/.../2-1.1:1.1",
        DeviceType::Unsupported,
        0xff,
        0x42,
        0x01,
        "PhoneVendor",
    );

    let storage_result = dm.add_interface(storage.clone());
    let vendor_result = dm.add_interface(vendor.clone());

    assert!(storage_result.is_new_interface);
    assert!(vendor_result.is_new_interface);
    assert_eq!(storage_result.parent_path, "/sys/.../2-1.1");
    assert_eq!(vendor_result.parent_path, "/sys/.../2-1.1");
    assert_eq!(dm.count(), 1);

    let record = dm.get_by_parent("/sys/.../2-1.1").unwrap();
    assert_eq!(record.interface_count(), 2);
    assert_eq!(
        record
            .interface("/sys/.../2-1.1:1.0")
            .unwrap()
            .info
            .device_type,
        DeviceType::Storage
    );
    assert_eq!(
        record
            .interface("/sys/.../2-1.1:1.1")
            .unwrap()
            .info
            .device_type,
        DeviceType::Unsupported
    );
}

#[test]
fn duplicate_interface_is_not_added_twice() {
    let mut dm = DeviceManager::new();
    let storage = make_info(
        "/sys/.../2-1.2:1.0",
        DeviceType::Storage,
        0x08,
        0x06,
        0x50,
        "USB",
    );

    let first = dm.add_interface(storage.clone());
    let second = dm.add_interface(storage);

    assert!(first.is_new_interface);
    assert!(!second.is_new_interface);
    assert_eq!(
        dm.get_by_parent("/sys/.../2-1.2")
            .unwrap()
            .interface_count(),
        1
    );
}

#[test]
fn remove_supported_interface_returns_that_interface_even_when_parent_remains() {
    let mut dm = DeviceManager::new();
    dm.add_interface(make_info(
        "/sys/.../2-1.3:1.0",
        DeviceType::Storage,
        0x08,
        0x06,
        0x50,
        "PhoneStorage",
    ));
    dm.add_interface(make_info(
        "/sys/.../2-1.3:1.1",
        DeviceType::Unsupported,
        0xff,
        0x42,
        0x01,
        "PhoneVendor",
    ));

    let removed = dm.remove_interface("/sys/.../2-1.3:1.0").unwrap();

    assert_eq!(removed.parent_path, "/sys/.../2-1.3");
    assert_eq!(removed.interface.info.device_type, DeviceType::Storage);
    assert!(!removed.parent_removed);
    assert_eq!(dm.count(), 1);
    assert_eq!(
        dm.get_by_parent("/sys/.../2-1.3")
            .unwrap()
            .interface_count(),
        1
    );
}

#[test]
fn remove_last_interface_removes_parent_record() {
    let mut dm = DeviceManager::new();
    dm.add_interface(make_info(
        "/sys/.../2-1.4:1.0",
        DeviceType::Unsupported,
        0xff,
        0x00,
        0x00,
        "OnlyVendor",
    ));

    let removed = dm.remove_interface("/sys/.../2-1.4:1.0").unwrap();

    assert!(matches!(
        removed,
        InterfaceRemoveResult {
            parent_removed: true,
            ..
        }
    ));
    assert_eq!(dm.count(), 0);
}

#[test]
fn remove_unknown_path_returns_none() {
    let mut dm = DeviceManager::new();
    assert!(dm.remove_interface("/sys/.../nonexistent:1.0").is_none());
}
