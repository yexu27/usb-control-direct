//! 策略导入导出服务。
//!
//! 提供 [`PolicyService`] 用于策略的加密导出和解密导入。
//! 导出流程：读取 DB → JSON 序列化 → SM4 加密 → SM3 摘要 → SM2 签名 → 组装 .bin
//! 导入流程：解析 .bin → SM2 验签 → SM3 校验 → SM4 解密 → JSON 反序列化 → 写入 DB

use tracing::{debug, info};

use std::sync::Arc;

use storage::model::UsbWhitelistInsert;
use storage::Storage;
use whitelist::WhitelistManager;

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
    whitelist_manager: Arc<WhitelistManager>,
}

impl PolicyService {
    /// 创建策略导入导出服务实例。
    ///
    /// 参数:
    /// - `storage`: 数据存储句柄。
    /// - `key_provider`: 密钥提供者。
    /// - `whitelist_manager`: 与白名单增删改共享协调锁和缓存的管理器。
    pub fn new(
        storage: Arc<Storage>,
        key_provider: Arc<dyn PolicyKeyProvider>,
        whitelist_manager: Arc<WhitelistManager>,
    ) -> Self {
        Self {
            storage,
            key_provider,
            whitelist_manager,
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
        info!("开始导出策略配置");

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
        let digest = crypto::sm3_hash(&ciphertext)?;

        // SM2 签名（对摘要签名）
        let signature = crypto::sm2_sign(&keys.sm2_private_key, &digest)?;

        // 组装 .bin
        let bin = format::assemble_bin(&iv, &ciphertext, &digest, &signature)?;

        debug!(size = bin.len(), "策略数据导出完成");
        info!("策略配置导出成功");

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
        info!(size = bin_data.len(), "开始导入策略配置");

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
        let computed_digest = crypto::sm3_hash(&parsed.ciphertext)?;
        if computed_digest != parsed.digest {
            return Err(PolicyError::DigestError);
        }

        // SM4 解密
        let plaintext = crypto::sm4_cbc_decrypt(&keys.sm4_key, &parsed.iv, &parsed.ciphertext)?;

        debug!("策略数据解密验证通过");

        // 反序列化策略内容
        let content = format::deserialize_policy(&plaintext)?;

        // 写入数据库
        self.apply_policy(&content)?;

        info!("策略配置导入成功");

        Ok(())
    }

    /// 将策略内容应用到数据库（事务写入）。
    fn apply_policy(&self, content: &PolicyContent) -> Result<(), PolicyError> {
        validate_policy_content(content)?;

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

        let policy_updates: Vec<(String, i32)> = content
            .file_access_policies
            .iter()
            .map(|entry| (entry.policy_key.clone(), entry.enabled))
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

        self.whitelist_manager
            .coordinate_policy_import(|| {
                self.storage.policy_import_transaction(
                    &whitelist_inserts,
                    &policy_updates,
                    &blacklist_inserts,
                )
            })
            .map_err(|e| PolicyError::ImportFailed(format!("策略导入失败: {}", e)))?;

        Ok(())
    }
}

fn validate_policy_content(content: &PolicyContent) -> Result<(), PolicyError> {
    for item in &content.whitelist {
        if !matches!(item.permission, 0 | 1) {
            return invalid_policy_content("白名单权限必须为 0 或 1");
        }
        if !matches!(item.add_method, 0 | 1) {
            return invalid_policy_content("白名单添加方式必须为 0 或 1");
        }
        if item.device_type != "storage" {
            return invalid_policy_content("白名单设备类型必须为 storage");
        }
        if item.capacity_bytes.is_some_and(|capacity| capacity < 0) {
            return invalid_policy_content("白名单容量不得为负数");
        }
    }
    for policy in &content.file_access_policies {
        if !matches!(policy.enabled, 0 | 1) {
            return invalid_policy_content("文件访问策略开关必须为 0 或 1");
        }
    }
    for item in &content.file_type_blacklist {
        if !matches!(item.is_default, 0 | 1) {
            return invalid_policy_content("黑名单默认标记必须为 0 或 1");
        }
    }
    Ok(())
}

fn invalid_policy_content(reason: &str) -> Result<(), PolicyError> {
    Err(PolicyError::ImportFailed(format!(
        "策略内容校验失败: {reason}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{FileAccessPolicyEntry, FileTypeBlacklistEntry, WhitelistEntry};
    use crate::key_provider::{PolicyKeys, SM4_KEY_LEN};
    use storage_test_support::initialize_database;
    use tempfile::NamedTempFile;

    struct UnusedKeyProvider;

    impl PolicyKeyProvider for UnusedKeyProvider {
        fn load_keys(&self) -> Result<PolicyKeys, PolicyError> {
            Ok(PolicyKeys {
                sm4_key: [0; SM4_KEY_LEN],
                sm2_private_key: String::new(),
                sm2_public_key: String::new(),
            })
        }
    }

    struct TestKeyProvider(PolicyKeys);

    impl PolicyKeyProvider for TestKeyProvider {
        fn load_keys(&self) -> Result<PolicyKeys, PolicyError> {
            Ok(self.0.clone())
        }
    }

    fn complete_content(serial_number: &str) -> PolicyContent {
        PolicyContent {
            whitelist: vec![WhitelistEntry {
                serial_number: serial_number.into(),
                vid: None,
                pid: None,
                device_name: None,
                capacity_bytes: None,
                device_type: "storage".into(),
                description: None,
                permission: 0,
                add_method: 0,
            }],
            file_access_policies: vec![
                FileAccessPolicyEntry {
                    policy_key: "exec_control".into(),
                    enabled: 1,
                },
                FileAccessPolicyEntry {
                    policy_key: "auto_read_control".into(),
                    enabled: 1,
                },
                FileAccessPolicyEntry {
                    policy_key: "file_type_blacklist_control".into(),
                    enabled: 1,
                },
            ],
            file_type_blacklist: vec![FileTypeBlacklistEntry {
                extension: ".IMPORTED".into(),
                description: None,
                is_default: 1,
            }],
        }
    }

    #[test]
    fn apply_policy_normalizes_blacklist_and_preserves_default_semantics() {
        let file = NamedTempFile::new().unwrap();
        initialize_database(file.path());
        let storage = Arc::new(Storage::open(file.path()).unwrap());
        storage
            .whitelist_insert(&UsbWhitelistInsert {
                serial_number: "A".into(),
                vid: None,
                pid: None,
                device_name: None,
                capacity_bytes: None,
                device_type: "storage".into(),
                description: None,
                permission: 0,
                add_method: 0,
            })
            .unwrap();
        let whitelist_manager = Arc::new(
            WhitelistManager::new(Arc::new(Storage::open(file.path()).unwrap())).unwrap(),
        );
        let service = PolicyService::new(
            storage.clone(),
            Arc::new(UnusedKeyProvider),
            Arc::clone(&whitelist_manager),
        );

        service.apply_policy(&complete_content("IMPORTED")).unwrap();

        let blacklist = storage.blacklist_query_all().unwrap();
        assert_eq!(blacklist.len(), 1);
        assert_eq!(blacklist[0].extension, ".imported");
        assert_eq!(blacklist[0].is_default, 1);
        assert!(whitelist_manager.is_whitelisted("A").is_none());
        assert!(whitelist_manager.is_whitelisted("IMPORTED").is_some());
    }

    #[test]
    fn export_import_preserves_deleted_default_blacklist_entry() {
        let source_file = NamedTempFile::new().unwrap();
        initialize_database(source_file.path());
        let source_storage = Arc::new(Storage::open(source_file.path()).unwrap());
        source_storage.blacklist_delete(".ps1").unwrap();
        let (private_key, public_key) = smcrypto::sm2::gen_keypair();
        let keys = PolicyKeys {
            sm4_key: [0x5a; SM4_KEY_LEN],
            sm2_private_key: private_key,
            sm2_public_key: public_key,
        };
        let source_service = PolicyService::new(
            source_storage,
            Arc::new(TestKeyProvider(keys.clone())),
            Arc::new(
                WhitelistManager::new(Arc::new(Storage::open(source_file.path()).unwrap())).unwrap(),
            ),
        );
        let policy_data = source_service.export_policy().unwrap();

        let target_file = NamedTempFile::new().unwrap();
        initialize_database(target_file.path());
        let target_storage = Arc::new(Storage::open(target_file.path()).unwrap());
        let target_service = PolicyService::new(
            target_storage.clone(),
            Arc::new(TestKeyProvider(keys)),
            Arc::new(
                WhitelistManager::new(Arc::new(Storage::open(target_file.path()).unwrap())).unwrap(),
            ),
        );
        target_service.import_policy(&policy_data).unwrap();

        assert!(target_storage
            .blacklist_query_by_ext(".ps1")
            .unwrap()
            .is_none());
        assert_eq!(target_storage.blacklist_count().unwrap(), 37);
    }

    #[test]
    fn apply_policy_rejects_unsafe_enums_without_changing_existing_policy() {
        let file = NamedTempFile::new().unwrap();
        initialize_database(file.path());
        let storage = Arc::new(Storage::open(file.path()).unwrap());
        storage
            .whitelist_insert(&UsbWhitelistInsert {
                serial_number: "ORIGINAL".into(),
                vid: None,
                pid: None,
                device_name: None,
                capacity_bytes: Some(1024),
                device_type: "storage".into(),
                description: None,
                permission: 0,
                add_method: 0,
            })
            .unwrap();
        let service = PolicyService::new(
            storage.clone(),
            Arc::new(UnusedKeyProvider),
            Arc::new(
                WhitelistManager::new(Arc::new(Storage::open(file.path()).unwrap())).unwrap(),
            ),
        );
        let original_whitelist: Vec<_> = storage
            .whitelist_query_all()
            .unwrap()
            .into_iter()
            .map(|item| {
                (
                    item.id,
                    item.serial_number,
                    item.vid,
                    item.pid,
                    item.device_name,
                    item.capacity_bytes,
                    item.device_type,
                    item.description,
                    item.permission,
                    item.add_method,
                    item.created_at,
                )
            })
            .collect();
        let original_policies: Vec<_> = storage
            .policy_query_all()
            .unwrap()
            .into_iter()
            .map(|item| (item.policy_key, item.enabled, item.updated_at))
            .collect();
        let original_blacklist: Vec<_> = storage
            .blacklist_query_all()
            .unwrap()
            .into_iter()
            .map(|item| {
                (
                    item.id,
                    item.extension,
                    item.description,
                    item.is_default,
                    item.created_at,
                )
            })
            .collect();

        let mut invalid_permission = complete_content("IMPORTED");
        invalid_permission.whitelist[0].permission = 2;
        let mut invalid_enabled = complete_content("IMPORTED");
        invalid_enabled.file_access_policies[0].enabled = 2;
        let mut invalid_is_default = complete_content("IMPORTED");
        invalid_is_default.file_type_blacklist[0].is_default = 2;
        let mut invalid_add_method = complete_content("IMPORTED");
        invalid_add_method.whitelist[0].add_method = 2;
        let mut invalid_device_type = complete_content("IMPORTED");
        invalid_device_type.whitelist[0].device_type = "keyboard".into();
        let mut invalid_capacity = complete_content("IMPORTED");
        invalid_capacity.whitelist[0].capacity_bytes = Some(-1);

        for (field, content) in [
            ("permission", invalid_permission),
            ("enabled", invalid_enabled),
            ("is_default", invalid_is_default),
            ("add_method", invalid_add_method),
            ("device_type", invalid_device_type),
            ("capacity_bytes", invalid_capacity),
        ] {
            assert!(
                matches!(
                    service.apply_policy(&content),
                    Err(PolicyError::ImportFailed(_))
                ),
                "应拒绝非法字段: {field}"
            );
            assert_eq!(
                storage
                    .whitelist_query_all()
                    .unwrap()
                    .into_iter()
                    .map(|item| {
                        (
                            item.id,
                            item.serial_number,
                            item.vid,
                            item.pid,
                            item.device_name,
                            item.capacity_bytes,
                            item.device_type,
                            item.description,
                            item.permission,
                            item.add_method,
                            item.created_at,
                        )
                    })
                    .collect::<Vec<_>>(),
                original_whitelist,
                "非法字段 {field} 不得修改白名单"
            );
            assert_eq!(
                storage
                    .policy_query_all()
                    .unwrap()
                    .into_iter()
                    .map(|item| (item.policy_key, item.enabled, item.updated_at))
                    .collect::<Vec<_>>(),
                original_policies,
                "非法字段 {field} 不得修改策略开关"
            );
            assert_eq!(
                storage
                    .blacklist_query_all()
                    .unwrap()
                    .into_iter()
                    .map(|item| {
                        (
                            item.id,
                            item.extension,
                            item.description,
                            item.is_default,
                            item.created_at,
                        )
                    })
                    .collect::<Vec<_>>(),
                original_blacklist,
                "非法字段 {field} 不得修改黑名单"
            );
        }
    }
}
