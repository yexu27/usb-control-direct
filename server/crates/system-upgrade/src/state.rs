//! 升级任务状态、合法转换和原子持久化。

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use rand::rngs::OsRng;
use rand::RngCore;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{SystemVersion, UpgradeError};

pub(crate) const TASK_FORMAT_VERSION: u32 = 1;
const MAX_JSON_SIZE: u64 = 1024 * 1024;

/// 系统升级任务状态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeStatus {
    Validating,
    Rejected,
    Prepared,
    Cancelled,
    Accepted,
    ScheduleFailed,
    Stopping,
    Installing,
    Migrating,
    Starting,
    HealthChecking,
    Committed,
    RollingBack,
    RolledBack,
    RollbackFailed,
}

impl UpgradeStatus {
    /// 判断能否从当前状态转换为目标状态。
    pub fn can_transition_to(self, target: Self) -> bool {
        use UpgradeStatus::*;

        matches!(
            (self, target),
            (Validating, Rejected)
                | (Validating, Prepared)
                | (Prepared, Cancelled)
                | (Prepared, Accepted)
                | (Accepted, ScheduleFailed)
                | (Accepted, Stopping)
                | (Accepted, RollingBack)
                | (Stopping, Installing)
                | (Stopping, RollingBack)
                | (Installing, Migrating)
                | (Installing, RollingBack)
                | (Migrating, Starting)
                | (Migrating, RollingBack)
                | (Starting, HealthChecking)
                | (Starting, RollingBack)
                | (HealthChecking, Committed)
                | (HealthChecking, RollingBack)
                | (RollingBack, RolledBack)
                | (RollingBack, RollbackFailed)
        )
    }

    /// 判断该状态是否不再占用活动任务槽位。
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Rejected
                | Self::Cancelled
                | Self::ScheduleFailed
                | Self::Committed
                | Self::RolledBack
                | Self::RollbackFailed
        )
    }
}

/// 系统升级任务的稳定持久化格式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UpgradeTask {
    pub format_version: u32,
    pub upgrade_id: String,
    pub status: UpgradeStatus,
    pub username: String,
    pub role: i32,
    pub source_ip: String,
    pub source_version: SystemVersion,
    pub target_version: SystemVersion,
    pub package_sha256: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl UpgradeTask {
    pub(crate) fn transition_to(
        &mut self,
        target: UpgradeStatus,
        updated_at: i64,
    ) -> Result<(), UpgradeError> {
        if !self.status.can_transition_to(target) {
            return Err(UpgradeError::State(format!(
                "非法升级状态转换: {:?} -> {:?}",
                self.status, target
            )));
        }
        self.status = target;
        self.updated_at = updated_at.max(self.created_at);
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<(), UpgradeError> {
        if self.format_version != TASK_FORMAT_VERSION {
            return Err(UpgradeError::State(format!(
                "不支持的升级任务格式版本: {}",
                self.format_version
            )));
        }
        validate_upgrade_id(&self.upgrade_id)?;
        if !is_lower_hex_64(&self.package_sha256) {
            return Err(UpgradeError::State("升级包摘要格式非法".into()));
        }
        if self.created_at <= 0 || self.updated_at < self.created_at {
            return Err(UpgradeError::State("升级任务时间字段非法".into()));
        }
        Ok(())
    }
}

/// `current.json` 与任务历史的唯一持久化入口。
#[derive(Debug, Clone)]
pub struct UpgradeTaskStore {
    root: PathBuf,
}

impl UpgradeTaskStore {
    /// 创建任务存储，并确保受控目录使用私有权限。
    pub fn new(root: PathBuf) -> Result<Self, UpgradeError> {
        create_private_dir_all(&root)?;
        create_private_dir_all(&root.join("history"))?;
        create_private_dir_all(&root.join("staging"))?;
        Ok(Self { root })
    }

    /// 返回升级根目录。
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 读取当前任务。未知字段、格式版本或非法字段均被拒绝。
    pub fn current(&self) -> Result<Option<UpgradeTask>, UpgradeError> {
        read_optional_json(&self.root.join("current.json"))
    }

    /// 读取指定历史任务。
    pub fn history(&self, upgrade_id: &str) -> Result<Option<UpgradeTask>, UpgradeError> {
        validate_upgrade_id(upgrade_id)?;
        read_optional_json(&self.history_path(upgrade_id))
    }

    /// 创建唯一的活动任务；已有 current 时不会覆盖。
    pub fn create(&self, task: &UpgradeTask) -> Result<(), UpgradeError> {
        task.validate()?;
        if task.status != UpgradeStatus::Prepared {
            return Err(UpgradeError::State(
                "只有 prepared 任务可以创建为 current".into(),
            ));
        }
        let path = self.root.join("current.json");
        if path.exists() {
            return Err(UpgradeError::Busy);
        }
        atomic_write_json(&path, task, PublishMode::CreateCurrent)
    }

    /// 按唯一状态机转换当前任务；终态原子写入 history 后清除 current。
    pub fn transition(
        &self,
        upgrade_id: &str,
        target: UpgradeStatus,
        updated_at: i64,
    ) -> Result<UpgradeTask, UpgradeError> {
        validate_upgrade_id(upgrade_id)?;
        let mut task = self
            .current()?
            .ok_or_else(|| UpgradeError::State("当前没有升级任务".into()))?;
        if task.upgrade_id != upgrade_id {
            return Err(UpgradeError::State("升级任务标识与 current 不一致".into()));
        }
        task.transition_to(target, updated_at)?;
        task.validate()?;

        if target.is_terminal() {
            atomic_write_json(
                &self.history_path(upgrade_id),
                &task,
                PublishMode::CreateHistory,
            )?;
            remove_file_and_sync_parent(&self.root.join("current.json"))?;
        } else {
            atomic_write_json(&self.root.join("current.json"), &task, PublishMode::Replace)?;
        }
        Ok(task)
    }

    pub(crate) fn record_rejected(&self, task: &UpgradeTask) -> Result<(), UpgradeError> {
        task.validate()?;
        if task.status != UpgradeStatus::Rejected {
            return Err(UpgradeError::State("拒绝历史必须处于 rejected".into()));
        }
        atomic_write_json(
            &self.history_path(&task.upgrade_id),
            task,
            PublishMode::CreateHistory,
        )
    }

    pub(crate) fn remove_staging(&self, upgrade_id: &str) -> Result<(), UpgradeError> {
        validate_upgrade_id(upgrade_id)?;
        let staging_parent = self.root.join("staging");
        let path = staging_parent.join(upgrade_id);
        match fs::remove_dir_all(&path) {
            Ok(()) => sync_dir(&staging_parent).map_err(UpgradeError::Io),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(UpgradeError::Io(error)),
        }
    }

    fn history_path(&self, upgrade_id: &str) -> PathBuf {
        self.root.join("history").join(format!("{upgrade_id}.json"))
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PublishMode {
    Replace,
    CreateCurrent,
    CreateHistory,
}

pub(crate) fn atomic_write_json<T: Serialize>(
    path: &Path,
    value: &T,
    mode: PublishMode,
) -> Result<(), UpgradeError> {
    let parent = path
        .parent()
        .ok_or_else(|| UpgradeError::State("JSON 文件缺少父目录".into()))?;
    create_private_dir_all(parent)?;
    let bytes = serde_json::to_vec(value)
        .map_err(|error| UpgradeError::State(format!("序列化升级状态失败: {error}")))?;
    if bytes.len() as u64 > MAX_JSON_SIZE {
        return Err(UpgradeError::State("升级状态 JSON 超过大小上限".into()));
    }

    let temporary = temporary_path(path);
    let write_result = (|| -> Result<(), UpgradeError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        match mode {
            PublishMode::Replace => {
                fs::rename(&temporary, path)?;
                sync_dir(parent)?;
            }
            PublishMode::CreateCurrent | PublishMode::CreateHistory => {
                if let Err(error) = fs::hard_link(&temporary, path) {
                    if error.kind() == io::ErrorKind::AlreadyExists {
                        return Err(match mode {
                            PublishMode::CreateCurrent => UpgradeError::Busy,
                            PublishMode::CreateHistory => UpgradeError::State(format!(
                                "升级历史文件已存在: {}",
                                path.display()
                            )),
                            PublishMode::Replace => unreachable!(),
                        });
                    }
                    return Err(UpgradeError::Io(error));
                }
                if let Err(error) = fs::remove_file(&temporary) {
                    rollback_created_file(path, parent);
                    return Err(UpgradeError::Io(error));
                }
                if let Err(error) = sync_dir(parent) {
                    rollback_created_file(path, parent);
                    return Err(UpgradeError::Io(error));
                }
            }
        }
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

pub(crate) fn read_optional_json<T>(path: &Path) -> Result<Option<T>, UpgradeError>
where
    T: DeserializeOwned + PersistedFormat,
{
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(UpgradeError::Io(error)),
    };
    let length = file.metadata()?.len();
    if length > MAX_JSON_SIZE {
        return Err(UpgradeError::State("升级状态 JSON 超过大小上限".into()));
    }
    let mut bytes = Vec::with_capacity(length as usize);
    file.read_to_end(&mut bytes)?;
    let value: T = serde_json::from_slice(&bytes)
        .map_err(|error| UpgradeError::State(format!("升级状态 JSON 无效: {error}")))?;
    value.validate_persisted()?;
    Ok(Some(value))
}

pub(crate) trait PersistedFormat {
    fn validate_persisted(&self) -> Result<(), UpgradeError>;
}

impl PersistedFormat for UpgradeTask {
    fn validate_persisted(&self) -> Result<(), UpgradeError> {
        self.validate()
    }
}

fn create_private_dir_all(path: &Path) -> io::Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

fn remove_file_and_sync_parent(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "file has no parent"))?;
    fs::remove_file(path)?;
    sync_dir(parent)
}

fn rollback_created_file(path: &Path, parent: &Path) {
    let _ = fs::remove_file(path);
    let _ = sync_dir(parent);
}

fn sync_dir(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut random = [0u8; 16];
    OsRng.fill_bytes(&mut random);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("upgrade.json");
    path.with_file_name(format!(".{name}.tmp-{}", hex::encode(random)))
}

pub(crate) fn validate_upgrade_id(upgrade_id: &str) -> Result<(), UpgradeError> {
    let valid = !upgrade_id.is_empty()
        && upgrade_id.len() <= 128
        && upgrade_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(UpgradeError::State("升级任务标识非法".into()))
    }
}

pub(crate) fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
