//! 日志审计错误类型。

use thiserror::Error;

/// 日志审计统一错误。
#[derive(Debug, Error)]
pub enum AuditError {
    /// 存储层错误。
    #[error("存储错误: {0}")]
    Storage(#[from] storage::StorageError),

    /// IO 错误（获取磁盘使用率时）。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 空间不足且当前类别无可覆盖旧日志。
    #[error("存储空间不足，当前类别无可覆盖日志")]
    StorageFull,
}
