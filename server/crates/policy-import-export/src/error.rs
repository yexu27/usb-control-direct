//! 策略导入导出统一错误类型。

use common::code::ResultCode;
use thiserror::Error;

/// 策略导入导出错误。
///
/// 覆盖密钥加载、加密解密、签名验证、格式解析和存储写入全链路错误场景。
#[derive(Debug, Error)]
pub enum PolicyError {
    /// 密钥文件不存在。
    #[error("密钥文件不存在: {0}")]
    KeyFileNotFound(String),

    /// 密钥格式错误（长度不符或非法字符）。
    #[error("密钥格式错误: {0}")]
    KeyFormatError(String),

    /// 策略文件格式错误（魔数或结构不匹配）。
    #[error("策略文件格式错误: {0}")]
    FormatError(String),

    /// 策略文件版本不兼容。
    #[error("策略文件版本不兼容: 文件版本={0}, 当前版本={1}")]
    VersionIncompatible(u16, u16),

    /// SM2 签名校验失败。
    #[error("SM2 签名校验失败")]
    SignatureError,

    /// SM3 摘要校验失败。
    #[error("SM3 摘要校验失败")]
    DigestError,

    /// SM4 解密失败。
    #[error("SM4 解密失败: {0}")]
    DecryptError(String),

    /// 序列化或反序列化失败。
    #[error("序列化错误: {0}")]
    SerializeError(String),

    /// 导出失败。
    #[error("策略导出失败: {0}")]
    ExportFailed(String),

    /// 导入失败。
    #[error("策略导入失败: {0}")]
    ImportFailed(String),

    /// 存储层错误。
    #[error("存储错误: {0}")]
    Storage(#[from] storage::StorageError),

    /// IO 错误。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

impl PolicyError {
    /// 将策略错误映射为协议层统一结果码。
    ///
    /// 返回:
    /// - 对应的 [`ResultCode`] 变体。
    pub fn to_result_code(&self) -> ResultCode {
        match self {
            PolicyError::FormatError(_) => ResultCode::PolicyFormatError,
            PolicyError::VersionIncompatible(_, _) => ResultCode::PolicyVersionIncompatible,
            PolicyError::SignatureError => ResultCode::PolicySignatureError,
            PolicyError::DigestError => ResultCode::PolicyDigestError,
            PolicyError::DecryptError(_) => ResultCode::PolicyDecryptError,
            PolicyError::ImportFailed(_) | PolicyError::Storage(_) => {
                ResultCode::PolicyImportFailed
            }
            PolicyError::ExportFailed(_) | PolicyError::SerializeError(_) => {
                ResultCode::PolicyExportFailed
            }
            PolicyError::KeyFileNotFound(_)
            | PolicyError::KeyFormatError(_)
            | PolicyError::Io(_) => ResultCode::InternalError,
        }
    }
}
