use common::code::ResultCode;
use protocol_gateway::upgrade_error::map_upgrade_error;
use system_upgrade::{UpgradeError, UpgradePreflightFailure};

#[test]
fn maps_admission_errors_to_existing_result_codes_and_stable_messages() {
    let cases = [
        (
            UpgradeError::VersionNotGreater,
            ResultCode::VersionTooLow,
            "目标版本必须高于当前版本",
        ),
        (
            UpgradeError::DigestMismatch,
            ResultCode::UpgradeChecksumError,
            "升级包摘要不匹配",
        ),
        (
            UpgradeError::SignatureInvalid,
            ResultCode::UpgradeFormatError,
            "升级包签名无效",
        ),
        (
            UpgradeError::SigningDigest("internal digest failure".into()),
            ResultCode::UpgradeFormatError,
            "升级包签名无效",
        ),
        (
            UpgradeError::InvalidSigningDigestLength,
            ResultCode::UpgradeFormatError,
            "升级包签名无效",
        ),
        (
            UpgradeError::Busy,
            ResultCode::DeviceBusy,
            "已有系统升级任务正在处理",
        ),
        (
            UpgradeError::SchemaIncompatible,
            ResultCode::UpgradeFormatError,
            "升级包数据库 Schema 不兼容",
        ),
        (
            UpgradeError::State("invalid state".into()),
            ResultCode::InternalError,
            "系统升级受理失败",
        ),
    ];

    for (error, expected_code, expected_message) in cases {
        let mapped = map_upgrade_error(&error);
        assert_eq!(mapped.result_code, expected_code, "source error: {error}");
        assert_eq!(mapped.message, expected_message, "source error: {error}");
    }
}

#[test]
fn maps_preflight_failures_without_exposing_probe_details() {
    let cases = [
        (
            UpgradePreflightFailure::InsufficientSpace {
                required: 100,
                available: 10,
            },
            ResultCode::DeviceBusy,
            "装置升级空间不足",
        ),
        (
            UpgradePreflightFailure::DpkgBusy,
            ResultCode::DeviceBusy,
            "装置正忙，请稍后重试",
        ),
        (
            UpgradePreflightFailure::ServiceUnavailable,
            ResultCode::DeviceBusy,
            "装置当前不可升级",
        ),
        (
            UpgradePreflightFailure::PlatformIncompatible,
            ResultCode::InternalError,
            "装置升级环境不可用",
        ),
        (
            UpgradePreflightFailure::ProbeFailed("secret stderr".into()),
            ResultCode::InternalError,
            "装置升级环境不可用",
        ),
    ];
    for (failure, expected_code, expected_message) in cases {
        let mapped = map_upgrade_error(&UpgradeError::Preflight(failure));
        assert_eq!(mapped.result_code, expected_code);
        assert_eq!(mapped.message, expected_message);
        assert!(!mapped.message.contains("secret"));
    }
}
