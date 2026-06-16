//! CMD_AUTH_STATUS_QUERY (0x0003) handler。

use prost::Message;

use common::proto::{CmdAuthStatusQuery, RspAuthStatus};

use crate::codec;
use crate::context::RequestContext;

/// RSP_AUTH_STATUS 消息类型。
const RSP_AUTH_STATUS: u32 = 0x0004;

/// 授权状态查询 handler。
///
/// 此 cmd 在中间件白名单中，handler 自行校验 session_token。
/// 返回当前授权状态（P02 阶段固定返回 unauthorized，P03 授权模块实现后再对接）。
pub fn handle_auth_status(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdAuthStatusQuery::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return make_rsp(ctx.seq_id, false, 0, "", "unauthorized");
        }
    };

    // handler 自行校验 token
    let token_valid = ctx
        .auth_service
        .validate_token(&cmd.session_token)
        .is_ok();

    if token_valid {
        ctx.auth_service.refresh_token(&cmd.session_token);
    }

    let rsp = RspAuthStatus {
        authorized: false,
        expire_time: 0,
        device_description: String::new(),
        auth_status: if token_valid {
            "unauthorized".to_string()
        } else {
            "failed".to_string()
        },
    };

    let payload = rsp.encode_to_vec();
    codec::encode_frame(RSP_AUTH_STATUS, ctx.seq_id, &payload).unwrap_or_default()
}

/// 构造 RspAuthStatus 响应帧。
fn make_rsp(
    seq_id: u32,
    authorized: bool,
    expire_time: i64,
    device_desc: &str,
    auth_status: &str,
) -> Vec<u8> {
    let rsp = RspAuthStatus {
        authorized,
        expire_time,
        device_description: device_desc.to_string(),
        auth_status: auth_status.to_string(),
    };
    codec::encode_frame(RSP_AUTH_STATUS, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}
