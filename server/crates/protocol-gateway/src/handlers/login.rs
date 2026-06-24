//! CMD_LOGIN (0x0001) handler。

use prost::Message;
use tracing::{debug, info, warn};

use auth_session::service::LoginResult;
use common::code::ResultCode;
use common::mapping::role_int_to_str;
use common::proto::{CmdLogin, RspLogin};
use storage::model::OperationLogInsert;

use super::license_state::{read_license_snapshot, LicenseSnapshot};
use crate::codec;
use crate::context::RequestContext;

/// RSP_LOGIN 消息类型。
const RSP_LOGIN: u32 = 0x0002;

/// 登录 handler。
pub fn handle_login(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!(source_ip = %ctx.source_ip, "收到登录请求");

    let cmd = match CmdLogin::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            debug!(source_ip = %ctx.source_ip, "登录请求 protobuf 解码失败");
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    if cmd.username.is_empty() || cmd.password.is_empty() {
        debug!(source_ip = %ctx.source_ip, "登录请求参数为空");
        return error_response(ctx.seq_id, ResultCode::ValidationFailed, "用户名或密码不能为空");
    }

    let result = ctx.auth_service.login(&cmd.username, &cmd.password, &ctx.source_ip);

    // 记录操作日志（成功/失败都记录）
    let (log_result, log_fail_reason) = match &result {
        Ok(_) => (0, None),
        Err(e) => (1, Some(format!("{}", e))),
    };

    let mut log = OperationLogInsert {
        op_time: 0,
        username: cmd.username.clone(),
        role: result.as_ref().map(|r| r.role).unwrap_or(-1),
        log_type: "login_auth".into(),
        action_type: Some("login".into()),
        target: None,
        before_value: None,
        after_value: None,
        related_file: None,
        related_version: None,
        result: log_result,
        fail_reason: log_fail_reason,
        source_ip: Some(ctx.source_ip.clone()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: None,
    };
    let _ = ctx.audit_service.log_operation(&mut log);

    match result {
        Ok(login_result) => {
            info!(user = %login_result.username, role = login_result.role, source_ip = %ctx.source_ip, "用户登录成功");
            let role_str = role_int_to_str(login_result.role)
                .ok()
                .unwrap_or("unknown");

            let storage = match ctx.storage() {
                Some(storage) => storage,
                None => {
                    let _ = ctx.auth_service.logout(&login_result.token);
                    return error_response(
                        ctx.seq_id,
                        ResultCode::InternalError,
                        "存储服务未初始化",
                    );
                }
            };
            let license = match read_license_snapshot(storage, common::time::now_unix()) {
                Ok(license) => license,
                Err(_) => {
                    let _ = ctx.auth_service.logout(&login_result.token);
                    return error_response(
                        ctx.seq_id,
                        ResultCode::InternalError,
                        "授权状态读取失败",
                    );
                }
            };

            let rsp = build_success_response(login_result, role_str, license);
            let payload = rsp.encode_to_vec();
            codec::encode_frame(RSP_LOGIN, ctx.seq_id, &payload).unwrap_or_default()
        }
        Err(e) => {
            warn!(user = %cmd.username, source_ip = %ctx.source_ip, reason = %e, "用户登录失败");
            let code = e.to_result_code();
            let rsp = RspLogin {
                success: false,
                session_token: String::new(),
                username: String::new(),
                role: String::new(),
                authorized: false,
                auth_expire_time: 0,
                device_description: String::new(),
                result_code: code.as_u16() as i32,
                error_message: format!("{}", e),
                auth_status: String::new(),
            };
            let payload = rsp.encode_to_vec();
            codec::encode_frame(RSP_LOGIN, ctx.seq_id, &payload).unwrap_or_default()
        }
    }
}

fn build_success_response(
    login_result: LoginResult,
    role: &str,
    license: LicenseSnapshot,
) -> RspLogin {
    RspLogin {
        success: true,
        session_token: login_result.token,
        username: login_result.username,
        role: role.to_string(),
        authorized: license.authorized,
        auth_expire_time: license.expire_time,
        device_description: license.device_description,
        result_code: ResultCode::Success.as_u16() as i32,
        error_message: String::new(),
        auth_status: license.status,
    }
}

/// 构造错误 RspLogin。
fn error_response(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspLogin {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
        ..Default::default()
    };
    codec::encode_frame(RSP_LOGIN, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use auth_session::service::LoginResult;

    use super::build_success_response;
    use crate::handlers::license_state::LicenseSnapshot;

    #[test]
    fn success_response_contains_real_license_snapshot() {
        let response = build_success_response(
            LoginResult {
                token: "session-token".to_string(),
                username: "admin".to_string(),
                role: 0,
            },
            "admin",
            LicenseSnapshot {
                authorized: true,
                status: "authorized".to_string(),
                expire_time: 1_893_455_999,
                device_description: "USB_DEVICE_01".to_string(),
            },
        );

        assert!(response.success);
        assert!(response.authorized);
        assert_eq!(response.auth_status, "authorized");
        assert_eq!(response.auth_expire_time, 1_893_455_999);
        assert_eq!(response.device_description, "USB_DEVICE_01");
        assert_eq!(response.session_token, "session-token");
        assert_eq!(response.role, "admin");
    }
}
