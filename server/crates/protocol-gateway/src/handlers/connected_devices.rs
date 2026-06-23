//! CMD_GET_CONNECTED_DEVICES handler。

use prost::Message;

use common::code::ResultCode;
use common::proto::{CmdGetConnectedDevices, ConnectedDevice, RspConnectedDevices};
use common::types::DeviceType;
use usb_identify::descriptor::{admission_status_str, detect_spoof, interface_type_str};

use crate::codec;
use crate::context::RequestContext;

const RSP_CONNECTED_DEVICES: u32 = 0x0103;

/// CMD_GET_CONNECTED_DEVICES (0x0102)。
pub fn handle_get_connected_devices(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let _cmd = match CmdGetConnectedDevices::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let dm = match ctx.device_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "设备管理器未初始化"),
    };

    let dm_guard = match dm.read() {
        Ok(g) => g,
        Err(_) => {
            return error_response(
                ctx.seq_id,
                ResultCode::InternalError,
                "设备管理器锁获取失败",
            )
        }
    };

    let wl_mgr = ctx.whitelist_manager.as_ref();

    let devices: Vec<ConnectedDevice> = dm_guard
        .list_all()
        .iter()
        .filter_map(|record| {
            let info = &record.info;
            let is_spoof = detect_spoof(info);
            let is_in_whitelist = wl_mgr
                .map(|m| m.is_whitelisted(&info.serial_number).is_some())
                .unwrap_or(false);

            if info.device_type != DeviceType::Storage
                || info.serial_number.trim().is_empty()
                || interface_type_str(info) != "mass_storage"
                || admission_status_str(info, is_in_whitelist, is_spoof) != "addable"
            {
                return None;
            }

            Some(ConnectedDevice {
                serial_number: info.serial_number.clone(),
                device_name: info.device_name.clone(),
                vid: info.vid.clone(),
                pid: info.pid.clone(),
                capacity_bytes: info.capacity_bytes.unwrap_or(0),
                device_type: format!("{:?}", info.device_type).to_lowercase(),
                interface_type: interface_type_str(info).to_string(),
                admission_status: "addable".to_string(),
                fail_reason: String::new(),
            })
        })
        .collect();

    let rsp = RspConnectedDevices { devices };
    codec::encode_frame(RSP_CONNECTED_DEVICES, ctx.seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

fn error_response(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    use common::proto::RspCommon;
    let rsp = RspCommon {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(0xFF00, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    use auth_session::{AuthService, SessionManager};
    use common::types::DeviceType;
    use log_audit::AuditService;
    use storage::Storage;
    use tempfile::{NamedTempFile, TempPath};
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

    fn context() -> (RequestContext, TempPath) {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        let auth = Arc::new(AuthService::new(
            Storage::open(&path).unwrap(),
            SessionManager::new(),
        ));
        let audit = Arc::new(AuditService::new(Storage::open(&path).unwrap(), &path));
        let whitelist = Arc::new(WhitelistManager::new(Storage::open(&path).unwrap()).unwrap());
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
        manager.add(device("ADDABLE", DeviceType::Storage, 0x08));
        manager.add(device("KEYBOARD", DeviceType::Keyboard, 0x03));
        manager.add(device("UNKNOWN", DeviceType::Unknown, 0xff));
        manager.add(device("WHITELISTED", DeviceType::Storage, 0x08));
        manager.add(device("SPOOF", DeviceType::Storage, 0x03));
        manager.add(device("   ", DeviceType::Storage, 0x08));

        (
            RequestContext {
                seq_id: 7,
                session: None,
                source_ip: "127.0.0.1".into(),
                auth_service: auth,
                audit_service: audit,
                whitelist_manager: Some(whitelist),
                device_manager: Some(Arc::new(RwLock::new(manager))),
                storage: None,
                policy_service: None,
                license_validator: None,
                system_upgrade_mgr: None,
                virusdb_upgrade_mgr: None,
            },
            path,
        )
    }

    #[test]
    fn returns_only_addable_mass_storage_devices() {
        let (ctx, _path) = context();
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
}
