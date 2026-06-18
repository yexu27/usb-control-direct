//! 授权升级统一错误类型。

use common::code::ResultCode;
use thiserror::Error;

/// 授权与系统维护错误。
///
/// 覆盖机器码生成、授权校验、系统升级和病毒库升级全链路错误场景。
#[derive(Debug, Error)]
pub enum LicenseUpgradeError {
    /// 机器码生成失败（CPU 序列号或 MAC 地址读取异常）。
    #[error("机器码生成失败: {0}")]
    MachineCodeError(String),

    /// 授权文件格式错误。
    #[error("授权文件格式错误")]
    LicenseFormatError,

    /// 授权文件校验失败（签名或机器码不匹配）。
    #[error("授权文件校验失败: {0}")]
    LicenseVerifyFailed(String),

    /// 授权文件已过有效期。
    #[error("授权文件已过有效期")]
    LicenseExpired,

    /// 升级包版本低于当前版本。
    #[error("升级包版本低于当前版本")]
    VersionTooLow,

    /// 升级包文件格式错误。
    #[error("升级包文件格式错误")]
    UpgradeFormatError,

    /// SHA256 校验和不匹配。
    #[error("SHA256 校验和不匹配")]
    UpgradeChecksumError,

    /// 系统升级安装失败（已回滚至原版本）。
    #[error("系统升级安装失败: {0}")]
    UpgradeApplyFailed(String),

    /// 病毒库版本号命中逢 4 跳过规则。
    #[error("病毒库版本号命中逢 4 跳过规则")]
    VersionNumberForbidden,

    /// 病毒库文件完整性校验失败。
    #[error("病毒库文件完整性校验失败")]
    VirusdbIntegrityError,

    /// clamd 重新加载失败（已回滚）。
    #[error("clamd 重新加载失败")]
    ClamdReloadFailed,

    /// 病毒库文件替换失败。
    #[error("病毒库文件替换失败: {0}")]
    VirusdbApplyFailed(String),

    /// 设备描述格式不合法。
    #[error("设备描述格式不合法")]
    DeviceDescFormatError,

    /// 装置有 USB 设备连接时禁止修改设备描述。
    #[error("装置有 USB 设备连接时禁止修改设备描述")]
    DeviceHasUsbConnected,

    /// 存储层错误。
    #[error("存储错误: {0}")]
    Storage(#[from] storage::StorageError),

    /// IO 错误。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 内部错误。
    #[error("内部错误: {0}")]
    Internal(String),
}

impl LicenseUpgradeError {
    /// 将授权升级错误映射为协议层统一结果码。
    ///
    /// 返回:
    /// - 对应的 [`ResultCode`] 变体。
    pub fn to_result_code(&self) -> ResultCode {
        match self {
            LicenseUpgradeError::MachineCodeError(_) => ResultCode::InternalError,
            LicenseUpgradeError::LicenseFormatError => ResultCode::LicenseFormatError,
            LicenseUpgradeError::LicenseVerifyFailed(_) => ResultCode::LicenseVerifyFailed,
            LicenseUpgradeError::LicenseExpired => ResultCode::LicenseExpired,
            LicenseUpgradeError::VersionTooLow => ResultCode::VersionTooLow,
            LicenseUpgradeError::UpgradeFormatError => ResultCode::UpgradeFormatError,
            LicenseUpgradeError::UpgradeChecksumError => ResultCode::UpgradeChecksumError,
            LicenseUpgradeError::UpgradeApplyFailed(_) => ResultCode::UpgradeApplyFailed,
            LicenseUpgradeError::VersionNumberForbidden => ResultCode::VersionNumberForbidden,
            LicenseUpgradeError::VirusdbIntegrityError => ResultCode::VirusdbIntegrityError,
            LicenseUpgradeError::ClamdReloadFailed => ResultCode::ClamdReloadFailed,
            LicenseUpgradeError::VirusdbApplyFailed(_) => ResultCode::VirusdbApplyFailed,
            LicenseUpgradeError::DeviceDescFormatError => ResultCode::DeviceDescFormatError,
            LicenseUpgradeError::DeviceHasUsbConnected => ResultCode::DeviceHasUsbConnected,
            LicenseUpgradeError::Storage(_) | LicenseUpgradeError::Io(_) => {
                ResultCode::InternalError
            }
            LicenseUpgradeError::Internal(_) => ResultCode::InternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_to_result_code_license_format() {
        let err = LicenseUpgradeError::LicenseFormatError;
        assert_eq!(err.to_result_code(), ResultCode::LicenseFormatError);
    }

    #[test]
    fn error_to_result_code_version_too_low() {
        let err = LicenseUpgradeError::VersionTooLow;
        assert_eq!(err.to_result_code(), ResultCode::VersionTooLow);
    }

    #[test]
    fn error_display_includes_detail() {
        let err = LicenseUpgradeError::UpgradeApplyFailed("写入失败".into());
        assert!(format!("{err}").contains("写入失败"));
    }
}
