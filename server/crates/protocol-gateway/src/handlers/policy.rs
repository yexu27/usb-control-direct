//! 策略导入导出 handler（0x0300/0x0302）。

use prost::Message;

use auth_session::session::SessionInfo;
use common::code::ResultCode;
use common::proto::{CmdExportPolicy, CmdImportPolicy, RspCommon, RspExportPolicy};
use storage::model::OperationLogInsert;

use crate::codec;
use crate::context::RequestContext;

/// RspExportPolicy 消息类型。
const RSP_EXPORT_POLICY: u32 = 0x0301;

/// RspCommon 消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// CMD_EXPORT_POLICY (0x0300) handler。
pub fn handle_export_policy(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let _cmd = match CmdExportPolicy::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return export_error(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = ctx.session_required();

    let policy_service = match ctx.policy_service.as_ref() {
        Some(s) => s,
        None => {
            return export_error(ctx.seq_id, ResultCode::InternalError, "策略服务未初始化");
        }
    };

    match policy_service.export_policy() {
        Ok(data) => {
            log_operation(ctx, session, "export", "策略配置", 0, None);
            let rsp = RspExportPolicy {
                success: true,
                policy_data: data,
                result_code: ResultCode::Success.as_u16() as i32,
                error_message: String::new(),
            };
            codec::encode_frame(RSP_EXPORT_POLICY, ctx.seq_id, &rsp.encode_to_vec())
                .unwrap_or_default()
        }
        Err(e) => {
            let code = e.to_result_code();
            log_operation(ctx, session, "export", "策略配置", 1, Some(&e.to_string()));
            export_error(ctx.seq_id, code, "策略导出失败")
        }
    }
}

/// CMD_IMPORT_POLICY (0x0302) handler。
pub fn handle_import_policy(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdImportPolicy::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = ctx.session_required();

    let policy_service = match ctx.policy_service.as_ref() {
        Some(s) => s,
        None => {
            return error_response(ctx.seq_id, ResultCode::InternalError, "策略服务未初始化");
        }
    };

    if cmd.policy_data.is_empty() {
        return error_response(ctx.seq_id, ResultCode::ValidationFailed, "策略数据不能为空");
    }

    match policy_service.import_policy(&cmd.policy_data) {
        Ok(()) => {
            log_operation(ctx, session, "import", "策略配置", 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => {
            let code = e.to_result_code();
            log_operation(ctx, session, "import", "策略配置", 1, Some(&e.to_string()));
            error_response(ctx.seq_id, code, "策略导入失败")
        }
    }
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
        log_type: "policy_management".into(),
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

/// 构造 RspExportPolicy 错误响应。
fn export_error(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspExportPolicy {
        success: false,
        policy_data: Vec::new(),
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_EXPORT_POLICY, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
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
