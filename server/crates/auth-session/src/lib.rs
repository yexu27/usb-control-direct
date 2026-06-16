//! S08 用户认证与会话管理。
//!
//! 提供密码校验、账号锁定、session token 生命周期和改密码功能。

pub mod error;
pub mod password;
pub mod service;
pub mod session;

pub use service::AuthService;
pub use session::{SessionInfo, SessionManager};
