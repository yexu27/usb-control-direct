//! CMD_CHANGE_PASSWORD (0x0605) handler。

use prost::Message;

use common::code::ResultCode;
use common::proto::{CmdChangePassword, RspCommon};
use storage::model::OperationLogInsert;

use crate::codec;
use crate::context::RequestContext;

/// RSP_COMMON 消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// 修改密码 handler。
pub fn handle_change_password(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdChangePassword::decode(payload) {
        Ok(c) => c,
        Err(_) => {
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
                log_type: "login_auth".into(),
                action_type: Some("change_password".into()),
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

            make_rsp(ctx.seq_id, true, ResultCode::Success, "")
        }
        Err(e) => {
            let code = e.to_result_code();

            if let Some(ref session) = ctx.session {
                let mut log = OperationLogInsert {
                    op_time: 0,
                    username: session.username.clone(),
                    role: session.role,
                    log_type: "login_auth".into(),
                    action_type: Some("change_password".into()),
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
