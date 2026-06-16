//! S08 鉴权服务错误类型。

use common::code::ResultCode;
use thiserror::Error;

/// 鉴权服务统一错误。
#[derive(Debug, Error)]
pub enum AuthError {
    /// 用户名或密码错误。
    #[error("用户名或密码错误")]
    UserOrPasswordError,

    /// 账号已锁定。
    #[error("账号已锁定")]
    AccountLocked,

    /// 会话无效或已过期。
    #[error("会话无效或已过期")]
    Unauthenticated,

    /// 密码复杂度不符合要求。
    #[error("密码复杂度不符合要求")]
    PasswordComplexity,

    /// 旧密码错误。
    #[error("旧密码错误")]
    OldPasswordError,

    /// 两次密码不一致。
    #[error("两次密码不一致")]
    PasswordConfirmMismatch,

    /// 用户名已存在。
    #[error("用户名已存在")]
    UsernameExists,

    /// 已删除用户名不可复用。
    #[error("已删除用户名不可复用")]
    UsernameDeletedReuse,

    /// 用户不存在。
    #[error("用户不存在")]
    UserNotFound,

    /// 内置用户不可删除。
    #[error("内置用户不可删除")]
    BuiltinUserNoDelete,

    /// 存储层错误。
    #[error("存储错误: {0}")]
    Storage(#[from] storage::error::StorageError),

    /// bcrypt 错误。
    #[error("bcrypt 错误: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    /// 内部错误。
    #[error("内部错误: {0}")]
    Internal(String),
}

impl AuthError {
    /// 映射到统一结果码。
    pub fn to_result_code(&self) -> ResultCode {
        match self {
            AuthError::UserOrPasswordError => ResultCode::UserOrPasswordError,
            AuthError::AccountLocked => ResultCode::AccountLocked,
            AuthError::Unauthenticated => ResultCode::Unauthenticated,
            AuthError::PasswordComplexity => ResultCode::PasswordComplexityError,
            AuthError::OldPasswordError => ResultCode::OldPasswordError,
            AuthError::PasswordConfirmMismatch => ResultCode::PasswordConfirmMismatch,
            AuthError::UsernameExists => ResultCode::UsernameExists,
            AuthError::UsernameDeletedReuse => ResultCode::UsernameDeletedReuse,
            AuthError::UserNotFound => ResultCode::UserNotFound,
            AuthError::BuiltinUserNoDelete => ResultCode::BuiltinUserNoDelete,
            AuthError::Storage(_) | AuthError::Bcrypt(_) | AuthError::Internal(_) => {
                ResultCode::InternalError
            }
        }
    }
}
