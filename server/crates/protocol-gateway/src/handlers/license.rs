//! 授权管理 handler（0x0005/0x0007）。

use prost::Message;

use auth_session::session::SessionInfo;
use common::code::ResultCode;
use common::proto::{
    CmdGetMachineCode, CmdUploadLicense, RspCommon, RspMachineCode, RspUploadLicense,
};
use storage::model::OperationLogInsert;

use crate::codec;
use crate::context::RequestContext;

/// RspMachineCode 消息类型。
const RSP_MACHINE_CODE: u32 = 0x0006;

/// RspUploadLicense 消息类型。
const RSP_UPLOAD_LICENSE: u32 = 0x0008;

/// RspCommon 消息类型（用于错误响应回退）。
const RSP_COMMON: u32 = 0xFF00;

/// CMD_GET_MACHINE_CODE (0x0005) handler。
pub fn handle_get_machine_code(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let _cmd = match CmdGetMachineCode::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return machine_code_error(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session.as_ref() {
        Some(s) => s,
        None => {
            return machine_code_error(ctx.seq_id, ResultCode::Unauthenticated, "未登录");
        }
    };

    // 已授权状态下仅管理员可操作
    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return machine_code_error(ctx.seq_id, ResultCode::InternalError, "存储未初始化");
        }
    };
    if is_device_authorized(storage) && session.role != 0 {
        return machine_code_error(
            ctx.seq_id,
            ResultCode::PermissionDenied,
            "已授权状态下仅管理员可操作",
        );
    }

    match license_upgrade::generate_machine_code() {
        Ok(result) => {
            log_operation(ctx, session, "get_machine_code", "机器码", 0, None);
            let rsp = RspMachineCode {
                machine_code: result.machine_code,
                qrcode_png: result.qrcode_png,
            };
            codec::encode_frame(RSP_MACHINE_CODE, ctx.seq_id, &rsp.encode_to_vec())
                .unwrap_or_default()
        }
        Err(e) => {
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                "get_machine_code",
                "机器码",
                1,
                Some(&e.to_string()),
            );
            machine_code_error(ctx.seq_id, code, &e.to_string())
        }
    }
}

/// CMD_UPLOAD_LICENSE (0x0007) handler。
pub fn handle_upload_license(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdUploadLicense::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return license_error(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session.as_ref() {
        Some(s) => s,
        None => {
            return license_error(ctx.seq_id, ResultCode::Unauthenticated, "未登录");
        }
    };

    // 已授权状态下仅管理员可操作
    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return license_error(ctx.seq_id, ResultCode::InternalError, "存储未初始化");
        }
    };
    if is_device_authorized(storage) && session.role != 0 {
        return license_error(
            ctx.seq_id,
            ResultCode::PermissionDenied,
            "已授权状态下仅管理员可操作",
        );
    }

    let validator = match ctx.license_validator.as_ref() {
        Some(v) => v,
        None => {
            return license_error(ctx.seq_id, ResultCode::InternalError, "授权校验器未初始化");
        }
    };

    if cmd.license_data.is_empty() {
        return license_error(ctx.seq_id, ResultCode::ValidationFailed, "授权文件不能为空");
    }

    // 获取当前机器码用于校验
    let machine_code = match license_upgrade::generate_machine_code() {
        Ok(r) => r.machine_code,
        Err(e) => {
            return license_error(ctx.seq_id, ResultCode::InternalError, &e.to_string());
        }
    };

    match validator.validate(&cmd.license_data, &machine_code) {
        Ok(info) => {
            // 更新授权状态到数据库
            let _ = storage.config_set("auth_status", "authorized");
            let _ = storage.config_set("auth_expire_time", &info.expire_time.to_string());

            log_operation(ctx, session, "upload_license", "授权文件", 0, None);
            let rsp = RspUploadLicense {
                success: true,
                expire_time: info.expire_time,
                result_code: ResultCode::Success.as_u16() as i32,
                error_message: String::new(),
            };
            codec::encode_frame(RSP_UPLOAD_LICENSE, ctx.seq_id, &rsp.encode_to_vec())
                .unwrap_or_default()
        }
        Err(e) => {
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                "upload_license",
                "授权文件",
                1,
                Some(&e.to_string()),
            );
            license_error(ctx.seq_id, code, &e.to_string())
        }
    }
}

/// 判断设备是否已授权。
fn is_device_authorized(storage: &storage::Storage) -> bool {
    storage
        .config_get("auth_status")
        .ok()
        .flatten()
        .map(|c| c.config_value.as_deref() == Some("authorized"))
        .unwrap_or(false)
}

/// 记录操作日志。
fn log_operation(
    ctx: &RequestContext,
    session: &SessionInfo,
    action_type: &str,
    target: &str,
    result: i32,
    fail_reason: Option<&str>,
) {
    let mut log = OperationLogInsert {
        op_time: 0,
        username: session.username.clone(),
        role: session.role,
        log_type: "system_management".into(),
        action_type: Some(action_type.into()),
        target: Some(target.into()),
        before_value: None,
        after_value: None,
        related_file: None,
        related_version: None,
        result,
        fail_reason: fail_reason.map(|s| s.to_string()),
        source_ip: Some(ctx.source_ip.clone()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: None,
    };
    let _ = ctx.audit_service.log_operation(&mut log);
}

/// 构造机器码错误响应（使用 RspCommon 传递错误信息）。
fn machine_code_error(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspCommon {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

/// 构造 RspUploadLicense 错误响应。
fn license_error(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspUploadLicense {
        success: false,
        expire_time: 0,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_UPLOAD_LICENSE, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}
