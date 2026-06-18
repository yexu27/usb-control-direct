//! 授权校验。
//!
//! 定义授权校验 trait 和 mock 实现。生产环境通过 trait 注入实际校验逻辑，
//! 测试时使用 [`MockLicenseValidator`] 进行基本格式和有效期校验。

use chrono::Utc;

use crate::error::LicenseUpgradeError;

/// 授权信息。
#[derive(Debug, Clone)]
pub struct LicenseInfo {
    /// 授权过期时间（Unix 时间戳，秒）。
    pub expire_time: i64,
}

/// 授权校验能力。
///
/// 实现者负责解析授权文件内容，校验签名、机器码匹配和有效期。
pub trait LicenseValidator: Send + Sync {
    /// 校验授权文件。
    ///
    /// 参数:
    /// - `license_data`: 授权文件原始字节。
    /// - `machine_code`: 当前装置机器码（Base64 编码）。
    ///
    /// 返回:
    /// - 成功时返回 [`LicenseInfo`]；失败时返回 [`LicenseUpgradeError`]。
    fn validate(
        &self,
        license_data: &[u8],
        machine_code: &str,
    ) -> Result<LicenseInfo, LicenseUpgradeError>;
}

/// Mock 授权校验器。
///
/// 授权文件格式：
/// - 第 1 行：机器码（与当前装置机器码匹配）。
/// - 第 2 行：Unix 时间戳（过期时间）。
///
/// 仅用于开发和测试阶段，不得在生产代码中使用。
#[doc(hidden)]
pub struct MockLicenseValidator;

impl LicenseValidator for MockLicenseValidator {
    fn validate(
        &self,
        license_data: &[u8],
        machine_code: &str,
    ) -> Result<LicenseInfo, LicenseUpgradeError> {
        let text = std::str::from_utf8(license_data)
            .map_err(|_| LicenseUpgradeError::LicenseFormatError)?;

        let mut lines = text.lines();

        let file_machine_code = lines
            .next()
            .ok_or(LicenseUpgradeError::LicenseFormatError)?
            .trim();

        let expire_str = lines
            .next()
            .ok_or(LicenseUpgradeError::LicenseFormatError)?
            .trim();

        if file_machine_code != machine_code {
            return Err(LicenseUpgradeError::LicenseVerifyFailed(
                "机器码不匹配".into(),
            ));
        }

        let expire_time: i64 = expire_str
            .parse()
            .map_err(|_| LicenseUpgradeError::LicenseFormatError)?;

        let now = Utc::now().timestamp();
        if expire_time <= now {
            return Err(LicenseUpgradeError::LicenseExpired);
        }

        Ok(LicenseInfo { expire_time })
    }
}
