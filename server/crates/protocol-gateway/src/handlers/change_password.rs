//! CMD_CHANGE_PASSWORD (0x0605) handler。

use prost::Message;
use tracing::{debug, info, warn};

use common::audit_const::{action_type, log_type};
use common::code::ResultCode;
use common::proto::{CmdChangePassword, RspCommon};
use storage::model::OperationLogInsert;

use crate::codec;
use crate::context::RequestContext;

/// RSP_COMMON 消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// 修改密码 handler。
pub fn handle_change_password(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!(source_ip = %ctx.source_ip, "收到修改密码请求");

    let cmd = match CmdChangePassword::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            debug!("修改密码请求 protobuf 解码失败");
            return make_rsp(ctx.seq_id, false, ResultCode::ValidationFailed, "消息解码失败")
        }
    };

    let result = ctx.auth_service.change_password(
        &cmd.session_token,
        &cmd.old_password,
        &cmd.new_password,
        &cmd.confirm_password,
    );

    match result {
        Ok(info) => {
            let mut log = OperationLogInsert {
                op_time: 0,
                username: info.username.clone(),
                role: info.role,
                log_type: log_type::LOGIN_AUTH.into(),
                action_type: Some(action_type::PASSWORD_CHANGE.into()),
                target: Some(info.username.clone()),
                before_value: None,
                after_value: None,
                related_file: None,
                related_version: None,
                result: 0,
                fail_reason: None,
                source_ip: Some(ctx.source_ip.clone()),
                app_version: None,
                session_id: None,
                request_id: None,
                detail: None,
            };
            let _ = ctx.audit_service.log_operation(&mut log);

            info!(user = %info.username, source_ip = %ctx.source_ip, "密码修改成功");
            make_rsp(ctx.seq_id, true, ResultCode::Success, "")
        }
        Err(e) => {
            warn!(source_ip = %ctx.source_ip, reason = %e, "密码修改失败");
            let code = e.to_result_code();

            if let Some(ref session) = ctx.session {
                let mut log = OperationLogInsert {
                    op_time: 0,
                    username: session.username.clone(),
                    role: session.role,
                    log_type: log_type::LOGIN_AUTH.into(),
                    action_type: Some(action_type::PASSWORD_CHANGE.into()),
                    target: Some(session.username.clone()),
                    before_value: None,
                    after_value: None,
                    related_file: None,
                    related_version: None,
                    result: 1,
                    fail_reason: Some(format!("{}", e)),
                    source_ip: Some(ctx.source_ip.clone()),
                    app_version: None,
                    session_id: None,
                    request_id: None,
                    detail: None,
                };
                let _ = ctx.audit_service.log_operation(&mut log);
            }

            make_rsp(ctx.seq_id, false, code, &format!("{}", e))
        }
    }
}

/// 构造 RspCommon 响应帧。
fn make_rsp(seq_id: u32, success: bool, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspCommon {
        success,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}
