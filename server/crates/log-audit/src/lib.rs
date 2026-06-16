//! S07 日志审计服务。
//!
//! 提供三类日志写入接口（USB 审计/恶意代码/操作日志）和 80% 滚动覆盖逻辑。

pub mod audit;
pub mod error;

pub use audit::{AuditService, LogCategory};
pub use error::AuditError;
