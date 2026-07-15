//! S05 白名单协议 handler。

use prost::Message;
use tracing::{debug, info, warn};

use super::audit_helper::{log_operation_from_ctx, OperationDetail};
use common::audit_const::{action_type, log_type};
use common::code::ResultCode;
use common::mapping::{
    add_method_int_to_str, add_method_str_to_int, permission_int_to_str, permission_str_to_int,
};
use common::proto::{
    CmdAddWhitelist, CmdListWhitelist, CmdRemoveWhitelist, CmdUpdateWhitelist, RspCommon,
    RspListWhitelist, WhitelistDevice,
};
use common::types::DeviceType;
use usb_identify::descriptor::{admission_status_str, detect_spoof, interface_type_str};
use whitelist::service::AddWhitelistRequest;
use whitelist::WhitelistError;

use crate::codec;
use crate::context::RequestContext;

const RSP_LIST_WHITELIST: u32 = 0x0101;
const RSP_COMMON: u32 = 0xFF00;

/// CMD_LIST_WHITELIST (0x0100)。
pub fn handle_list_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!("收到白名单列表查询请求");
    let _cmd = match CmdListWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    let list = match mgr.query_all() {
        Ok(l) => {
            debug!(count = l.len(), "白名单列表查询成功");
            l
        }
        Err(e) => {
            warn!(reason = %e, "白名单列表查询失败");
            return error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string());
        }
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

    if cmd.serial_number.trim().is_empty() {
        return error_response(ctx.seq_id, ResultCode::SerialNumberEmpty, "序列号不能为空");
    }

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
    debug!(sn = %serial_number, method = %cmd.add_method, "收到白名单添加请求");
    let description = (!cmd.description.is_empty()).then(|| cmd.description.clone());
    let add_result = if add_method == 0 {
        let dm = match ctx.device_manager.as_ref() {
            Some(manager) => manager,
            None => {
                return error_response(ctx.seq_id, ResultCode::InternalError, "设备管理器未初始化")
            }
        };
        let dm_guard = match dm.read() {
            Ok(guard) => guard,
            Err(_) => {
                return error_response(
                    ctx.seq_id,
                    ResultCode::InternalError,
                    "设备管理器锁获取失败",
                )
            }
        };
        let info = match dm_guard.connected_device_by_serial(&cmd.serial_number) {
            Some(info) => info,
            None => {
                return error_response(
                    ctx.seq_id,
                    ResultCode::ValidationFailed,
                    "设备已移除，请重新插入后再添加",
                )
            }
        };
        let is_in_whitelist = mgr.is_whitelisted(&info.serial_number).is_some();
        let is_spoof = detect_spoof(info);
        if is_spoof {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceSpoofSuspected,
                "设备描述符异常，疑似伪装设备，禁止添加",
            );
        }
        if matches!(
            info.device_type,
            DeviceType::Unknown | DeviceType::Unsupported
        ) {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceUnsupported,
                "不支持的USB设备类型，无法添加",
            );
        }
        if info.device_type != DeviceType::Storage || interface_type_str(info) != "mass_storage" {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceNotStorage,
                "仅支持添加大容量存储设备",
            );
        }
        if admission_status_str(info, is_in_whitelist, is_spoof) != "addable" {
            return error_response(ctx.seq_id, ResultCode::AlreadyExists, "该设备已在白名单中");
        }

        let req = AddWhitelistRequest {
            serial_number: cmd.serial_number.clone(),
            vid: (!info.vid.is_empty()).then(|| info.vid.clone()),
            pid: (!info.pid.is_empty()).then(|| info.pid.clone()),
            device_name: (!info.device_name.is_empty()).then(|| info.device_name.clone()),
            capacity_bytes: info.capacity_bytes,
            device_type: "storage".to_string(),
            description: description.clone(),
            permission,
            add_method,
        };
        mgr.add(req)
    } else {
        if cmd.device_type != "storage" {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceNotStorage,
                "仅支持添加大容量存储设备",
            );
        }
        let req = AddWhitelistRequest {
            serial_number: cmd.serial_number.clone(),
            vid: (!cmd.vid.is_empty()).then(|| cmd.vid.clone()),
            pid: (!cmd.pid.is_empty()).then(|| cmd.pid.clone()),
            device_name: (!cmd.device_name.is_empty()).then(|| cmd.device_name.clone()),
            capacity_bytes: (cmd.capacity_bytes != 0).then_some(cmd.capacity_bytes),
            device_type: cmd.device_type.clone(),
            description,
            permission,
            add_method,
        };
        mgr.add(req)
    };

    match add_result {
        Ok(_id) => {
            info!(sn = %serial_number, method = cmd.add_method, "白名单添加成功");
            log_operation_from_ctx(
                ctx,
                log_type::SECURITY_CONFIG,
                action_type::WHITELIST_ADD,
                Some(&serial_number),
                0,
                None,
                &OperationDetail::default(),
            );
            success_response(ctx.seq_id)
        }
        Err(WhitelistError::AlreadyExists(_)) => {
            info!(sn = %serial_number, "白名单添加失败：设备已存在");
            log_operation_from_ctx(
                ctx,
                log_type::SECURITY_CONFIG,
                action_type::WHITELIST_ADD,
                Some(&serial_number),
                1,
                Some("该设备已在白名单中"),
                &OperationDetail::default(),
            );
            error_response(ctx.seq_id, ResultCode::AlreadyExists, "该设备已在白名单中")
        }
        Err(e) => {
            warn!(sn = %serial_number, reason = %e, "白名单添加失败");
            log_operation_from_ctx(
                ctx,
                log_type::SECURITY_CONFIG,
                action_type::WHITELIST_ADD,
                Some(&serial_number),
                1,
                Some(&e.to_string()),
                &OperationDetail::default(),
            );
            error_response(ctx.seq_id, e.to_result_code(), &e.to_string())
        }
    }
}

/// CMD_REMOVE_WHITELIST (0x0105)。
pub fn handle_remove_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdRemoveWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    debug!(sn = %cmd.serial_number, "收到白名单删除请求");

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    match mgr.remove(&cmd.serial_number) {
        Ok(()) => {
            info!(sn = %cmd.serial_number, "白名单删除成功");
            log_operation_from_ctx(
                ctx,
                log_type::SECURITY_CONFIG,
                action_type::WHITELIST_REMOVE,
                Some(&cmd.serial_number),
                0,
                None,
                &OperationDetail::default(),
            );
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(sn = %cmd.serial_number, reason = %e, "白名单删除失败");
            log_operation_from_ctx(
                ctx,
                log_type::SECURITY_CONFIG,
                action_type::WHITELIST_REMOVE,
                Some(&cmd.serial_number),
                1,
                Some(&e.to_string()),
                &OperationDetail::default(),
            );
            error_response(ctx.seq_id, e.to_result_code(), &e.to_string())
        }
    }
}

/// CMD_UPDATE_WHITELIST (0x0106)。
pub fn handle_update_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdUpdateWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    debug!(sn = %cmd.serial_number, "收到白名单更新请求");

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

    // 查询变更前的权限值，用于审计日志。
    let old_permission = mgr
        .query_by_sn(&cmd.serial_number)
        .ok()
        .flatten()
        .map(|item| item.permission);

    match mgr.update(&cmd.serial_number, permission, description) {
        Ok(()) => {
            info!(sn = %cmd.serial_number, "白名单更新成功");
            let ext = OperationDetail {
                before_value: old_permission.map(|p| {
                    format!(
                        r#"{{"permission":"{}"}}"#,
                        permission_int_to_str(p).unwrap_or("unknown")
                    )
                }),
                after_value: permission.map(|p| {
                    format!(
                        r#"{{"permission":"{}"}}"#,
                        permission_int_to_str(p).unwrap_or("unknown")
                    )
                }),
                ..Default::default()
            };
            log_operation_from_ctx(
                ctx,
                log_type::SECURITY_CONFIG,
                action_type::WHITELIST_UPDATE,
                Some(&cmd.serial_number),
                0,
                None,
                &ext,
            );
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(sn = %cmd.serial_number, reason = %e, "白名单更新失败");
            log_operation_from_ctx(
                ctx,
                log_type::SECURITY_CONFIG,
                action_type::WHITELIST_UPDATE,
                Some(&cmd.serial_number),
                1,
                Some(&e.to_string()),
                &OperationDetail::default(),
            );
            error_response(ctx.seq_id, e.to_result_code(), &e.to_string())
        }
    }
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
