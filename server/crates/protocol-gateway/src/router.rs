//! 消息路由器。
//!
//! 提供 `Router::register(cmd, handler)` 和 `register_with_roles(cmd, handler, roles)` 注册接口，
//! 以及 `dispatch` 分发。未注册的 cmd 返回 VALIDATION_FAILED + "unknown command"。

use std::collections::HashMap;

use common::code::ResultCode;
use prost::Message;

use crate::codec;
use crate::context::RequestContext;

/// 消息处理函数类型。
///
/// 参数:
///   - `ctx`: 请求上下文（含 seq_id、session、source_ip、共享服务）。
///   - `payload`: 请求 payload（protobuf 序列化字节）。
///
/// 返回编码好的响应帧字节。
pub type HandlerFn = Box<dyn Fn(&RequestContext, &[u8]) -> Vec<u8> + Send + Sync>;

/// 通用响应消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// 路由条目。
struct RouteEntry {
    handler: HandlerFn,
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
                handler,
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
    pub fn dispatch(&self, ctx: &RequestContext, msg_type: u32, payload: &[u8]) -> Vec<u8> {
        if let Some(entry) = self.handlers.get(&msg_type) {
            (entry.handler)(ctx, payload)
        } else {
            self.unknown_command_response(ctx.seq_id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use auth_session::{AuthService, SessionManager};
    use log_audit::AuditService;
    use storage::Storage;
    use tempfile::NamedTempFile;

    /// 构建测试用 RequestContext。
    fn test_context(seq_id: u32) -> (RequestContext, tempfile::TempPath) {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.into_temp_path();
        let storage_auth = Storage::open(&path).unwrap();
        let storage_audit = Storage::open(&path).unwrap();
        let auth = Arc::new(AuthService::new(storage_auth, SessionManager::new()));
        let audit = Arc::new(AuditService::new(storage_audit, &path));
        let ctx = RequestContext {
            seq_id,
            session: None,
            source_ip: "127.0.0.1".into(),
            auth_service: auth,
            audit_service: audit,
            whitelist_manager: None,
            device_manager: None,
        };
        (ctx, path)
    }

    #[test]
    fn dispatch_unknown_command_returns_validation_failed() {
        let router = Router::new();
        let (ctx, _path) = test_context(1);
        let response = router.dispatch(&ctx, 0x9999, &[]);

        let (header, payload, _) = codec::try_decode_frame(&response).unwrap().unwrap();
        assert_eq!(header.msg_type, RSP_COMMON);
        assert_eq!(header.seq_id, 1);

        let rsp = common::proto::RspCommon::decode(payload.as_slice()).unwrap();
        assert!(!rsp.success);
        assert_eq!(rsp.result_code, ResultCode::ValidationFailed.as_u16() as i32);
        assert_eq!(rsp.error_message, "unknown command");
    }

    #[test]
    fn dispatch_registered_handler() {
        let mut router = Router::new();
        router.register(
            0x0001,
            Box::new(|ctx: &RequestContext, _payload| {
                codec::encode_frame(0x0002, ctx.seq_id, b"ok").unwrap()
            }),
        );

        let (ctx, _path) = test_context(42);
        let response = router.dispatch(&ctx, 0x0001, &[]);
        let (header, payload, _) = codec::try_decode_frame(&response).unwrap().unwrap();
        assert_eq!(header.msg_type, 0x0002);
        assert_eq!(header.seq_id, 42);
        assert_eq!(payload, b"ok");
    }

    #[test]
    fn register_with_roles_stores_roles() {
        let mut router = Router::new();
        router.register_with_roles(
            0x0010,
            Box::new(|_ctx, _payload| Vec::new()),
            vec![0, 1],
        );
        assert!(router.is_registered(0x0010));
        assert_eq!(router.allowed_roles(0x0010), &[0, 1]);
    }

    #[test]
    fn unregistered_cmd_has_empty_roles() {
        let router = Router::new();
        assert!(!router.is_registered(0x9999));
        assert!(router.allowed_roles(0x9999).is_empty());
    }
}
