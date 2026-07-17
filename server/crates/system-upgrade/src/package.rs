//! 升级发布容器的受限数据模型。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::SystemVersion;

/// 签名升级清单。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpgradeManifest {
    pub format_version: u32,
    pub product: String,
    pub package_version: SystemVersion,
    pub architecture: String,
    pub protocol_version: u32,
    pub tls_cert_sha256: String,
    pub deb_file: String,
    pub deb_size: u64,
    pub deb_sha256: String,
    pub schema_to: u32,
    pub signing_key_id: String,
}

/// 已安全落盘且完成容器结构解析的升级包。
#[derive(Debug)]
pub struct StagedPackage {
    pub root: PathBuf,
    pub package_path: PathBuf,
    pub manifest_path: PathBuf,
    pub deb_path: PathBuf,
    pub signature_path: PathBuf,
    pub manifest_raw: Vec<u8>,
    pub manifest: UpgradeManifest,
    pub signature: Vec<u8>,
}
