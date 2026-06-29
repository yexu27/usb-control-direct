//! 存储层统一错误类型。

use common::code::ResultCode;
use thiserror::Error;

/// 存储层错误。
///
/// 所有数据库操作、序列化操作和业务校验失败均归一到此枚举；
/// 上层通过 [`StorageError::to_result_code`] 将其映射为协议结果码。
#[derive(Debug, Error)]
pub enum StorageError {
    /// SQLite 驱动层错误。
    #[error("sqlite 错误: {0}")]
    Sqlite(rusqlite::Error),

    /// 输入参数校验失败，携带失败原因描述。
    #[error("参数校验失败: {0}")]
    Validation(String),

    /// 目标记录已存在（唯一约束冲突）。
    #[error("记录已存在")]
    AlreadyExists,

    /// 目标记录不存在。
    #[error("记录不存在: {0}")]
    NotFound(String),

    /// 数据库尚未通过安装期初始化。
    #[error("数据库未初始化: {0}")]
    DatabaseNotInitialized(String),

    /// 数据库 schema 版本高于当前程序可识别范围。
    #[error("数据库 schema 版本不兼容: current={current}, supported={supported}")]
    SchemaVersionUnsupported { current: i32, supported: i32 },

    /// JSON 序列化或反序列化失败。
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(ref e, _) = err {
            if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
                || e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
            {
                return StorageError::AlreadyExists;
            }
        }
        StorageError::Sqlite(err)
    }
}

impl StorageError {
    /// 将存储层错误映射为协议层统一结果码。
    ///
    /// 返回:
    /// - 对应的 [`ResultCode`] 变体，供 RPC 响应填充 `result_code` 字段。
    pub fn to_result_code(&self) -> ResultCode {
        match self {
            StorageError::Sqlite(_) => ResultCode::InternalError,
            StorageError::Validation(_) => ResultCode::ValidationFailed,
            StorageError::AlreadyExists => ResultCode::AlreadyExists,
            StorageError::NotFound(_) => ResultCode::NotFound,
            StorageError::DatabaseNotInitialized(_) => ResultCode::InternalError,
            StorageError::SchemaVersionUnsupported { .. } => ResultCode::InternalError,
            StorageError::Json(_) => ResultCode::InternalError,
        }
    }
}
