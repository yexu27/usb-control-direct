//! 消息路由器骨架。
//!
//! 提供 `Router::register(cmd, handler)` 注册接口和 `dispatch` 分发。
//! 未注册的 cmd 返回 VALIDATION_FAILED + "unknown command"。

use std::collections::HashMap;

use common::code::ResultCode;
use prost::Message;

use crate::codec;

/// 消息处理函数类型。
///
/// 参数:
///   - `seq_id`: 请求序列号。
///   - `payload`: 请求 payload（protobuf 序列化字节）。
///
/// 返回编码好的响应帧字节。
pub type HandlerFn = Box<dyn Fn(u32, &[u8]) -> Vec<u8> + Send + Sync>;

/// 通用响应消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// 消息路由器。
pub struct Router {
    handlers: HashMap<u32, HandlerFn>,
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

    /// 注册消息处理函数。
    pub fn register(&mut self, msg_type: u32, handler: HandlerFn) {
        self.handlers.insert(msg_type, handler);
    }

    /// 分发消息。
    ///
    /// 返回编码好的响应帧字节（包含帧头）。
    pub fn dispatch(&self, msg_type: u32, seq_id: u32, payload: &[u8]) -> Vec<u8> {
        if let Some(handler) = self.handlers.get(&msg_type) {
            handler(seq_id, payload)
        } else {
            self.unknown_command_response(seq_id)
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

    #[test]
    fn dispatch_unknown_command_returns_validation_failed() {
        let router = Router::new();
        let response = router.dispatch(0x9999, 1, &[]);

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
        router.register(0x0001, Box::new(|seq_id, _payload| {
            codec::encode_frame(0x0002, seq_id, b"ok").unwrap()
        }));

        let response = router.dispatch(0x0001, 42, &[]);
        let (header, payload, _) = codec::try_decode_frame(&response).unwrap().unwrap();
        assert_eq!(header.msg_type, 0x0002);
        assert_eq!(header.seq_id, 42);
        assert_eq!(payload, b"ok");
    }
}
