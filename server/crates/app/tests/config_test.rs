use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use usb_control_app::config::AppConfig;

const COMPLETE_CONFIG: &str = r#"
listen_addr = "127.0.0.1:19600"
database_path = "/tmp/usb-control/device.db"
tls_cert_path = "/tmp/usb-control/server.crt"
tls_key_path = "/tmp/usb-control/server.key"
policy_key_dir = "/tmp/usb-control/keys"
license_pubkey_path = "/tmp/usb-control/keys/license_verify.pub"
log_dir = "/tmp/usb-control/log"
log_level_conf = "/tmp/usb-control/log.conf"
clamdscan_path = "/usr/bin/clamdscan"
scan_log_dir = "/tmp/usb-control/log/scan"

[upgrade]
root_dir = "/var/lib/usb-control/upgrade"
verify_key_dir = "/etc/usb-control/keys"
max_package_size = 134217728
"#;

#[test]
fn load_from_args_uses_explicit_config_path_and_structured_upgrade_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("usb-control.toml");
    fs::write(&config_path, COMPLETE_CONFIG).unwrap();

    let config = AppConfig::load_from_args([
        "usb-control".to_string(),
        "--config".to_string(),
        config_path.display().to_string(),
    ])
    .unwrap();

    assert_eq!(config.listen_addr, "127.0.0.1:19600");
    assert_eq!(
        config.database_path,
        PathBuf::from("/tmp/usb-control/device.db")
    );
    assert_eq!(
        config.upgrade.root_dir,
        PathBuf::from("/var/lib/usb-control/upgrade")
    );
    assert_eq!(
        config.upgrade.verify_key_dir,
        PathBuf::from("/etc/usb-control/keys")
    );
    assert_eq!(config.upgrade.max_package_size, 128 * 1024 * 1024);
}

#[test]
fn default_config_uses_the_production_upgrade_contract() {
    let config = AppConfig::default();

    assert_eq!(
        config.upgrade.root_dir,
        PathBuf::from("/var/lib/usb-control/upgrade")
    );
    assert_eq!(
        config.upgrade.verify_key_dir,
        PathBuf::from("/etc/usb-control/keys")
    );
    assert_eq!(config.upgrade.max_package_size, 128 * 1024 * 1024);
}

#[test]
fn legacy_install_dir_and_service_name_are_not_accepted() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("legacy.toml");
    fs::write(
        &path,
        COMPLETE_CONFIG.replace(
            "[upgrade]",
            "install_dir = \"/opt/usb-control\"\nservice_name = \"usb-control\"\n\n[upgrade]",
        ),
    )
    .unwrap();

    assert!(AppConfig::load_from_path(&path).is_err());
}

#[test]
fn load_from_args_uses_default_config_path_when_missing() {
    let config = AppConfig::load_from_args(["usb-control".to_string()]).unwrap();
    assert_eq!(
        config.config_path,
        PathBuf::from("/etc/usb-control/usb-control.toml")
    );
}

#[test]
fn package_version_uses_the_shared_release_version() {
    assert_eq!(
        AppConfig::package_version(),
        release_info::display_version()
    );
}
