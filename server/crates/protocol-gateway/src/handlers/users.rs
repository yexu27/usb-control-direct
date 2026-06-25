//! 用户管理 handler（0x0600/0x0602/0x0603/0x0604）。

use prost::Message;
use tracing::{debug, info, warn};

use common::audit_const::{action_type, log_type};
use common::code::ResultCode;
use common::mapping::{role_int_to_str, role_str_to_int};
use common::proto::{
    CmdCreateUser, CmdDeleteUser, CmdListUsers, CmdResetPassword, RspCommon, RspListUsers,
    UserItem,
};

use super::audit_helper::log_operation;
use crate::codec;
use crate::context::RequestContext;

/// RspListUsers 消息类型。
const RSP_LIST_USERS: u32 = 0x0601;

/// RspCommon 消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// CMD_LIST_USERS (0x0600) handler。
pub fn handle_list_users(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!("收到查询用户列表请求");
    let _cmd = match CmdListUsers::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    match ctx.auth_service.list_users() {
        Ok(users) => {
            debug!(count = users.len(), "用户列表查询成功");
            let items: Vec<UserItem> = users.iter().map(map_user_item).collect();
            let rsp = RspListUsers { users: items };
            codec::encode_frame(RSP_LIST_USERS, ctx.seq_id, &rsp.encode_to_vec())
                .unwrap_or_default()
        }
        Err(e) => {
            warn!(reason = %e, "用户列表查询失败");
            error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string())
        }
    }
}

/// CMD_CREATE_USER (0x0602) handler。
pub fn handle_create_user(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!("收到创建用户请求");
    let cmd = match CmdCreateUser::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return error_response(ctx.seq_id, code, "会话状态异常"),
    };

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
            info!(user = %cmd.username, "用户创建成功");
            log_operation(ctx, session, log_type::USER_MANAGEMENT, action_type::USER_CREATE, &cmd.username, 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(user = %cmd.username, reason = %e, "用户创建失败");
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                log_type::USER_MANAGEMENT,
                action_type::USER_CREATE,
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
    debug!("收到删除用户请求");
    let cmd = match CmdDeleteUser::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return error_response(ctx.seq_id, code, "会话状态异常"),
    };

    match ctx
        .auth_service
        .delete_user(&cmd.username, session.user_id)
    {
        Ok(()) => {
            info!(target_user = %cmd.username, "用户删除成功");
            log_operation(ctx, session, log_type::USER_MANAGEMENT, action_type::USER_DELETE, &cmd.username, 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(target_user = %cmd.username, reason = %e, "用户删除失败");
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                log_type::USER_MANAGEMENT,
                action_type::USER_DELETE,
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
    debug!("收到重置密码请求");
    let cmd = match CmdResetPassword::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return error_response(ctx.seq_id, code, "会话状态异常"),
    };

    match ctx
        .auth_service
        .reset_password(&cmd.username, &cmd.new_password, &cmd.confirm_password)
    {
        Ok(()) => {
            info!(target_user = %cmd.username, "密码重置成功");
            log_operation(ctx, session, log_type::USER_MANAGEMENT, action_type::PASSWORD_RESET, &cmd.username, 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(target_user = %cmd.username, reason = %e, "密码重置失败");
            let code = e.to_result_code();
            log_operation(
                ctx,
                session,
                log_type::USER_MANAGEMENT,
                action_type::PASSWORD_RESET,
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
