//! 策略包 .bin 文件格式定义与解析。
//!
//! 文件格式：
//! ```text
//! +------------------+--------+
//! | 字段             | 字节数 |
//! +------------------+--------+
//! | magic (USCP)     | 4      |
//! | version          | 2      |
//! | alg_id           | 2      |
//! | iv               | 16     |
//! | ciphertext_len   | 4 (u32)|
//! | ciphertext       | N      |
//! | digest (SM3)     | 32     |
//! | signature_len    | 4 (u32)|
//! | signature        | M      |
//! +------------------+--------+
//! ```

use serde::{Deserialize, Serialize};
use storage::model::{FileAccessPolicy, FileTypeBlacklist, UsbWhitelist};

use crate::error::PolicyError;

/// 文件魔数。
pub const MAGIC: &[u8; 4] = b"USCP";

/// 当前协议版本。
pub const CURRENT_VERSION: u16 = 1;

/// 算法标识（SM4+SM3+SM2 组合）。
pub const ALG_ID: u16 = 0x0001;

/// SM3 摘要长度（字节）。
pub const SM3_DIGEST_LEN: usize = 32;

/// 文件头固定长度：magic(4) + version(2) + alg_id(2) + iv(16) = 24。
pub const HEADER_LEN: usize = 24;

/// 策略内容（序列化/反序列化载体）。
///
/// 包含白名单、文件访问策略和文件类型黑名单三部分数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContent {
    /// 白名单条目列表。
    pub whitelist: Vec<WhitelistEntry>,
    /// 文件访问控制策略列表。
    pub file_access_policies: Vec<FileAccessPolicyEntry>,
    /// 文件类型黑名单列表。
    pub file_type_blacklist: Vec<FileTypeBlacklistEntry>,
}

/// 白名单导出条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub serial_number: String,
    pub vid: Option<String>,
    pub pid: Option<String>,
    pub device_name: Option<String>,
    pub capacity_bytes: Option<i64>,
    pub device_type: String,
    pub description: Option<String>,
    pub permission: i32,
    pub add_method: i32,
}

/// 文件访问控制策略导出条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessPolicyEntry {
    pub policy_key: String,
    pub enabled: i32,
}

/// 文件类型黑名单导出条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeBlacklistEntry {
    pub extension: String,
    pub description: Option<String>,
    pub is_default: i32,
}

impl From<&UsbWhitelist> for WhitelistEntry {
    fn from(item: &UsbWhitelist) -> Self {
        Self {
            serial_number: item.serial_number.clone(),
            vid: item.vid.clone(),
            pid: item.pid.clone(),
            device_name: item.device_name.clone(),
            capacity_bytes: item.capacity_bytes,
            device_type: item.device_type.clone(),
            description: item.description.clone(),
            permission: item.permission,
            add_method: item.add_method,
        }
    }
}

impl From<&FileAccessPolicy> for FileAccessPolicyEntry {
    fn from(item: &FileAccessPolicy) -> Self {
        Self {
            policy_key: item.policy_key.clone(),
            enabled: item.enabled,
        }
    }
}

impl From<&FileTypeBlacklist> for FileTypeBlacklistEntry {
    fn from(item: &FileTypeBlacklist) -> Self {
        Self {
            extension: item.extension.clone(),
            description: item.description.clone(),
            is_default: item.is_default,
        }
    }
}

/// 将策略内容序列化为 JSON 字节。
///
/// 参数:
/// - `content`: 策略内容。
///
/// 返回:
/// - 成功时返回 JSON 字节序列；失败时返回 [`PolicyError::SerializeError`]。
pub fn serialize_policy(content: &PolicyContent) -> Result<Vec<u8>, PolicyError> {
    serde_json::to_vec(content).map_err(|e| PolicyError::SerializeError(e.to_string()))
}

/// 将 JSON 字节反序列化为策略内容。
///
/// 参数:
/// - `data`: JSON 字节序列。
///
/// 返回:
/// - 成功时返回 [`PolicyContent`]；失败时返回 [`PolicyError::SerializeError`]。
pub fn deserialize_policy(data: &[u8]) -> Result<PolicyContent, PolicyError> {
    serde_json::from_slice(data).map_err(|e| PolicyError::SerializeError(e.to_string()))
}

/// 组装 .bin 文件。
///
/// 参数:
/// - `iv`: 初始化向量（16 字节）。
/// - `ciphertext`: SM4 加密后的密文。
/// - `digest`: SM3 摘要（32 字节）。
/// - `signature`: SM2 签名。
///
/// 返回:
/// - 成功时返回组装后的 .bin 文件字节序列；失败时返回 [`PolicyError::FormatError`]。
pub fn assemble_bin(
    iv: &[u8; 16],
    ciphertext: &[u8],
    digest: &[u8],
    signature: &[u8],
) -> Result<Vec<u8>, PolicyError> {
    let ciphertext_len = u32::try_from(ciphertext.len()).map_err(|_| {
        PolicyError::FormatError(format!(
            "密文长度超出 u32 范围: {} 字节",
            ciphertext.len()
        ))
    })?;
    let signature_len = u32::try_from(signature.len()).map_err(|_| {
        PolicyError::FormatError(format!(
            "签名长度超出 u32 范围: {} 字节",
            signature.len()
        ))
    })?;

    let total_len = HEADER_LEN + 4 + ciphertext.len() + SM3_DIGEST_LEN + 4 + signature.len();
    let mut buf = Vec::with_capacity(total_len);

    // header
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&CURRENT_VERSION.to_le_bytes());
    buf.extend_from_slice(&ALG_ID.to_le_bytes());
    buf.extend_from_slice(iv);

    // ciphertext
    buf.extend_from_slice(&ciphertext_len.to_le_bytes());
    buf.extend_from_slice(ciphertext);

    // digest
    buf.extend_from_slice(digest);

    // signature
    buf.extend_from_slice(&signature_len.to_le_bytes());
    buf.extend_from_slice(signature);

    Ok(buf)
}

/// 解析后的 .bin 文件结构。
pub struct ParsedBin {
    /// 文件版本。
    pub version: u16,
    /// 算法标识。
    pub alg_id: u16,
    /// 初始化向量。
    pub iv: [u8; 16],
    /// 密文数据。
    pub ciphertext: Vec<u8>,
    /// SM3 摘要。
    pub digest: Vec<u8>,
    /// SM2 签名。
    pub signature: Vec<u8>,
}

/// 解析 .bin 文件。
///
/// 参数:
/// - `data`: .bin 文件原始字节。
///
/// 返回:
/// - 成功时返回 [`ParsedBin`]；失败时返回格式或版本错误。
pub fn parse_bin(data: &[u8]) -> Result<ParsedBin, PolicyError> {
    // 最小长度：header(24) + ciphertext_len(4) + digest(32) + signature_len(4) = 64
    if data.len() < HEADER_LEN + 4 + SM3_DIGEST_LEN + 4 {
        return Err(PolicyError::FormatError("文件长度不足".into()));
    }

    // 校验魔数
    if &data[0..4] != MAGIC {
        return Err(PolicyError::FormatError(format!(
            "魔数不匹配: 期望 {:?}, 实际 {:?}",
            MAGIC,
            &data[0..4]
        )));
    }

    // 解析版本
    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != CURRENT_VERSION {
        return Err(PolicyError::VersionIncompatible(version, CURRENT_VERSION));
    }

    // 解析算法标识
    let alg_id = u16::from_le_bytes([data[6], data[7]]);
    if alg_id != ALG_ID {
        return Err(PolicyError::FormatError(format!(
            "算法标识不匹配: 期望 0x{:04X}, 实际 0x{:04X}",
            ALG_ID, alg_id
        )));
    }

    // 解析 IV
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&data[8..24]);

    // 解析密文
    let mut offset = HEADER_LEN;
    if offset + 4 > data.len() {
        return Err(PolicyError::FormatError("缺少密文长度字段".into()));
    }
    let ciphertext_len =
        u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
            as usize;
    offset += 4;

    if offset + ciphertext_len > data.len() {
        return Err(PolicyError::FormatError(format!(
            "密文长度越界: 声明 {} 字节, 剩余 {} 字节",
            ciphertext_len,
            data.len() - offset
        )));
    }
    let ciphertext = data[offset..offset + ciphertext_len].to_vec();
    offset += ciphertext_len;

    // 解析摘要
    if offset + SM3_DIGEST_LEN > data.len() {
        return Err(PolicyError::FormatError("缺少 SM3 摘要字段".into()));
    }
    let digest = data[offset..offset + SM3_DIGEST_LEN].to_vec();
    offset += SM3_DIGEST_LEN;

    // 解析签名
    if offset + 4 > data.len() {
        return Err(PolicyError::FormatError("缺少签名长度字段".into()));
    }
    let signature_len =
        u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
            as usize;
    offset += 4;

    if offset + signature_len > data.len() {
        return Err(PolicyError::FormatError(format!(
            "签名长度越界: 声明 {} 字节, 剩余 {} 字节",
            signature_len,
            data.len() - offset
        )));
    }
    let signature = data[offset..offset + signature_len].to_vec();

    Ok(ParsedBin {
        version,
        alg_id,
        iv,
        ciphertext,
        digest,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_content() -> PolicyContent {
        PolicyContent {
            whitelist: vec![WhitelistEntry {
                serial_number: "SN001".into(),
                vid: Some("1234".into()),
                pid: Some("5678".into()),
                device_name: Some("Test USB".into()),
                capacity_bytes: Some(1024 * 1024),
                device_type: "storage".into(),
                description: Some("测试设备".into()),
                permission: 1,
                add_method: 0,
            }],
            file_access_policies: vec![FileAccessPolicyEntry {
                policy_key: "exec_control".into(),
                enabled: 1,
            }],
            file_type_blacklist: vec![FileTypeBlacklistEntry {
                extension: ".exe".into(),
                description: Some("可执行文件".into()),
                is_default: 1,
            }],
        }
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let content = make_test_content();
        let json = serialize_policy(&content).expect("序列化失败");
        let restored = deserialize_policy(&json).expect("反序列化失败");

        assert_eq!(restored.whitelist.len(), 1);
        assert_eq!(restored.whitelist[0].serial_number, "SN001");
        assert_eq!(restored.file_access_policies.len(), 1);
        assert_eq!(restored.file_type_blacklist.len(), 1);
    }

    #[test]
    fn assemble_parse_roundtrip() {
        let iv = [1u8; 16];
        let ciphertext = b"encrypted data here".to_vec();
        let digest = [2u8; 32].to_vec();
        let signature = b"signature bytes".to_vec();

        let bin = assemble_bin(&iv, &ciphertext, &digest, &signature).expect("组装失败");
        let parsed = parse_bin(&bin).expect("解析失败");

        assert_eq!(parsed.version, CURRENT_VERSION);
        assert_eq!(parsed.alg_id, ALG_ID);
        assert_eq!(parsed.iv, iv);
        assert_eq!(parsed.ciphertext, ciphertext);
        assert_eq!(parsed.digest, digest);
        assert_eq!(parsed.signature, signature);
    }

    #[test]
    fn parse_bin_rejects_bad_magic() {
        let mut bin = vec![0u8; 64 + 32];
        bin[0..4].copy_from_slice(b"XXXX");
        let result = parse_bin(&bin);
        assert!(result.is_err());
    }

    #[test]
    fn parse_bin_rejects_short_data() {
        let result = parse_bin(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_bin_rejects_wrong_version() {
        let mut bin = vec![0u8; 64 + 32];
        bin[0..4].copy_from_slice(MAGIC);
        bin[4..6].copy_from_slice(&99u16.to_le_bytes());
        let result = parse_bin(&bin);
        assert!(matches!(result, Err(PolicyError::VersionIncompatible(99, _))));
    }
}
