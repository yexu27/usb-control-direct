//! 装置端服务启动配置。

use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const DEFAULT_CONFIG_PATH: &str = "/etc/usb-control/usb-control.toml";

#[derive(Debug)]
pub enum ConfigError {
    MissingConfigValue,
    UnknownArgument(String),
    MissingArgumentValue(String),
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseFailed {
        path: PathBuf,
        source: toml::de::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingConfigValue => write!(f, "配置路径不能为空"),
            ConfigError::UnknownArgument(arg) => write!(f, "未知启动参数: {arg}"),
            ConfigError::MissingArgumentValue(arg) => write!(f, "启动参数 {arg} 缺少值"),
            ConfigError::ReadFailed { path, source } => {
                write!(f, "读取配置文件 {} 失败: {source}", path.display())
            }
            ConfigError::ParseFailed { path, source } => {
                write!(f, "解析配置文件 {} 失败: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(skip)]
    pub config_path: PathBuf,
    pub listen_addr: String,
    pub database_path: PathBuf,
    pub tls_cert_path: PathBuf,
    pub tls_key_path: PathBuf,
    pub install_dir: PathBuf,
    pub service_name: String,
    pub policy_key_dir: PathBuf,
    pub license_pubkey_path: PathBuf,
    pub log_dir: PathBuf,
    pub log_level_conf: PathBuf,
    pub clamdscan_path: String,
    pub scan_log_dir: PathBuf,
    #[serde(default)]
    pub gadget: GadgetConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GadgetConfig {
    pub name: String,
    pub config: String,
    pub udc: Option<String>,
    #[serde(default)]
    pub keep_adb: bool,
    pub storage: GadgetStorageConfig,
    pub keyboard: GadgetHidConfig,
    pub mouse: GadgetHidConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GadgetStorageConfig {
    pub function: String,
    pub lun: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GadgetHidConfig {
    pub function: String,
}

impl Default for GadgetConfig {
    fn default() -> Self {
        Self {
            name: "rockchip".to_string(),
            config: "b.1".to_string(),
            udc: Some("fcc00000.dwc3".to_string()),
            keep_adb: false,
            storage: GadgetStorageConfig::default(),
            keyboard: GadgetHidConfig {
                function: "hid.keyboard".to_string(),
            },
            mouse: GadgetHidConfig {
                function: "hid.mouse".to_string(),
            },
        }
    }
}

impl Default for GadgetStorageConfig {
    fn default() -> Self {
        Self {
            function: "mass_storage.usb0".to_string(),
            lun: 0,
        }
    }
}

impl Default for GadgetHidConfig {
    fn default() -> Self {
        Self {
            function: String::new(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from(DEFAULT_CONFIG_PATH),
            listen_addr: "0.0.0.0:9600".to_string(),
            database_path: PathBuf::from("/var/lib/usb-control/device.db"),
            tls_cert_path: PathBuf::from("/etc/usb-control/tls/server.crt"),
            tls_key_path: PathBuf::from("/etc/usb-control/tls/server.key"),
            install_dir: PathBuf::from("/opt/usb-control"),
            service_name: "usb-control".to_string(),
            policy_key_dir: PathBuf::from("/etc/usb-control/keys"),
            license_pubkey_path: PathBuf::from("/etc/usb-control/keys/license_verify.pub"),
            log_dir: PathBuf::from("/var/log/usb-control"),
            log_level_conf: PathBuf::from("/etc/usb-control/log.conf"),
            clamdscan_path: "/usr/bin/clamdscan".to_string(),
            scan_log_dir: PathBuf::from("/var/log/usb-control/scan"),
            gadget: GadgetConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn package_version() -> &'static str {
        concat!("V", env!("CARGO_PKG_VERSION"))
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut args = args.into_iter().map(Into::into);
        let _program = args.next();
        let mut config_path = PathBuf::from(DEFAULT_CONFIG_PATH);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--config" | "-c" => {
                    let value = args
                        .next()
                        .ok_or_else(|| ConfigError::MissingArgumentValue(arg.clone()))?;
                    if value.trim().is_empty() {
                        return Err(ConfigError::MissingConfigValue);
                    }
                    config_path = PathBuf::from(value);
                }
                "--version" | "-V" => {
                    println!("{}", Self::package_version());
                    std::process::exit(0);
                }
                other => return Err(ConfigError::UnknownArgument(other.to_string())),
            }
        }

        if config_path == PathBuf::from(DEFAULT_CONFIG_PATH) && !config_path.exists() {
            let mut cfg = Self::default();
            cfg.config_path = config_path;
            return Ok(cfg);
        }

        Self::load_from_path(&config_path)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::ReadFailed {
            path: path.to_path_buf(),
            source,
        })?;
        let mut cfg: Self =
            toml::from_str(&content).map_err(|source| ConfigError::ParseFailed {
                path: path.to_path_buf(),
                source,
            })?;
        cfg.config_path = path.to_path_buf();
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gadget_config_with_production_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg_path = dir.path().join("usb-control.toml");
        std::fs::write(
            &cfg_path,
            r#"
listen_addr = "0.0.0.0:9600"
database_path = "/var/lib/usb-control/device.db"
tls_cert_path = "/etc/usb-control/tls/server.crt"
tls_key_path = "/etc/usb-control/tls/server.key"
install_dir = "/opt/usb-control"
service_name = "usb-control"
policy_key_dir = "/etc/usb-control/keys"
license_pubkey_path = "/etc/usb-control/keys/license_verify.pub"
log_dir = "/var/log/usb-control"
log_level_conf = "/etc/usb-control/log.conf"
clamdscan_path = "/usr/bin/clamdscan"
scan_log_dir = "/var/log/usb-control/scan"

[gadget]
name = "rockchip"
config = "b.1"
udc = "fcc00000.dwc3"
keep_adb = false

[gadget.storage]
function = "mass_storage.usb0"
lun = 0

[gadget.keyboard]
function = "hid.keyboard"

[gadget.mouse]
function = "hid.mouse"
"#,
        )
        .unwrap();

        let cfg = AppConfig::load_from_path(&cfg_path).unwrap();
        assert_eq!(cfg.gadget.name, "rockchip");
        assert_eq!(cfg.gadget.config, "b.1");
        assert_eq!(cfg.gadget.udc.as_deref(), Some("fcc00000.dwc3"));
        assert!(!cfg.gadget.keep_adb);
        assert_eq!(cfg.gadget.storage.function, "mass_storage.usb0");
        assert_eq!(cfg.gadget.storage.lun, 0);
        assert_eq!(cfg.gadget.keyboard.function, "hid.keyboard");
        assert_eq!(cfg.gadget.mouse.function, "hid.mouse");
    }

    #[test]
    fn default_config_disables_adb_and_uses_business_functions() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.gadget.name, "rockchip");
        assert_eq!(cfg.gadget.config, "b.1");
        assert_eq!(cfg.gadget.udc.as_deref(), Some("fcc00000.dwc3"));
        assert!(!cfg.gadget.keep_adb);
        assert_eq!(cfg.gadget.storage.function, "mass_storage.usb0");
        assert_eq!(cfg.gadget.storage.lun, 0);
        assert_eq!(cfg.gadget.keyboard.function, "hid.keyboard");
        assert_eq!(cfg.gadget.mouse.function, "hid.mouse");
    }
}
