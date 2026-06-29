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
    pub nbd_device: PathBuf,
    pub gadget_functions_base: PathBuf,
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
            nbd_device: PathBuf::from("/dev/nbd0"),
            gadget_functions_base: PathBuf::from(
                "/sys/kernel/config/usb_gadget/rockchip/functions",
            ),
        }
    }
}

impl AppConfig {
    pub fn package_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
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
        let mut cfg: Self = toml::from_str(&content).map_err(|source| {
            ConfigError::ParseFailed {
                path: path.to_path_buf(),
                source,
            }
        })?;
        cfg.config_path = path.to_path_buf();
        Ok(cfg)
    }
}
