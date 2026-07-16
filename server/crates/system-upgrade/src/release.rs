//! 已安装发布元数据的严格模型。

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::state::{is_lower_hex_64, read_optional_json, PersistedFormat};
use crate::{SystemVersion, UpgradeError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstalledRelease {
    pub format_version: u32,
    pub product: String,
    pub version: SystemVersion,
    pub architecture: String,
    pub supported_schema_min: u32,
    pub supported_schema_max: u32,
    pub tls_cert_sha256: String,
    pub upgrade_signing_key_id: String,
}

pub fn read_installed_release(path: &Path) -> Result<InstalledRelease, UpgradeError> {
    read_optional_json(path)?.ok_or_else(|| UpgradeError::State("release.json 不存在".into()))
}

impl PersistedFormat for InstalledRelease {
    fn validate_persisted(&self) -> Result<(), UpgradeError> {
        if self.format_version != 1
            || self.product != "usb-control"
            || self.architecture != "arm64"
            || self.supported_schema_min == 0
            || self.supported_schema_min > self.supported_schema_max
            || !is_lower_hex_64(&self.tls_cert_sha256)
            || !valid_key_id(&self.upgrade_signing_key_id)
        {
            return Err(UpgradeError::State("已安装发布元数据字段非法".into()));
        }
        Ok(())
    }
}

fn valid_key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
