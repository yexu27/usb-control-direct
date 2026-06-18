//! 用户管理 handler（0x0600/0x0602/0x0603/0x0604）。

use prost::Message;

use auth_session::session::SessionInfo;
use common::code::ResultCode;
use common::mapping::{role_int_to_str, role_str_to_int};
use common::proto::{
    CmdCreateUser, CmdDeleteUser, CmdListUsers, CmdResetPassword, RspCommon, RspListUsers,
    UserItem,
};
use storage::model::OperationLogInsert;

use crate::codec;
use crate::context::RequestContext;

/// RspListUsers 消息类型。
const RSP_LIST_USERS: u32 = 0x0601;

/// RspCommon 消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// CMD_LIST_USERS (0x0600) handler。
pub fn handle_list_users(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let _cmd = match CmdListUsers::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    match ctx.auth_service.list_users() {
        Ok(users) => {
            let items: Vec<UserItem> = users.iter().map(map_user_item).collect();
            let rsp = RspListUsers { users: items };
            codec::encode_frame(RSP_LIST_USERS, ctx.seq_id, &rsp.encode_to_vec())
                .unwrap_or_default()
        }
        Err(e) => error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string()),
    }
}

/// CMD_CREATE_USER (0x0602) handler。
pub fn handle_create_user(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdCreateUser::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = ctx.session_required();

    let role = match role_str_to_int(&cmd.role) {
        Ok(r) => r,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "无效的角色");
        }
    };

    match ctx
        .auth_service
        .create_user(&cmd.username, role, &cmd.password, &cmd.confirm_password)
    {
        Ok(_id) => {
            log_operation(ctx, session, "create", &cmd.username, 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => {
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                "create",
                &cmd.username,
                1,
                Some(&e.to_string()),
            );
            error_response(ctx.seq_id, code, "用户创建失败")
        }
    }
}

/// CMD_DELETE_USER (0x0603) handler。
pub fn handle_delete_user(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdDeleteUser::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = ctx.session_required();

    match ctx
        .auth_service
        .delete_user(&cmd.username, session.user_id)
    {
        Ok(()) => {
            log_operation(ctx, session, "delete", &cmd.username, 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => {
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                "delete",
                &cmd.username,
                1,
                Some(&e.to_string()),
            );
            error_response(ctx.seq_id, code, "用户删除失败")
        }
    }
}

/// CMD_RESET_PASSWORD (0x0604) handler。
pub fn handle_reset_password(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdResetPassword::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = ctx.session_required();

    match ctx
        .auth_service
        .reset_password(&cmd.username, &cmd.new_password, &cmd.confirm_password)
    {
        Ok(()) => {
            log_operation(ctx, session, "reset_password", &cmd.username, 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => {
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                "reset_password",
                &cmd.username,
                1,
                Some(&e.to_string()),
            );
            error_response(ctx.seq_id, code, "密码重置失败")
        }
    }
}

/// User → UserItem 转换。
fn map_user_item(user: &storage::model::User) -> UserItem {
    let role_str = role_int_to_str(user.role)
        .unwrap_or("unknown")
        .to_string();
    let status_str = match user.status {
        0 => "active",
        1 => "locked",
        2 => "deleted",
        _ => "unknown",
    }
    .to_string();

    UserItem {
        username: user.username.clone(),
        role: role_str,
        status: status_str,
        is_builtin: user.is_builtin != 0,
        created_at: user.created_at,
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
        log_type: "user_management".into(),
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
