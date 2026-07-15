mod support;

use common::proto::{CmdGetConnectedDevices, RspConnectedDevices};
use prost::Message;
use protocol_gateway::codec;
use protocol_gateway::handlers::connected_devices::handle_get_connected_devices;
use std::sync::{Arc, RwLock};
use support::{request_fixture, RequestFixture};

use common::types::DeviceType;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::monitor::DeviceManager;
use whitelist::service::AddWhitelistRequest;
use whitelist::WhitelistManager;

fn device(serial_number: &str, device_type: DeviceType, interface_class: u8) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: format!("/sys/{serial_number}"),
        dev_path: Some(format!("/dev/{serial_number}")),
        serial_number: serial_number.to_string(),
        vid: "0951".to_string(),
        pid: "1666".to_string(),
        device_name: serial_number.to_string(),
        device_type,
        interface_class,
        interface_subclass: 0x06,
        interface_protocol: 0x50,
        capacity_bytes: Some(1024),
    }
}

fn context() -> RequestFixture {
    let mut fixture = request_fixture(7);
    let whitelist = Arc::new(WhitelistManager::new(Arc::clone(&fixture.storage)).unwrap());
    whitelist
        .add(AddWhitelistRequest {
            serial_number: "WHITELISTED".into(),
            vid: None,
            pid: None,
            device_name: None,
            capacity_bytes: None,
            device_type: "storage".into(),
            description: None,
            permission: 0,
            add_method: 1,
        })
        .unwrap();

    let mut manager = DeviceManager::new();
    manager.add_interface(device("ADDABLE", DeviceType::Storage, 0x08));
    manager.add_interface(device("KEYBOARD", DeviceType::Keyboard, 0x03));
    manager.add_interface(device("UNKNOWN", DeviceType::Unknown, 0xff));
    manager.add_interface(device("WHITELISTED", DeviceType::Storage, 0x08));
    manager.add_interface(device("SPOOF", DeviceType::Storage, 0x03));
    manager.add_interface(device("   ", DeviceType::Storage, 0x08));

    fixture.context.whitelist_manager = Some(whitelist);
    fixture.context.device_manager = Some(Arc::new(RwLock::new(manager)));
    fixture
}

#[test]
fn returns_only_addable_mass_storage_devices() {
    let fixture = context();
    let ctx = fixture.context;
    let response = handle_get_connected_devices(
        &ctx,
        &CmdGetConnectedDevices {
            session_token: String::new(),
        }
        .encode_to_vec(),
    );
    let (_, payload, _) = codec::try_decode_frame(&response).unwrap().unwrap();
    let rsp = RspConnectedDevices::decode(payload.as_slice()).unwrap();

    assert_eq!(rsp.devices.len(), 1);
    assert_eq!(rsp.devices[0].serial_number, "ADDABLE");
    assert_eq!(rsp.devices[0].device_type, "storage");
    assert_eq!(rsp.devices[0].interface_type, "mass_storage");
    assert_eq!(rsp.devices[0].admission_status, "addable");
    assert!(rsp.devices[0].fail_reason.is_empty());
}
