//! 系统升级共享领域错误。

use thiserror::Error;

/// 装置环境在受理升级前的只读预检失败。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UpgradePreflightFailure {
    #[error("升级空间不足: required={required}, available={available}")]
    InsufficientSpace { required: u64, available: u64 },
    #[error("dpkg 正被其他进程占用")]
    DpkgBusy,
    #[error("dpkg 状态异常")]
    DpkgDamaged,
    #[error("主服务未处于可升级状态")]
    ServiceUnavailable,
    #[error("ClamAV 基础依赖不可用")]
    ClamAvUnavailable,
    #[error("装置运行平台不兼容")]
    PlatformIncompatible,
    #[error("升级环境探测失败: {0}")]
    ProbeFailed(String),
}

/// 系统升级包受理和领域校验错误。
#[derive(Debug, Error)]
pub enum UpgradeError {
    #[error("无效的系统版本: {0}")]
    InvalidVersion(String),
    #[error("目标版本必须高于当前版本")]
    VersionNotGreater,
    #[error("升级包格式错误: {0}")]
    Format(String),
    #[error("升级包摘要不匹配")]
    DigestMismatch,
    #[error("升级包签名无效")]
    SignatureInvalid,
    #[error("升级签名摘要计算失败: {0}")]
    SigningDigest(String),
    #[error("升级签名摘要长度非法")]
    InvalidSigningDigestLength,
    #[error("升级包产品不匹配")]
    ProductMismatch,
    #[error("升级包架构不匹配")]
    ArchitectureMismatch,
    #[error("升级包数据库 Schema 不兼容")]
    SchemaIncompatible,
    #[error("DEB 检查失败: {0}")]
    DebInspection(String),
    #[error("已有系统升级任务正在处理")]
    Busy,
    #[error("系统升级预检失败: {0}")]
    Preflight(UpgradePreflightFailure),
    #[error("系统升级 IO 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("系统升级状态错误: {0}")]
    State(String),
}
