use std::fs;

use tempfile::tempdir;

#[path = "../src/config.rs"]
mod config;

use config::AppConfig;

#[test]
fn load_from_args_uses_explicit_config_path() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("usb-control.toml");
    fs::write(
        &config_path,
        r#"
listen_addr = "127.0.0.1:19600"
database_path = "/tmp/usb-control/device.db"
tls_cert_path = "/tmp/usb-control/server.crt"
tls_key_path = "/tmp/usb-control/server.key"
install_dir = "/opt/usb-control"
service_name = "usb-control"
policy_key_dir = "/tmp/usb-control/keys"
license_pubkey_path = "/tmp/usb-control/keys/license_verify.pub"
log_dir = "/tmp/usb-control/log"
log_level_conf = "/tmp/usb-control/log.conf"
clamdscan_path = "/usr/bin/clamdscan"
scan_log_dir = "/tmp/usb-control/log/scan"
"#,
    )
    .unwrap();

    let cfg = AppConfig::load_from_args([
        "usb-control".to_string(),
        "--config".to_string(),
        config_path.display().to_string(),
    ])
    .unwrap();

    assert_eq!(cfg.listen_addr, "127.0.0.1:19600");
    assert_eq!(cfg.database_path.display().to_string(), "/tmp/usb-control/device.db");
    assert_eq!(cfg.tls_cert_path.display().to_string(), "/tmp/usb-control/server.crt");
    assert_eq!(cfg.tls_key_path.display().to_string(), "/tmp/usb-control/server.key");
    assert_eq!(cfg.install_dir.display().to_string(), "/opt/usb-control");
    assert_eq!(cfg.service_name, "usb-control");
    assert_eq!(cfg.policy_key_dir.display().to_string(), "/tmp/usb-control/keys");
    assert_eq!(
        cfg.license_pubkey_path.display().to_string(),
        "/tmp/usb-control/keys/license_verify.pub"
    );
    assert_eq!(cfg.log_dir.display().to_string(), "/tmp/usb-control/log");
    assert_eq!(cfg.log_level_conf.display().to_string(), "/tmp/usb-control/log.conf");
    assert_eq!(cfg.clamdscan_path, "/usr/bin/clamdscan");
    assert_eq!(cfg.scan_log_dir.display().to_string(), "/tmp/usb-control/log/scan");
}

#[test]
fn load_from_args_uses_default_config_path_when_missing() {
    let cfg = AppConfig::load_from_args(["usb-control".to_string()]).unwrap();
    assert_eq!(cfg.config_path.display().to_string(), "/etc/usb-control/usb-control.toml");
}

#[test]
fn package_version_matches_cargo_package_version() {
    assert_eq!(AppConfig::package_version(), concat!("V", env!("CARGO_PKG_VERSION")));
}
