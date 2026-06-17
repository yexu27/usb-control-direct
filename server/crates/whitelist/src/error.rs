//! 白名单错误类型。

use common::code::ResultCode;
use thiserror::Error;

/// 白名单服务统一错误。
///
/// 所有白名单业务校验失败、存储层错误均归一到此枚举；
/// 上层通过 [`WhitelistError::to_result_code`] 将其映射为协议结果码。
#[derive(Debug, Error)]
pub enum WhitelistError {
    /// 序列号为空。
    #[error("序列号不能为空")]
    SerialNumberEmpty,

    /// 序列号已存在，携带重复序列号。
    #[error("序列号已存在: {0}")]
    AlreadyExists(String),

    /// 目标设备不存在，携带序列号。
    #[error("设备不存在: {0}")]
    NotFound(String),

    /// 非存储设备不可添加，携带设备类型描述。
    #[error("非存储设备不可添加: {0}")]
    DeviceNotStorage(String),

    /// 疑似伪装设备（描述符与接口能力不一致），携带设备描述。
    #[error("疑似伪装设备: {0}")]
    DeviceSpoofSuspected(String),

    /// 设备类型不支持，携带设备类型描述。
    #[error("设备类型不支持: {0}")]
    DeviceUnsupported(String),

    /// 存储层错误。
    #[error("存储错误: {0}")]
    Storage(#[from] storage::StorageError),

    /// 参数校验失败，携带失败原因描述。
    #[error("参数校验失败: {0}")]
    Validation(String),

    /// 内部错误，携带原因描述。
    #[error("内部错误: {0}")]
    Internal(String),
}

impl WhitelistError {
    /// 将白名单错误映射为协议层统一结果码。
    ///
    /// 返回:
    /// - 对应的 [`ResultCode`] 变体，供 RPC 响应填充 `result_code` 字段。
    pub fn to_result_code(&self) -> ResultCode {
        match self {
            WhitelistError::SerialNumberEmpty => ResultCode::SerialNumberEmpty,
            WhitelistError::AlreadyExists(_) => ResultCode::AlreadyExists,
            WhitelistError::NotFound(_) => ResultCode::NotFound,
            WhitelistError::DeviceNotStorage(_) => ResultCode::DeviceNotStorage,
            WhitelistError::DeviceSpoofSuspected(_) => ResultCode::DeviceSpoofSuspected,
            WhitelistError::DeviceUnsupported(_) => ResultCode::DeviceUnsupported,
            WhitelistError::Storage(e) => e.to_result_code(),
            WhitelistError::Validation(_) => ResultCode::ValidationFailed,
            WhitelistError::Internal(_) => ResultCode::InternalError,
        }
    }
}
