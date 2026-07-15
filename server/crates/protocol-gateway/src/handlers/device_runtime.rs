//! CMD_GET_DEVICE_RUNTIME_STATUS handler。

use prost::Message;
use tracing::debug;

use common::code::ResultCode;
use common::proto::{
    CmdGetDeviceRuntimeStatus, DeviceRuntimeStatus, RspCommon, RspDeviceRuntimeStatus,
};

use crate::codec;
use crate::context::RequestContext;

const RSP_DEVICE_RUNTIME_STATUS: u32 = 0x0407;
const RSP_COMMON: u32 = 0xFF00;

/// 处理受控设备运行态查询。
pub fn handle_get_device_runtime_status(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!("收到设备运行态查询请求");

    let _cmd = match CmdGetDeviceRuntimeStatus::decode(payload) {
        Ok(cmd) => cmd,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let registry = match ctx.device_runtime_registry.as_ref() {
        Some(registry) => registry,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "设备运行态未初始化"),
    };

    let devices = registry
        .list()
        .into_iter()
        .map(|item| DeviceRuntimeStatus {
            runtime_id: item.runtime_id,
            parent_path: item.parent_path,
            interface_path: item.interface_path,
            serial_number: item.serial_number,
            device_name: item.device_name,
            device_type: item.device_type,
            interface_type: item.interface_type,
            status: item.status,
            stage: item.stage,
            fail_code: item.fail_code,
            fail_reason: item.fail_reason,
            connected_at: item.connected_at,
            updated_at: item.updated_at,
        })
        .collect::<Vec<_>>();

    debug!(count = devices.len(), "设备运行态查询成功");

    let rsp = RspDeviceRuntimeStatus { devices };
    codec::encode_frame(RSP_DEVICE_RUNTIME_STATUS, ctx.seq_id, &rsp.encode_to_vec())
        .unwrap_or_default()
}
fn error_response(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspCommon {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}
