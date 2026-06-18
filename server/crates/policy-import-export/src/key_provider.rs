//! 策略密钥提供者。
//!
//! 通过 [`PolicyKeyProvider`] trait 抽象密钥获取方式，
//! 生产环境使用 [`FileKeyProvider`] 从磁盘读取密钥文件，
//! 测试环境通过 trait mock 注入。

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::PolicyError;

/// SM4 密钥长度（字节）。
pub const SM4_KEY_LEN: usize = 16;

/// SM2 私钥 hex 编码长度（64 个十六进制字符 = 32 字节密钥）。
pub const SM2_PRIVATE_KEY_HEX_LEN: usize = 64;

/// SM2 公钥 hex 编码长度（128 个十六进制字符 = 64 字节公钥）。
pub const SM2_PUBLIC_KEY_HEX_LEN: usize = 128;

/// 策略密钥集合。
///
/// 包含 SM4 对称加密密钥和 SM2 非对称签名密钥对。
#[derive(Clone)]
pub struct PolicyKeys {
    /// SM4 对称加密密钥（16 字节）。
    pub sm4_key: [u8; SM4_KEY_LEN],
    /// SM2 私钥（hex 编码字符串，64 字符）。
    pub sm2_private_key: String,
    /// SM2 公钥（hex 编码字符串，128 字符）。
    pub sm2_public_key: String,
}

/// 策略密钥提供者 trait。
///
/// 上层通过本 trait 获取加解密和签名验签所需密钥，
/// 实现方负责密钥的存储和加载细节。
pub trait PolicyKeyProvider: Send + Sync {
    /// 加载策略密钥集合。
    ///
    /// 返回:
    /// - 成功时返回 [`PolicyKeys`]；失败时返回 [`PolicyError`]。
    fn load_keys(&self) -> Result<PolicyKeys, PolicyError>;
}

/// 从磁盘文件加载密钥的提供者。
///
/// 密钥目录下需要三个文件：
/// - `sm4_policy.key`: SM4 密钥原始字节（16 字节）
/// - `sm2_policy.key`: SM2 私钥 hex 编码文本（64 字符）
/// - `sm2_policy.pub`: SM2 公钥 hex 编码文本（128 字符）
pub struct FileKeyProvider {
    key_dir: PathBuf,
}

impl FileKeyProvider {
    /// 创建文件密钥提供者。
    ///
    /// 参数:
    /// - `key_dir`: 密钥文件所在目录路径。
    pub fn new(key_dir: impl AsRef<Path>) -> Self {
        Self {
            key_dir: key_dir.as_ref().to_path_buf(),
        }
    }

    /// 读取并校验 SM4 密钥文件。
    fn load_sm4_key(&self) -> Result<[u8; SM4_KEY_LEN], PolicyError> {
        let path = self.key_dir.join("sm4_policy.key");
        if !path.exists() {
            return Err(PolicyError::KeyFileNotFound(path.display().to_string()));
        }
        let data = fs::read(&path)?;
        if data.len() != SM4_KEY_LEN {
            return Err(PolicyError::KeyFormatError(format!(
                "SM4 密钥长度应为 {} 字节，实际为 {} 字节",
                SM4_KEY_LEN,
                data.len()
            )));
        }
        let mut key = [0u8; SM4_KEY_LEN];
        key.copy_from_slice(&data);
        Ok(key)
    }

    /// 读取并校验 SM2 私钥文件。
    fn load_sm2_private_key(&self) -> Result<String, PolicyError> {
        let path = self.key_dir.join("sm2_policy.key");
        if !path.exists() {
            return Err(PolicyError::KeyFileNotFound(path.display().to_string()));
        }
        let content = fs::read_to_string(&path)?.trim().to_string();
        if content.len() != SM2_PRIVATE_KEY_HEX_LEN {
            return Err(PolicyError::KeyFormatError(format!(
                "SM2 私钥 hex 长度应为 {} 字符，实际为 {} 字符",
                SM2_PRIVATE_KEY_HEX_LEN,
                content.len()
            )));
        }
        validate_hex_string(&content, "SM2 私钥")?;
        Ok(content)
    }

    /// 读取并校验 SM2 公钥文件。
    fn load_sm2_public_key(&self) -> Result<String, PolicyError> {
        let path = self.key_dir.join("sm2_policy.pub");
        if !path.exists() {
            return Err(PolicyError::KeyFileNotFound(path.display().to_string()));
        }
        let content = fs::read_to_string(&path)?.trim().to_string();
        if content.len() != SM2_PUBLIC_KEY_HEX_LEN {
            return Err(PolicyError::KeyFormatError(format!(
                "SM2 公钥 hex 长度应为 {} 字符，实际为 {} 字符",
                SM2_PUBLIC_KEY_HEX_LEN,
                content.len()
            )));
        }
        validate_hex_string(&content, "SM2 公钥")?;
        Ok(content)
    }
}

impl PolicyKeyProvider for FileKeyProvider {
    fn load_keys(&self) -> Result<PolicyKeys, PolicyError> {
        let sm4_key = self.load_sm4_key()?;
        let sm2_private_key = self.load_sm2_private_key()?;
        let sm2_public_key = self.load_sm2_public_key()?;
        Ok(PolicyKeys {
            sm4_key,
            sm2_private_key,
            sm2_public_key,
        })
    }
}

/// 校验字符串是否为合法的十六进制编码。
fn validate_hex_string(s: &str, label: &str) -> Result<(), PolicyError> {
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PolicyError::KeyFormatError(format!(
            "{} 包含非法十六进制字符",
            label
        )));
    }
    Ok(())
}
