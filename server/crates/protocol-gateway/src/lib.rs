//! S10 私有协议 API 网关。
//!
//! 基于 tokio + tokio-rustls 实现 TLS 监听、单连接管理、帧流编解码和 router 骨架。

pub mod codec;
pub mod connection;
pub mod context;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod post_send;
pub mod router;
pub mod tls;
pub mod upgrade_error;

pub use error::GatewayError;
