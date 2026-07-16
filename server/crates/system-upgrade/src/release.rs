//! 已安装发布元数据与已健康提交发布的严格模型。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::state::{
    is_lower_hex_64, read_optional_json, sync_dir, validate_upgrade_id, PersistedFormat,
};
use crate::{SystemVersion, UpgradeError, UpgradeStateLock, UpgradeTaskStore};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActiveRelease {
    pub format_version: u32,
    pub version: SystemVersion,
    pub schema_version: u32,
    pub committed_at: i64,
    pub online_upgrade_id: Option<String>,
}

pub fn read_installed_release(path: &Path) -> Result<InstalledRelease, UpgradeError> {
    read_optional_json(path)?.ok_or_else(|| UpgradeError::State("release.json 不存在".into()))
}

#[derive(Debug, thiserror::Error)]
pub enum ActiveCommitError {
    #[error("有效发布 rename 前失败: {0}")]
    BeforeRename(UpgradeError),
    #[error("有效发布 rename 已完成但父目录同步失败: {0}")]
    AfterRename(UpgradeError),
}

#[derive(Debug, Clone)]
pub struct ActiveReleaseStore {
    root: PathBuf,
}

impl ActiveReleaseStore {
    pub fn new(root: PathBuf) -> Result<Self, UpgradeError> {
        UpgradeTaskStore::new(root.clone())?;
        Ok(Self { root })
    }

    pub fn current(&self) -> Result<Option<ActiveRelease>, UpgradeError> {
        read_optional_json(&self.root.join("active-release.json"))
    }

    pub fn commit(
        &self,
        lock: &UpgradeStateLock,
        release: &ActiveRelease,
    ) -> Result<(), ActiveCommitError> {
        lock.require_root(&self.root)
            .map_err(ActiveCommitError::BeforeRename)?;
        release
            .validate_persisted()
            .map_err(ActiveCommitError::BeforeRename)?;
        let path = self.root.join("active-release.json");
        let temporary = self
            .root
            .join(format!(".active-release.{}.tmp", std::process::id()));
        let before_rename = (|| -> Result<(), UpgradeError> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)?;
            file.write_all(
                &serde_json::to_vec(release)
                    .map_err(|error| UpgradeError::State(error.to_string()))?,
            )?;
            file.sync_all()?;
            fs::rename(&temporary, &path)?;
            Ok(())
        })();
        if let Err(error) = before_rename {
            let _ = fs::remove_file(&temporary);
            return Err(ActiveCommitError::BeforeRename(error));
        }
        sync_dir(&self.root)
            .map_err(UpgradeError::Io)
            .map_err(ActiveCommitError::AfterRename)
    }
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

impl PersistedFormat for ActiveRelease {
    fn validate_persisted(&self) -> Result<(), UpgradeError> {
        if self.format_version != 1 || self.schema_version == 0 || self.committed_at <= 0 {
            return Err(UpgradeError::State("有效发布字段非法".into()));
        }
        if let Some(upgrade_id) = &self.online_upgrade_id {
            validate_upgrade_id(upgrade_id)?;
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
