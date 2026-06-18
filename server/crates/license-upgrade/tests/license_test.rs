//! MockLicenseValidator 集成测试。

use license_upgrade::error::LicenseUpgradeError;
use license_upgrade::license::{LicenseValidator, MockLicenseValidator};

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
