//! CMD_GET_CONNECTED_DEVICES handler。

use prost::Message;

use common::code::ResultCode;
use common::proto::{CmdGetConnectedDevices, ConnectedDevice, RspConnectedDevices};
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
        None => {
            return error_response(
                ctx.seq_id,
                ResultCode::InternalError,
                "设备管理器未初始化",
            )
        }
    };

    let dm_guard = match dm.read() {
        Ok(g) => g,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::InternalError, "设备管理器锁获取失败")
        }
    };

    let wl_mgr = ctx.whitelist_manager.as_ref();

    let devices: Vec<ConnectedDevice> = dm_guard
        .list_all()
        .iter()
        .map(|record| {
            let info = &record.info;
            let is_spoof = detect_spoof(info);
            let is_in_whitelist = wl_mgr
                .map(|m| m.is_whitelisted(&info.serial_number).is_some())
                .unwrap_or(false);

            ConnectedDevice {
                serial_number: info.serial_number.clone(),
                device_name: info.device_name.clone(),
                vid: info.vid.clone(),
                pid: info.pid.clone(),
                capacity_bytes: info.capacity_bytes.unwrap_or(0),
                device_type: format!("{:?}", info.device_type).to_lowercase(),
                interface_type: interface_type_str(info).to_string(),
                admission_status: admission_status_str(info, is_in_whitelist, is_spoof)
                    .to_string(),
                fail_reason: String::new(),
            }
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
