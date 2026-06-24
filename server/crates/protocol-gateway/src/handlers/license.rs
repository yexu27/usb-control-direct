//! 授权管理 handler（0x0005/0x0007）。

use prost::Message;
use tracing::{debug, info, warn};

use common::code::ResultCode;
use common::proto::{
    CmdGetMachineCode, CmdUploadLicense, RspCommon, RspMachineCode, RspUploadLicense,
};

use super::audit_helper::log_operation;
use super::license_state::read_license_snapshot;
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

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return machine_code_error(ctx.seq_id, code, "会话状态异常"),
    };

    debug!(user = %session.username, "收到授权管理请求");

    // 已授权状态下仅管理员可操作
    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return machine_code_error(ctx.seq_id, ResultCode::InternalError, "存储未初始化");
        }
    };
    let license = match read_license_snapshot(storage, common::time::now_unix()) {
        Ok(license) => license,
        Err(_) => {
            return machine_code_error(ctx.seq_id, ResultCode::InternalError, "授权状态读取失败");
        }
    };
    if license.authorized && session.role != 0 {
        return machine_code_error(
            ctx.seq_id,
            ResultCode::PermissionDenied,
            "已授权状态下仅管理员可操作",
        );
    }

    match license_upgrade::generate_machine_code() {
        Ok(result) => {
            info!(user = %session.username, "下载机器码");
            log_operation(ctx, session, "system_management", "get_machine_code", "机器码", 0, None);
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
                "system_management",
                "get_machine_code",
                "机器码",
                1,
                Some(&e.to_string()),
            );
            machine_code_error(ctx.seq_id, code, "机器码生成失败")
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

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return license_error(ctx.seq_id, code, "会话状态异常"),
    };

    debug!(user = %session.username, "收到授权管理请求");

    // 已授权状态下仅管理员可操作(license handler)
    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return license_error(ctx.seq_id, ResultCode::InternalError, "存储未初始化");
        }
    };
    let license = match read_license_snapshot(storage, common::time::now_unix()) {
        Ok(license) => license,
        Err(_) => {
            return license_error(ctx.seq_id, ResultCode::InternalError, "授权状态读取失败");
        }
    };
    if license.authorized && session.role != 0 {
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
        Ok(ref result) => {
            debug!("服务端真实机器码已获取");
            result.machine_code.clone()
        }
        Err(_e) => {
            return license_error(ctx.seq_id, ResultCode::InternalError, "机器码获取失败");
        }
    };

    match validator.validate(&cmd.license_data, &machine_code) {
        Ok(info) => {
            info!(user = %session.username, expire = info.expire_time, "授权文件校验成功");
            if let Err(_e) = storage.config_set("auth_status", "authorized") {
                warn!(user = %session.username, reason = "授权状态持久化失败", "授权上传持久化异常");
                log_operation(ctx, session, "system_management", "upload_license", "授权文件", 1, Some("授权状态持久化失败"));
                return license_error(ctx.seq_id, ResultCode::InternalError, "授权状态持久化失败");
            }
            if let Err(_e) = storage.config_set("auth_expire_time", &info.expire_time.to_string()) {
                warn!(user = %session.username, reason = "授权状态持久化失败", "授权上传持久化异常");
                log_operation(ctx, session, "system_management", "upload_license", "授权文件", 1, Some("授权过期时间持久化失败"));
                return license_error(ctx.seq_id, ResultCode::InternalError, "授权过期时间持久化失败");
            }

            log_operation(ctx, session, "system_management", "upload_license", "授权文件", 0, None);
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
            warn!(user = %session.username, reason = %e, "授权文件校验失败");
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                "system_management",
                "upload_license",
                "授权文件",
                1,
                Some(&e.to_string()),
            );
            license_error(ctx.seq_id, code, "授权校验失败")
        }
    }
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
