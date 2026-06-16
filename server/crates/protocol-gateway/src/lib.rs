//! S10 私有协议 API 网关。
//!
//! 基于 tokio + tokio-rustls 实现 TLS 监听、单连接管理、帧流编解码和 router 骨架。

pub mod error;
pub mod codec;
pub mod tls;

pub use error::GatewayError;
