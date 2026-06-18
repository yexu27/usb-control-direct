//! 策略导入导出服务。
//!
//! 提供 [`PolicyService`] 用于策略的加密导出和解密导入。
//! 导出流程：读取 DB → JSON 序列化 → SM4 加密 → SM3 摘要 → SM2 签名 → 组装 .bin
//! 导入流程：解析 .bin → SM2 验签 → SM3 校验 → SM4 解密 → JSON 反序列化 → 写入 DB

use std::sync::Arc;

use storage::model::UsbWhitelistInsert;
use storage::Storage;

use crate::crypto;
use crate::error::PolicyError;
use crate::format::{
    self, FileAccessPolicyEntry, FileTypeBlacklistEntry, PolicyContent, WhitelistEntry,
};
use crate::key_provider::PolicyKeyProvider;

/// 策略导入导出服务。
///
/// 通过注入的 [`PolicyKeyProvider`] 获取密钥，
/// 通过注入的 [`Storage`] 读写数据库。
pub struct PolicyService {
    storage: Arc<Storage>,
    key_provider: Arc<dyn PolicyKeyProvider>,
}

impl PolicyService {
    /// 创建策略导入导出服务实例。
    ///
    /// 参数:
    /// - `storage`: 数据存储句柄。
    /// - `key_provider`: 密钥提供者。
    pub fn new(storage: Arc<Storage>, key_provider: Arc<dyn PolicyKeyProvider>) -> Self {
        Self {
            storage,
            key_provider,
        }
    }

    /// 导出全量策略为加密签名的 .bin 文件。
    ///
    /// 流程：
    /// 1. 从数据库读取白名单、文件访问策略、文件类型黑名单
    /// 2. JSON 序列化策略内容
    /// 3. 生成随机 IV，SM4-CBC 加密
    /// 4. 计算密文的 SM3 摘要
    /// 5. 使用 SM2 私钥对摘要签名
    /// 6. 组装为 .bin 文件格式
    ///
    /// 返回:
    /// - 成功时返回 .bin 文件字节序列；失败时返回 [`PolicyError`]。
    pub fn export_policy(&self) -> Result<Vec<u8>, PolicyError> {
        // 加载密钥
        let keys = self.key_provider.load_keys()?;

        // 从数据库读取策略数据
        let whitelist_items = self
            .storage
            .whitelist_query_all()
            .map_err(|e| PolicyError::ExportFailed(format!("读取白名单失败: {}", e)))?;
        let policy_items = self
            .storage
            .policy_query_all()
            .map_err(|e| PolicyError::ExportFailed(format!("读取文件访问策略失败: {}", e)))?;
        let blacklist_items = self
            .storage
            .blacklist_query_all()
            .map_err(|e| PolicyError::ExportFailed(format!("读取文件类型黑名单失败: {}", e)))?;

        // 构建策略内容
        let content = PolicyContent {
            whitelist: whitelist_items.iter().map(WhitelistEntry::from).collect(),
            file_access_policies: policy_items
                .iter()
                .map(FileAccessPolicyEntry::from)
                .collect(),
            file_type_blacklist: blacklist_items
                .iter()
                .map(FileTypeBlacklistEntry::from)
                .collect(),
        };

        // 序列化
        let plaintext = format::serialize_policy(&content)?;

        // SM4 加密
        let iv = crypto::generate_random_iv();
        let ciphertext = crypto::sm4_cbc_encrypt(&keys.sm4_key, &iv, &plaintext);

        // SM3 摘要（对密文计算）
        let digest = crypto::sm3_hash(&ciphertext);

        // SM2 签名（对摘要签名）
        let signature = crypto::sm2_sign(&keys.sm2_private_key, &digest)?;

        // 组装 .bin
        let bin = format::assemble_bin(&iv, &ciphertext, &digest, &signature);

        Ok(bin)
    }

    /// 导入加密签名的 .bin 文件，写入数据库。
    ///
    /// 流程：
    /// 1. 解析 .bin 文件结构
    /// 2. SM2 公钥验签
    /// 3. SM3 摘要校验
    /// 4. SM4-CBC 解密
    /// 5. JSON 反序列化策略内容
    /// 6. 事务写入数据库
    ///
    /// 参数:
    /// - `bin_data`: .bin 文件原始字节。
    ///
    /// 返回:
    /// - 成功时返回 `()`；失败时返回 [`PolicyError`]。
    pub fn import_policy(&self, bin_data: &[u8]) -> Result<(), PolicyError> {
        // 加载密钥
        let keys = self.key_provider.load_keys()?;

        // 解析 .bin 文件
        let parsed = format::parse_bin(bin_data)?;

        // SM2 验签（对摘要验签）
        let sig_valid = crypto::sm2_verify(
            &keys.sm2_public_key,
            &parsed.digest,
            &parsed.signature,
        )?;
        if !sig_valid {
            return Err(PolicyError::SignatureError);
        }

        // SM3 摘要校验（对密文计算摘要并比较）
        let computed_digest = crypto::sm3_hash(&parsed.ciphertext);
        if computed_digest != parsed.digest {
            return Err(PolicyError::DigestError);
        }

        // SM4 解密
        let plaintext = crypto::sm4_cbc_decrypt(&keys.sm4_key, &parsed.iv, &parsed.ciphertext)?;

        // 反序列化策略内容
        let content = format::deserialize_policy(&plaintext)?;

        // 写入数据库
        self.apply_policy(&content)?;

        Ok(())
    }

    /// 将策略内容应用到数据库（事务写入）。
    fn apply_policy(&self, content: &PolicyContent) -> Result<(), PolicyError> {
        // 转换为 Storage 自身的类型
        let whitelist_inserts: Vec<UsbWhitelistInsert> = content
            .whitelist
            .iter()
            .map(|entry| UsbWhitelistInsert {
                serial_number: entry.serial_number.clone(),
                vid: entry.vid.clone(),
                pid: entry.pid.clone(),
                device_name: entry.device_name.clone(),
                capacity_bytes: entry.capacity_bytes,
                device_type: entry.device_type.clone(),
                description: entry.description.clone(),
                permission: entry.permission,
                add_method: entry.add_method,
            })
            .collect();

        let policy_updates: Vec<(String, bool)> = content
            .file_access_policies
            .iter()
            .map(|entry| (entry.policy_key.clone(), entry.enabled != 0))
            .collect();

        let blacklist_inserts: Vec<(String, Option<String>, i32)> = content
            .file_type_blacklist
            .iter()
            .map(|entry| {
                (
                    entry.extension.clone(),
                    entry.description.clone(),
                    entry.is_default,
                )
            })
            .collect();

        self.storage
            .policy_import_transaction(&whitelist_inserts, &policy_updates, &blacklist_inserts)
            .map_err(|e| PolicyError::ImportFailed(format!("策略写入数据库失败: {}", e)))?;

        Ok(())
    }
}
