//! 协议网关错误类型。

use common::code::ResultCode;
use thiserror::Error;

/// 协议网关统一错误。
#[derive(Debug, Error)]
pub enum GatewayError {
    /// IO 错误。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// TLS 配置错误。
    #[error("TLS 配置错误: {0}")]
    TlsConfig(String),

    /// 协议帧错误。
    #[error("协议帧错误: {0}")]
    Frame(#[from] common::error::CommonError),

    /// CRC32 校验失败。
    #[error("CRC32 校验失败")]
    CrcMismatch,

    /// 连续 CRC 失败超限。
    #[error("连续 CRC 失败超限，断开连接")]
    CrcExceeded,

    /// 连接已被占用（单连接限制）。
    #[error("装置正在执行其他操作")]
    DeviceBusy,

    /// 心跳超时。
    #[error("心跳超时")]
    HeartbeatTimeout,

    /// 需要更多数据（帧不完整）。
    #[error("需要更多数据")]
    NeedMoreData,

    /// 未知命令。
    #[error("未知命令: 0x{0:04X}")]
    UnknownCommand(u32),
}

impl GatewayError {
    /// 映射到统一结果码。
    pub fn to_result_code(&self) -> ResultCode {
        match self {
            GatewayError::DeviceBusy => ResultCode::DeviceBusy,
            GatewayError::UnknownCommand(_) => ResultCode::ValidationFailed,
            _ => ResultCode::InternalError,
        }
    }
}
