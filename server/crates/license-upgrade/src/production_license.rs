//! 生产环境授权校验器。
//!
//! 授权文件格式（二进制）：
//! - [0..4]: 签名长度（4 字节大端）
//! - [4..4+sig_len]: SM2 签名（DER 编码）
//! - [4+sig_len..]: 授权数据明文（JSON）
//!
//! 授权数据 JSON 字段：
//! - machine_code: 机器码（Base64）
//! - expire_time: 过期时间戳（Unix 秒）
//!
//! 校验流程：
//! 1. SM2 验签（使用内置公钥）
//! 2. 解析 JSON
//! 3. 机器码匹配
//! 4. 有效期检查

use chrono::Utc;
use serde::Deserialize;

use crate::error::LicenseUpgradeError;
use crate::license::{LicenseInfo, LicenseValidator};

/// SM2 签名长度（DER 编码最大长度）。
const SM2_SIGNATURE_MAX_LEN: usize = 72;

/// 签名长度前缀（4 字节大端）。
const SIG_LEN_PREFIX: usize = 4;

/// 授权数据 JSON 结构。
#[derive(Deserialize)]
struct LicensePayload {
    machine_code: String,
    expire_time: i64,
}

/// 生产环境授权校验器。
pub struct ProductionLicenseValidator {
    /// SM2 验签公钥（原始字节，hex 编码后传给 smcrypto）。
    verify_key: Vec<u8>,
}

impl ProductionLicenseValidator {
    /// 创建生产授权校验器。
    ///
    /// 参数:
    /// - `verify_key`: SM2 公钥数据（原始字节）。
    pub fn new(verify_key: Vec<u8>) -> Self {
        Self { verify_key }
    }

    /// 从文件加载 SM2 公钥。
    pub fn from_key_file(path: &std::path::Path) -> Result<Self, LicenseUpgradeError> {
        let key_data = std::fs::read(path).map_err(|e| {
            LicenseUpgradeError::LicenseVerifyFailed(format!("读取公钥文件失败: {}", e))
        })?;
        Ok(Self::new(key_data))
    }
}

impl LicenseValidator for ProductionLicenseValidator {
    fn validate(
        &self,
        license_data: &[u8],
        machine_code: &str,
    ) -> Result<LicenseInfo, LicenseUpgradeError> {
        if license_data.len() < SIG_LEN_PREFIX + 1 {
            return Err(LicenseUpgradeError::LicenseFormatError);
        }

        let sig_len = u32::from_be_bytes(
            license_data[..SIG_LEN_PREFIX]
                .try_into()
                .map_err(|_| LicenseUpgradeError::LicenseFormatError)?,
        ) as usize;

        if sig_len > SM2_SIGNATURE_MAX_LEN || SIG_LEN_PREFIX + sig_len >= license_data.len() {
            return Err(LicenseUpgradeError::LicenseFormatError);
        }

        let signature = &license_data[SIG_LEN_PREFIX..SIG_LEN_PREFIX + sig_len];
        let payload_bytes = &license_data[SIG_LEN_PREFIX + sig_len..];

        let pk_hex = hex::encode(&self.verify_key);
        let verifier = smcrypto::sm2::Verify::new(&pk_hex);
        if !verifier.verify_raw(payload_bytes, signature) {
            return Err(LicenseUpgradeError::LicenseVerifyFailed(
                "SM2 签名验证失败".into(),
            ));
        }

        let payload: LicensePayload = serde_json::from_slice(payload_bytes)
            .map_err(|_| LicenseUpgradeError::LicenseFormatError)?;

        if payload.machine_code != machine_code {
            return Err(LicenseUpgradeError::LicenseVerifyFailed(
                "机器码不匹配".into(),
            ));
        }

        let now = Utc::now().timestamp();
        if payload.expire_time <= now {
            return Err(LicenseUpgradeError::LicenseExpired);
        }

        Ok(LicenseInfo {
            expire_time: payload.expire_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_too_short_data() {
        let validator = ProductionLicenseValidator::new(vec![0u8; 64]);
        let result = validator.validate(&[0, 0, 0], "test");
        assert!(result.is_err());
    }

    #[test]
    fn validate_rejects_invalid_sig_length() {
        let mut data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        data.extend_from_slice(b"payload");
        let validator = ProductionLicenseValidator::new(vec![0u8; 64]);
        let result = validator.validate(&data, "test");
        assert!(matches!(
            result.unwrap_err(),
            LicenseUpgradeError::LicenseFormatError
        ));
    }
}
