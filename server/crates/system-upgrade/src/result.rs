//! updater 与主服务共享的有效版本和终态结果格式。

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::state::{
    atomic_write_json, is_lower_hex_64, read_optional_json, validate_upgrade_id, PersistedFormat,
    PublishMode,
};
use crate::{SystemVersion, UpgradeError, UpgradeStatus, UpgradeTask};

/// 当前已提交、可对外报告的有效发布。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActiveRelease {
    pub format_version: u32,
    pub upgrade_id: String,
    pub version: SystemVersion,
    pub deb_sha256: String,
    pub schema_version: u32,
    pub committed_at: i64,
}

/// 当前版本对应的最后有效 DEB 及其兼容性事实。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LastKnownGoodRelease {
    pub format_version: u32,
    pub version: SystemVersion,
    pub deb_sha256: String,
    pub schema_version: u32,
    pub tls_cert_sha256: String,
}

/// updater 写入、主服务导入的稳定终态结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UpgradeResult {
    pub format_version: u32,
    pub upgrade_id: String,
    pub status: UpgradeStatus,
    pub username: String,
    pub role: i32,
    pub source_ip: String,
    pub source_version: SystemVersion,
    pub target_version: SystemVersion,
    pub effective_version: SystemVersion,
    pub failed_stage: Option<String>,
    pub original_error: Option<String>,
    pub rollback_error: Option<String>,
    pub finished_at: i64,
}

/// 从明确路径读取并严格校验当前有效发布。
pub fn read_active_release(path: &Path) -> Result<ActiveRelease, UpgradeError> {
    read_optional_json(path)?
        .ok_or_else(|| UpgradeError::State("active-release.json 不存在".into()))
}

/// 严格读取 LKG 元数据并流式验证对应 DEB 摘要。
pub fn read_last_known_good(
    metadata_path: &Path,
    deb_path: &Path,
) -> Result<LastKnownGoodRelease, UpgradeError> {
    let metadata: LastKnownGoodRelease = read_optional_json(metadata_path)?
        .ok_or_else(|| UpgradeError::State("last-known-good.json 不存在".into()))?;
    let mut file = File::open(deb_path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    if hex::encode(digest.finalize()) != metadata.deb_sha256 {
        return Err(UpgradeError::State("LKG DEB 摘要不匹配".into()));
    }
    Ok(metadata)
}

/// 从明确路径读取并严格校验 updater 终态结果。
pub fn read_upgrade_result(path: &Path) -> Result<UpgradeResult, UpgradeError> {
    read_optional_json(path)?.ok_or_else(|| UpgradeError::State("升级结果不存在".into()))
}

impl UpgradeResult {
    /// 仅服务可继续运行的终态进入业务操作日志。
    pub fn is_business_log_importable(&self) -> bool {
        matches!(
            self.status,
            UpgradeStatus::Committed | UpgradeStatus::RolledBack | UpgradeStatus::ScheduleFailed
        )
    }

    /// 在 active release 已完成原子提交、结果文件尚未落盘时重建成功结果。
    pub fn committed_from_active(
        task: &UpgradeTask,
        active: &ActiveRelease,
        finished_at: i64,
    ) -> Result<Self, UpgradeError> {
        task.validate()?;
        active.validate_persisted()?;
        if task.upgrade_id != active.upgrade_id
            || task.target_version != active.version
            || !matches!(
                task.status,
                UpgradeStatus::HealthChecking | UpgradeStatus::Committed
            )
        {
            return Err(UpgradeError::State(
                "活动发布与升级任务不一致，不能重建 committed 结果".into(),
            ));
        }
        let finished_at = finished_at.max(active.committed_at);
        if finished_at <= 0 {
            return Err(UpgradeError::State("升级完成时间非法".into()));
        }
        Ok(Self {
            format_version: 1,
            upgrade_id: task.upgrade_id.clone(),
            status: UpgradeStatus::Committed,
            username: task.username.clone(),
            role: task.role,
            source_ip: task.source_ip.clone(),
            source_version: task.source_version,
            target_version: task.target_version,
            effective_version: active.version,
            failed_stage: None,
            original_error: None,
            rollback_error: None,
            finished_at,
        })
    }
}

pub trait DirectorySync: Send + Sync {
    fn sync(&self, path: &Path) -> io::Result<()>;
}

#[derive(Debug, Default)]
struct FsDirectorySync;

impl DirectorySync for FsDirectorySync {
    fn sync(&self, path: &Path) -> io::Result<()> {
        File::open(path)?.sync_all()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ActiveCommitError {
    #[error("有效发布 rename 前失败: {0}")]
    BeforeRename(UpgradeError),
    #[error("有效发布 rename 已完成但父目录同步失败: {0}")]
    AfterRename(UpgradeError),
}

/// 有效发布与 updater 终态结果的共享严格原子存储。
#[derive(Clone)]
pub struct ReleaseStateStore {
    root: PathBuf,
    directory_sync: Arc<dyn DirectorySync>,
}

impl std::fmt::Debug for ReleaseStateStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReleaseStateStore")
            .field("root", &self.root)
            .finish()
    }
}

impl ReleaseStateStore {
    pub fn new(root: PathBuf) -> Result<Self, UpgradeError> {
        // UpgradeTaskStore 负责建立并收紧同一升级根目录的权限。
        let _ = crate::UpgradeTaskStore::new(root.clone())?;
        Ok(Self {
            root,
            directory_sync: Arc::new(FsDirectorySync),
        })
    }

    pub fn with_directory_sync(
        root: PathBuf,
        directory_sync: Arc<dyn DirectorySync>,
    ) -> Result<Self, UpgradeError> {
        let _ = crate::UpgradeTaskStore::new(root.clone())?;
        Ok(Self {
            root,
            directory_sync,
        })
    }

    pub fn active_release(&self) -> Result<Option<ActiveRelease>, UpgradeError> {
        read_optional_json(&self.root.join("active-release.json"))
    }

    pub fn commit_active_release(&self, release: &ActiveRelease) -> Result<(), ActiveCommitError> {
        release
            .validate_persisted()
            .map_err(ActiveCommitError::BeforeRename)?;
        self.commit_active_release_inner(release)
    }

    pub fn result(&self, upgrade_id: &str) -> Result<Option<UpgradeResult>, UpgradeError> {
        validate_upgrade_id(upgrade_id)?;
        read_optional_json(&self.result_path(upgrade_id))
    }

    pub fn write_result(&self, result: &UpgradeResult) -> Result<(), UpgradeError> {
        result.validate_persisted()?;
        atomic_write_json(
            &self.result_path(&result.upgrade_id),
            result,
            PublishMode::Replace,
        )
    }

    fn result_path(&self, upgrade_id: &str) -> PathBuf {
        self.root
            .join("history")
            .join(format!("{upgrade_id}.result.json"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn commit_active_release_inner(
        &self,
        release: &ActiveRelease,
    ) -> Result<(), ActiveCommitError> {
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
        self.directory_sync
            .sync(&self.root)
            .map_err(UpgradeError::Io)
            .map_err(ActiveCommitError::AfterRename)
    }
}

impl PersistedFormat for ActiveRelease {
    fn validate_persisted(&self) -> Result<(), UpgradeError> {
        if self.format_version != 1
            || self.schema_version == 0
            || self.committed_at <= 0
            || !is_lower_hex_64(&self.deb_sha256)
        {
            return Err(UpgradeError::State("有效发布字段非法".into()));
        }
        validate_upgrade_id(&self.upgrade_id)
    }
}

impl PersistedFormat for LastKnownGoodRelease {
    fn validate_persisted(&self) -> Result<(), UpgradeError> {
        if self.format_version != 1
            || self.schema_version == 0
            || !is_lower_hex_64(&self.deb_sha256)
            || !is_lower_hex_64(&self.tls_cert_sha256)
        {
            return Err(UpgradeError::State("LKG 元数据字段非法".into()));
        }
        Ok(())
    }
}

impl PersistedFormat for UpgradeResult {
    fn validate_persisted(&self) -> Result<(), UpgradeError> {
        if self.format_version != 1
            || self.finished_at <= 0
            || !matches!(
                self.status,
                UpgradeStatus::Committed
                    | UpgradeStatus::RolledBack
                    | UpgradeStatus::RollbackFailed
                    | UpgradeStatus::ScheduleFailed
            )
        {
            return Err(UpgradeError::State("升级结果字段非法".into()));
        }
        validate_upgrade_id(&self.upgrade_id)
    }
}
