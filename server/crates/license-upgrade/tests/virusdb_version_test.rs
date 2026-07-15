use license_upgrade::{LicenseUpgradeError, VirusdbUpgradeManager};

#[test]
fn virusdb_validation_uses_its_own_version_rule() {
    let manager = VirusdbUpgradeManager::new("/tmp/not-used-by-validation");

    assert!(manager.validate_upgrade("3.10.0", "3.9.9").is_ok());
    assert!(matches!(
        manager.validate_upgrade("3.9.9", "3.9.9"),
        Err(LicenseUpgradeError::VersionTooLow)
    ));
    assert!(matches!(
        manager.validate_upgrade("3.14.0", "3.9.9"),
        Err(LicenseUpgradeError::VersionNumberForbidden)
    ));
}
