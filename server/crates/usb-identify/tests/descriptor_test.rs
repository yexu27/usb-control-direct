use common::types::DeviceType;
use usb_identify::descriptor::{
    admission_status_str, classify_device, detect_spoof, interface_type_str, UsbDeviceInfo,
};

fn make_info(device_type: DeviceType, class: u8, sub: u8, proto: u8) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: "/sys/devices/test".into(),
        dev_path: None,
        serial_number: "TEST_SN".into(),
        vid: "0930".into(),
        pid: "6545".into(),
        device_name: "Test Device".into(),
        device_type,
        interface_class: class,
        interface_subclass: sub,
        interface_protocol: proto,
        capacity_bytes: None,
    }
}

#[test]
fn classify_mass_storage() {
    assert!(matches!(
        classify_device(0x08, 0x06, 0x50),
        DeviceType::Storage
    ));
}

#[test]
fn classify_keyboard() {
    assert!(matches!(
        classify_device(0x03, 0x01, 0x01),
        DeviceType::Keyboard
    ));
}

#[test]
fn classify_mouse() {
    assert!(matches!(
        classify_device(0x03, 0x01, 0x02),
        DeviceType::Mouse
    ));
}

#[test]
fn classify_hid_other_is_unsupported() {
    assert!(matches!(
        classify_device(0x03, 0x00, 0x00),
        DeviceType::Unsupported
    ));
}

#[test]
fn classify_unknown_class() {
    assert!(matches!(
        classify_device(0xFF, 0x00, 0x00),
        DeviceType::Unknown
    ));
}

#[test]
fn spoof_storage_with_wrong_class() {
    let info = make_info(DeviceType::Storage, 0x03, 0x01, 0x01);
    assert!(detect_spoof(&info));
}

#[test]
fn legitimate_storage_not_spoof() {
    let info = make_info(DeviceType::Storage, 0x08, 0x06, 0x50);
    assert!(!detect_spoof(&info));
}

#[test]
fn interface_type_strings() {
    let storage = make_info(DeviceType::Storage, 0x08, 0x06, 0x50);
    assert_eq!(interface_type_str(&storage), "mass_storage");

    let kb = make_info(DeviceType::Keyboard, 0x03, 0x01, 0x01);
    assert_eq!(interface_type_str(&kb), "hid_keyboard");

    let mouse = make_info(DeviceType::Mouse, 0x03, 0x01, 0x02);
    assert_eq!(interface_type_str(&mouse), "hid_mouse");
}

#[test]
fn admission_status_addable_when_in_whitelist() {
    let info = make_info(DeviceType::Storage, 0x08, 0x06, 0x50);
    assert_eq!(admission_status_str(&info, true, false), "addable");
}

#[test]
fn admission_status_blocked_when_not_in_whitelist() {
    let info = make_info(DeviceType::Storage, 0x08, 0x06, 0x50);
    assert_eq!(admission_status_str(&info, false, false), "blocked");
}

#[test]
fn admission_status_spoof_overrides_all() {
    let info = make_info(DeviceType::Storage, 0x08, 0x06, 0x50);
    assert_eq!(admission_status_str(&info, true, true), "spoof_suspected");
}
