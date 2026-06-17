//! S05 白名单协议 handler。

use prost::Message;

use common::code::ResultCode;
use common::mapping::{
    add_method_int_to_str, add_method_str_to_int, permission_int_to_str, permission_str_to_int,
};
use common::proto::{
    CmdAddWhitelist, CmdListWhitelist, CmdRemoveWhitelist, CmdUpdateWhitelist, RspCommon,
    RspListWhitelist, WhitelistDevice,
};
use storage::model::OperationLogInsert;
use whitelist::service::AddWhitelistRequest;

use crate::codec;
use crate::context::RequestContext;

const RSP_LIST_WHITELIST: u32 = 0x0101;
const RSP_COMMON: u32 = 0xFF00;

/// CMD_LIST_WHITELIST (0x0100)。
pub fn handle_list_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let _cmd = match CmdListWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    let list = match mgr.query_all() {
        Ok(l) => l,
        Err(e) => return error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string()),
    };

    let devices: Vec<WhitelistDevice> = list
        .iter()
        .map(|item| WhitelistDevice {
            serial_number: item.serial_number.clone(),
            vid: item.vid.clone().unwrap_or_default(),
            pid: item.pid.clone().unwrap_or_default(),
            device_name: item.device_name.clone().unwrap_or_default(),
            capacity_bytes: item.capacity_bytes.unwrap_or(0),
            permission: permission_int_to_str(item.permission)
                .unwrap_or("readonly")
                .to_string(),
            description: item.description.clone().unwrap_or_default(),
            add_method: add_method_int_to_str(item.add_method)
                .unwrap_or("device")
                .to_string(),
            created_at: item.created_at,
            device_type: item.device_type.clone(),
        })
        .collect();

    let rsp = RspListWhitelist { devices };
    codec::encode_frame(RSP_LIST_WHITELIST, ctx.seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

/// CMD_ADD_WHITELIST (0x0104)。
pub fn handle_add_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdAddWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    let permission = match permission_str_to_int(&cmd.permission) {
        Ok(p) => p,
        Err(_) => {
            return error_response(
                ctx.seq_id,
                ResultCode::ValidationFailed,
                &format!("无效的权限值: {}", cmd.permission),
            )
        }
    };

    let add_method = match add_method_str_to_int(&cmd.add_method) {
        Ok(m) => m,
        Err(_) => {
            return error_response(
                ctx.seq_id,
                ResultCode::ValidationFailed,
                &format!("无效的添加方式: {}", cmd.add_method),
            )
        }
    };

    let serial_number = cmd.serial_number.clone();
    let req = AddWhitelistRequest {
        serial_number: cmd.serial_number.clone(),
        vid: if cmd.vid.is_empty() { None } else { Some(cmd.vid) },
        pid: if cmd.pid.is_empty() { None } else { Some(cmd.pid) },
        device_name: if cmd.device_name.is_empty() {
            None
        } else {
            Some(cmd.device_name)
        },
        capacity_bytes: if cmd.capacity_bytes == 0 {
            None
        } else {
            Some(cmd.capacity_bytes)
        },
        device_type: if cmd.device_type.is_empty() {
            "storage".to_string()
        } else {
            cmd.device_type
        },
        description: if cmd.description.is_empty() {
            None
        } else {
            Some(cmd.description)
        },
        permission,
        add_method,
    };

    match mgr.add(req) {
        Ok(_id) => {
            write_audit_log(ctx, "whitelist_add", "add", Some(&serial_number), 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => error_response(ctx.seq_id, e.to_result_code(), &e.to_string()),
    }
}

/// CMD_REMOVE_WHITELIST (0x0105)。
pub fn handle_remove_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdRemoveWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    match mgr.remove(&cmd.serial_number) {
        Ok(()) => {
            write_audit_log(
                ctx,
                "whitelist_remove",
                "remove",
                Some(&cmd.serial_number),
                0,
                None,
            );
            success_response(ctx.seq_id)
        }
        Err(e) => error_response(ctx.seq_id, e.to_result_code(), &e.to_string()),
    }
}

/// CMD_UPDATE_WHITELIST (0x0106)。
pub fn handle_update_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdUpdateWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    let permission = if cmd.permission.is_empty() {
        None
    } else {
        match permission_str_to_int(&cmd.permission) {
            Ok(p) => Some(p),
            Err(_) => {
                return error_response(
                    ctx.seq_id,
                    ResultCode::ValidationFailed,
                    &format!("无效的权限值: {}", cmd.permission),
                )
            }
        }
    };

    let description = if cmd.description.is_empty() {
        None
    } else {
        Some(cmd.description.as_str())
    };

    match mgr.update(&cmd.serial_number, permission, description) {
        Ok(()) => {
            write_audit_log(
                ctx,
                "whitelist_update",
                "update",
                Some(&cmd.serial_number),
                0,
                None,
            );
            success_response(ctx.seq_id)
        }
        Err(e) => error_response(ctx.seq_id, e.to_result_code(), &e.to_string()),
    }
}

/// 写审计日志辅助函数。
fn write_audit_log(
    ctx: &RequestContext,
    log_type: &str,
    action_type: &str,
    target: Option<&str>,
    result: i32,
    fail_reason: Option<&str>,
) {
    let mut log = OperationLogInsert {
        op_time: 0,
        username: ctx
            .session
            .as_ref()
            .map(|s| s.username.clone())
            .unwrap_or_default(),
        role: ctx.session.as_ref().map(|s| s.role).unwrap_or(-1),
        log_type: log_type.into(),
        action_type: Some(action_type.into()),
        target: target.map(|t| t.to_string()),
        before_value: None,
        after_value: None,
        related_file: None,
        related_version: None,
        result,
        fail_reason: fail_reason.map(|r| r.to_string()),
        source_ip: Some(ctx.source_ip.clone()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: None,
    };
    let _ = ctx.audit_service.log_operation(&mut log);
}

fn success_response(seq_id: u32) -> Vec<u8> {
    let rsp = RspCommon {
        success: true,
        result_code: ResultCode::Success.as_u16() as i32,
        error_message: String::new(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

fn error_response(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspCommon {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}
