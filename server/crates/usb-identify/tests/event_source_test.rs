use std::sync::atomic::Ordering;

use common::types::DeviceType;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::event_source::{
    device_event_from_info, is_usb_interface_devtype, should_forward_usb_event,
    SubscriberStopToken, UsbEventSource,
};
use usb_identify::orchestrator::DeviceEvent;

fn info(device_type: DeviceType) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.2/2-1.2:1.0".into(),
        dev_path: Some("/dev/sda1".into()),
        serial_number: "04020026010624131314".into(),
        vid: "0781".into(),
        pid: "5567".into(),
        device_name: "Cruzer Blade".into(),
        device_type,
        interface_class: 8,
        interface_subclass: 6,
        interface_protocol: 80,
        capacity_bytes: Some(1024),
    }
}

#[test]
fn forwards_usb_interface_add_and_remove_only() {
    assert!(should_forward_usb_event("add", "usb_interface"));
    assert!(should_forward_usb_event("remove", "usb_interface"));

    assert!(!should_forward_usb_event("add", "usb_device"));
    assert!(!should_forward_usb_event("remove", "usb_device"));
    assert!(!should_forward_usb_event("bind", "usb_interface"));
    assert!(!should_forward_usb_event("unbind", "usb_interface"));
    assert!(!should_forward_usb_event("change", "usb_interface"));
    assert!(!should_forward_usb_event("remove", ""));
    assert!(!should_forward_usb_event("add", ""));
}

#[test]
fn converts_device_type_to_internal_device_event() {
    assert!(matches!(
        device_event_from_info(info(DeviceType::Storage)),
        DeviceEvent::StorageAdded(_)
    ));
    assert!(matches!(
        device_event_from_info(info(DeviceType::Keyboard)),
        DeviceEvent::KeyboardAdded(_)
    ));
    assert!(matches!(
        device_event_from_info(info(DeviceType::Mouse)),
        DeviceEvent::MouseAdded(_)
    ));
    assert!(matches!(
        device_event_from_info(info(DeviceType::Unknown)),
        DeviceEvent::UnsupportedAdded(_, _)
    ));
}

#[test]
fn unsupported_interface_maps_to_unsupported_added_event() {
    let info = UsbDeviceInfo {
        sys_path: "/sys/devices/platform/fd880000.usb/usb2/2-1/2-1.9/2-1.9:1.1".into(),
        dev_path: None,
        serial_number: "SN-PHONE".into(),
        vid: "18d1".into(),
        pid: "4ee7".into(),
        device_name: "Phone Vendor".into(),
        device_type: DeviceType::Unsupported,
        interface_class: 0xff,
        interface_subclass: 0x42,
        interface_protocol: 0x01,
        capacity_bytes: None,
    };

    assert!(matches!(
        device_event_from_info(info),
        DeviceEvent::UnsupportedAdded(_, _)
    ));
}

#[test]
fn enumerator_accepts_usb_interface_only() {
    assert!(is_usb_interface_devtype("usb_interface"));
    assert!(!is_usb_interface_devtype("usb_device"));
    assert!(!is_usb_interface_devtype(""));
}

#[test]
fn subscriber_stop_token_tracks_shutdown_request() {
    let token = SubscriberStopToken::new();
    assert!(!token.is_stopped());

    let cloned = token.clone();
    cloned.stop();

    assert!(token.is_stopped());
    assert!(token.inner().load(Ordering::SeqCst));
}

#[tokio::test]
async fn usb_event_source_stop_is_idempotent_before_start() {
    let source = UsbEventSource::new();

    source.stop();
    source.stop();

    assert!(source.is_stopped());
}
