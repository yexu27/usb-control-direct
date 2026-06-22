//! 策略导入导出端到端测试。

use std::sync::Arc;

use policy_import_export::crypto;
use policy_import_export::error::PolicyError;
use policy_import_export::format::{self, PolicyContent};
use policy_import_export::key_provider::{PolicyKeyProvider, PolicyKeys, SM4_KEY_LEN};
use policy_import_export::PolicyService;
use storage::Storage;
use whitelist::WhitelistManager;

/// 测试用密钥提供者。
///
/// 使用 smcrypto 生成的固定 SM2 密钥对和固定 SM4 密钥。
struct MockKeyProvider {
    keys: PolicyKeys,
}

impl MockKeyProvider {
    fn new() -> Self {
        let (private_key, public_key) = smcrypto::sm2::gen_keypair();
        Self {
            keys: PolicyKeys {
                sm4_key: [
                    0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98,
                    0x76, 0x54, 0x32, 0x10,
                ],
                sm2_private_key: private_key,
                sm2_public_key: public_key,
            },
        }
    }
}

impl PolicyKeyProvider for MockKeyProvider {
    fn load_keys(&self) -> Result<PolicyKeys, PolicyError> {
        Ok(self.keys.clone())
    }
}

/// 创建临时数据库并初始化 schema。
///
/// 返回 `(Arc<Storage>, TempDir)`。调用方必须持有 `TempDir`，
/// 不得提前 drop，否则临时目录会被删除导致数据库访问失败。
fn setup_storage() -> (Arc<Storage>, tempfile::TempDir) {
    let tmp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let db_path = tmp_dir.path().join("test.db");
    let storage = Arc::new(Storage::open(&db_path).expect("打开数据库失败"));
    (storage, tmp_dir)
}

fn setup_whitelist_manager(tmp_dir: &tempfile::TempDir) -> Arc<WhitelistManager> {
    let db_path = tmp_dir.path().join("test.db");
    Arc::new(WhitelistManager::new(Storage::open(&db_path).unwrap()).unwrap())
}

/// 向数据库插入测试白名单数据。
fn insert_test_whitelist(storage: &Storage) {
    use storage::model::UsbWhitelistInsert;

    let items = vec![
        UsbWhitelistInsert {
            serial_number: "TEST-SN-001".into(),
            vid: Some("1234".into()),
            pid: Some("5678".into()),
            device_name: Some("测试 USB 设备 A".into()),
            capacity_bytes: Some(1024 * 1024 * 1024),
            device_type: "storage".into(),
            description: Some("测试白名单设备".into()),
            permission: 1,
            add_method: 0,
        },
        UsbWhitelistInsert {
            serial_number: "TEST-SN-002".into(),
            vid: Some("ABCD".into()),
            pid: Some("EF01".into()),
            device_name: Some("测试 USB 设备 B".into()),
            capacity_bytes: Some(2 * 1024 * 1024 * 1024),
            device_type: "storage".into(),
            description: None,
            permission: 0,
            add_method: 1,
        },
    ];

    for item in &items {
        storage.whitelist_insert(item).expect("插入白名单失败");
    }
}

/// 向数据库插入自定义黑名单数据。
fn insert_test_blacklist(storage: &Storage) {
    storage
        .blacklist_insert(".custom_ext", Some("自定义测试后缀"))
        .expect("插入黑名单失败");
}

#[test]
fn export_import_roundtrip_preserves_data() {
    let (storage, _tmp) = setup_storage();
    let key_provider = Arc::new(MockKeyProvider::new());

    // 插入测试数据
    insert_test_whitelist(&storage);
    insert_test_blacklist(&storage);

    // 开启一个策略
    storage
        .policy_update("exec_control", true)
        .expect("更新策略失败");

    let service = PolicyService::new(
        storage.clone(),
        key_provider.clone(),
        setup_whitelist_manager(&_tmp),
    );

    // 导出
    let bin_data = service.export_policy().expect("导出失败");
    assert!(!bin_data.is_empty());

    // 验证 .bin 文件可解析
    let parsed = format::parse_bin(&bin_data).expect("解析导出文件失败");
    assert_eq!(parsed.version, format::CURRENT_VERSION);

    // 创建新数据库用于导入
    let (import_storage, _import_tmp) = setup_storage();
    let import_service = PolicyService::new(
        import_storage.clone(),
        key_provider,
        setup_whitelist_manager(&_import_tmp),
    );

    // 导入
    import_service.import_policy(&bin_data).expect("导入失败");

    // 验证白名单数据
    let whitelist = import_storage
        .whitelist_query_all()
        .expect("查询白名单失败");
    assert_eq!(whitelist.len(), 2);

    let sn_001 = whitelist
        .iter()
        .find(|w| w.serial_number == "TEST-SN-001")
        .expect("未找到 TEST-SN-001");
    assert_eq!(sn_001.vid.as_deref(), Some("1234"));
    assert_eq!(sn_001.pid.as_deref(), Some("5678"));
    assert_eq!(sn_001.permission, 1);

    let sn_002 = whitelist
        .iter()
        .find(|w| w.serial_number == "TEST-SN-002")
        .expect("未找到 TEST-SN-002");
    assert_eq!(sn_002.vid.as_deref(), Some("ABCD"));
    assert_eq!(sn_002.permission, 0);

    // 验证策略开关
    let policies = import_storage
        .policy_query_all()
        .expect("查询策略失败");
    let exec_control = policies
        .iter()
        .find(|p| p.policy_key == "exec_control")
        .expect("未找到 exec_control");
    assert_eq!(exec_control.enabled, 1);
}

#[test]
fn import_rejects_tampered_ciphertext() {
    let (storage, _tmp) = setup_storage();
    let key_provider = Arc::new(MockKeyProvider::new());

    insert_test_whitelist(&storage);

    let service = PolicyService::new(
        storage.clone(),
        key_provider.clone(),
        setup_whitelist_manager(&_tmp),
    );

    // 导出
    let bin_data = service.export_policy().expect("导出失败");

    // 篡改密文（修改 header 之后的密文区域）
    let mut tampered = bin_data.clone();
    // 密文起始位置 = HEADER_LEN(24) + 4(ciphertext_len)
    let ciphertext_start = format::HEADER_LEN + 4;
    if tampered.len() > ciphertext_start + 1 {
        tampered[ciphertext_start] ^= 0xFF;
        tampered[ciphertext_start + 1] ^= 0xFF;
    }

    // 导入应失败（摘要校验失败或签名校验失败）
    let (import_storage, _import_tmp) = setup_storage();
    let import_service = PolicyService::new(
        import_storage,
        key_provider,
        setup_whitelist_manager(&_import_tmp),
    );
    let result = import_service.import_policy(&tampered);
    assert!(result.is_err(), "篡改后导入应失败");
}

#[test]
fn import_rejects_tampered_signature() {
    let (storage, _tmp) = setup_storage();
    let key_provider = Arc::new(MockKeyProvider::new());

    insert_test_whitelist(&storage);

    let service = PolicyService::new(
        storage.clone(),
        key_provider.clone(),
        setup_whitelist_manager(&_tmp),
    );

    // 导出
    let bin_data = service.export_policy().expect("导出失败");

    // 解析并篡改签名
    let parsed = format::parse_bin(&bin_data).expect("解析失败");
    let mut bad_signature = parsed.signature.clone();
    if !bad_signature.is_empty() {
        bad_signature[0] ^= 0xFF;
    }

    // 用篡改的签名重新组装
    let tampered = format::assemble_bin(
        &parsed.iv,
        &parsed.ciphertext,
        &parsed.digest,
        &bad_signature,
    )
    .expect("重新组装失败");

    let (import_storage, _import_tmp) = setup_storage();
    let import_service = PolicyService::new(
        import_storage,
        key_provider,
        setup_whitelist_manager(&_import_tmp),
    );
    let result = import_service.import_policy(&tampered);
    assert!(result.is_err(), "篡改签名后导入应失败");
}

#[test]
fn import_rejects_bad_magic() {
    let key_provider = Arc::new(MockKeyProvider::new());
    let (import_storage, _import_tmp) = setup_storage();
    let import_service = PolicyService::new(
        import_storage,
        key_provider,
        setup_whitelist_manager(&_import_tmp),
    );

    let bad_data = b"XXXX0000000000000000000000000000000000000000000000000000000000000000";
    let result = import_service.import_policy(bad_data);
    assert!(result.is_err());
}

#[test]
fn crypto_sm4_roundtrip() {
    let key = [0x01u8; SM4_KEY_LEN];
    let iv = crypto::generate_random_iv();
    let plaintext = b"test plaintext data for sm4 encryption roundtrip";

    let ciphertext = crypto::sm4_cbc_encrypt(&key, &iv, plaintext);
    let decrypted = crypto::sm4_cbc_decrypt(&key, &iv, &ciphertext).expect("解密失败");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn crypto_sm2_sign_verify() {
    let (private_key, public_key) = smcrypto::sm2::gen_keypair();
    let data = b"data to sign for roundtrip test";

    let signature = crypto::sm2_sign(&private_key, data).expect("签名失败");
    let valid = crypto::sm2_verify(&public_key, data, &signature).expect("验签失败");
    assert!(valid);

    // 篡改数据后验签应失败
    let bad_data = b"tampered data";
    let invalid = crypto::sm2_verify(&public_key, bad_data, &signature).expect("验签失败");
    assert!(!invalid);
}

#[test]
fn format_serialize_deserialize() {
    let content = PolicyContent {
        whitelist: vec![],
        file_access_policies: vec![],
        file_type_blacklist: vec![],
    };

    let json = format::serialize_policy(&content).expect("序列化失败");
    let restored = format::deserialize_policy(&json).expect("反序列化失败");

    assert!(restored.whitelist.is_empty());
    assert!(restored.file_access_policies.is_empty());
    assert!(restored.file_type_blacklist.is_empty());
}
