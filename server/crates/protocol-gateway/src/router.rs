//! 消息路由器。
//!
//! 提供 `Router::register(cmd, handler)` 和 `register_with_roles(cmd, handler, roles)` 注册接口，
//! 以及 `dispatch` 分发。未注册的 cmd 返回 VALIDATION_FAILED + "unknown command"。

use std::collections::HashMap;

use common::code::ResultCode;
use prost::Message;

use crate::codec;
use crate::context::RequestContext;
use crate::post_send::HandlerOutcome;

/// 消息处理函数类型。
///
/// 参数:
///   - `ctx`: 请求上下文（含 seq_id、session、source_ip、共享服务）。
///   - `payload`: 请求 payload（protobuf 序列化字节）。
///
/// 返回编码好的响应帧字节。
pub type HandlerFn = Box<dyn Fn(&RequestContext, &[u8]) -> Vec<u8> + Send + Sync>;

/// 可返回发送后动作的消息处理函数类型。
pub type OutcomeHandlerFn = Box<dyn Fn(&RequestContext, &[u8]) -> HandlerOutcome + Send + Sync>;

/// 通用响应消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// 路由条目。
struct RouteEntry {
    handler: OutcomeHandlerFn,
    /// 允许的角色列表。为空表示无角色限制（白名单 cmd）。
    allowed_roles: Vec<i32>,
}

/// 消息路由器。
pub struct Router {
    handlers: HashMap<u32, RouteEntry>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// 创建空路由器。
    pub fn new() -> Self {
        Router {
            handlers: HashMap::new(),
        }
    }

    /// 注册消息处理函数（无角色限制）。
    pub fn register(&mut self, msg_type: u32, handler: HandlerFn) {
        self.handlers.insert(
            msg_type,
            RouteEntry {
                handler: wrap_ordinary_handler(handler),
                allowed_roles: Vec::new(),
            },
        );
    }

    /// 注册消息处理函数（带角色限制）。
    pub fn register_with_roles(
        &mut self,
        msg_type: u32,
        handler: HandlerFn,
        allowed_roles: Vec<i32>,
    ) {
        self.handlers.insert(
            msg_type,
            RouteEntry {
                handler: wrap_ordinary_handler(handler),
                allowed_roles,
            },
        );
    }

    /// 注册可返回发送后动作的消息处理函数（带角色限制）。
    ///
    /// 普通 handler 应继续使用 `register` 或 `register_with_roles`；该入口仅用于
    /// 必须在响应完整发送后才能启动后续工作的命令。
    pub fn register_outcome_with_roles(
        &mut self,
        msg_type: u32,
        handler: OutcomeHandlerFn,
        allowed_roles: Vec<i32>,
    ) {
        self.handlers.insert(
            msg_type,
            RouteEntry {
                handler,
                allowed_roles,
            },
        );
    }

    /// 查询 cmd 是否已注册。
    pub fn is_registered(&self, msg_type: u32) -> bool {
        self.handlers.contains_key(&msg_type)
    }

    /// 获取 cmd 所需的角色列表（空表示无限制）。
    pub fn allowed_roles(&self, msg_type: u32) -> &[i32] {
        self.handlers
            .get(&msg_type)
            .map(|e| e.allowed_roles.as_slice())
            .unwrap_or(&[])
    }

    /// 分发消息。
    ///
    /// 调用方（connection.rs）负责在调用前完成 token 和权限校验。
    pub fn dispatch(&self, ctx: &RequestContext, msg_type: u32, payload: &[u8]) -> HandlerOutcome {
        if let Some(entry) = self.handlers.get(&msg_type) {
            (entry.handler)(ctx, payload)
        } else {
            HandlerOutcome::response(self.unknown_command_response(ctx.seq_id))
        }
    }

    /// 未知命令响应。
    fn unknown_command_response(&self, seq_id: u32) -> Vec<u8> {
        let rsp = common::proto::RspCommon {
            success: false,
            result_code: ResultCode::ValidationFailed.as_u16() as i32,
            error_message: "unknown command".into(),
        };
        let payload = rsp.encode_to_vec();
        codec::encode_frame(RSP_COMMON, seq_id, &payload).unwrap_or_default()
    }
}

fn wrap_ordinary_handler(handler: HandlerFn) -> OutcomeHandlerFn {
    Box::new(move |ctx, payload| HandlerOutcome::response(handler(ctx, payload)))
}
