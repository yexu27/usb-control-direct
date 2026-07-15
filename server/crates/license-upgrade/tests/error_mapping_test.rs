use common::code::ResultCode;
use license_upgrade::LicenseUpgradeError;

#[test]
fn retained_license_and_virusdb_errors_keep_protocol_mappings() {
    let cases = [
        (
            LicenseUpgradeError::LicenseFormatError,
            ResultCode::LicenseFormatError,
        ),
        (
            LicenseUpgradeError::VersionTooLow,
            ResultCode::VersionTooLow,
        ),
        (
            LicenseUpgradeError::VersionNumberForbidden,
            ResultCode::VersionNumberForbidden,
        ),
        (
            LicenseUpgradeError::VirusdbIntegrityError,
            ResultCode::VirusdbIntegrityError,
        ),
        (
            LicenseUpgradeError::ClamdReloadFailed,
            ResultCode::ClamdReloadFailed,
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_result_code(), expected);
    }
}

#[test]
fn retained_detailed_errors_preserve_their_message() {
    let error = LicenseUpgradeError::VirusdbApplyFailed("写入失败".into());

    assert_eq!(error.to_result_code(), ResultCode::VirusdbApplyFailed);
    assert!(error.to_string().contains("写入失败"));
}
