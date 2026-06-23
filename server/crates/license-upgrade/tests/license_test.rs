//! MockLicenseValidator + ProductionLicenseValidator 集成测试。

use license_upgrade::error::LicenseUpgradeError;
use license_upgrade::license::{LicenseValidator, MockLicenseValidator};
use license_upgrade::production_license::ProductionLicenseValidator;

#[test]
fn production_validator_smcrypto_roundtrip() {
    use smcrypto::sm2::{gen_keypair, Sign, Verify};
    let (sk, pk_hex) = gen_keypair();

    // 验证 sign_raw + verify_raw 兼容
    let payload = r#"{"machine_code":"mc-001","expire_time":9999999999}"#;
    let signer = Sign::new(&sk);
    let sig_raw = signer.sign_raw(payload.as_bytes());
    let verifier = Verify::new(&pk_hex);
    assert!(verifier.verify_raw(payload.as_bytes(), &sig_raw), "sign_raw+verify_raw 失败");

    // ProductionLicenseValidator 使用 verify_raw → 授权文件需用 sign_raw
    let pk_raw = hex::decode(&pk_hex).unwrap();
    let validator = ProductionLicenseValidator::new(pk_raw);

    let mut license = Vec::new();
    let sig_len = sig_raw.len() as u32;
    license.extend_from_slice(&sig_len.to_be_bytes());
    license.extend_from_slice(&sig_raw);
    license.extend_from_slice(payload.as_bytes());

    let result = validator.validate(&license, "mc-001");
    assert!(result.is_ok(), "SM2 roundtrip: {:?}", result.err());
}

#[test]
fn mock_validator_wrong_machine_code_returns_verify_failed() {
    let validator = MockLicenseValidator;
    let machine_code = "correct-code";
    let license_data = b"wrong-code\n9999999999";

    let result = validator.validate(license_data, machine_code);
    assert!(result.is_err());
    match result.unwrap_err() {
        LicenseUpgradeError::LicenseVerifyFailed(msg) => {
            assert!(msg.contains("机器码不匹配"));
        }
        other => panic!("预期 LicenseVerifyFailed，实际: {other:?}"),
    }
}

#[test]
fn mock_validator_expired_license_returns_expired() {
    let validator = MockLicenseValidator;
    let machine_code = "test-machine-code";
    // 使用一个已过期的时间戳（2020-01-01）
    let license_data = b"test-machine-code\n1577836800";

    let result = validator.validate(license_data, machine_code);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LicenseUpgradeError::LicenseExpired
    ));
}

#[test]
fn mock_validator_valid_license_returns_info() {
    let validator = MockLicenseValidator;
    let machine_code = "test-machine-code";
    // 使用一个远未来的时间戳（2099-01-01）
    let expire_time: i64 = 4_070_908_800;
    let license_data = format!("{machine_code}\n{expire_time}");

    let result = validator.validate(license_data.as_bytes(), machine_code);
    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.expire_time, expire_time);
}

#[test]
fn mock_validator_empty_data_returns_format_error() {
    let validator = MockLicenseValidator;
    let result = validator.validate(b"", "test");
    assert!(matches!(
        result.unwrap_err(),
        LicenseUpgradeError::LicenseFormatError
    ));
}

#[test]
fn mock_validator_single_line_returns_format_error() {
    let validator = MockLicenseValidator;
    let result = validator.validate(b"only-machine-code", "only-machine-code");
    assert!(matches!(
        result.unwrap_err(),
        LicenseUpgradeError::LicenseFormatError
    ));
}

#[test]
fn mock_validator_invalid_timestamp_returns_format_error() {
    let validator = MockLicenseValidator;
    let result = validator.validate(b"code\nnot-a-number", "code");
    assert!(matches!(
        result.unwrap_err(),
        LicenseUpgradeError::LicenseFormatError
    ));
}
