//! updater 写入、主服务观察的稳定终态结果。

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::{
    atomic_write_json, create_private_dir_all, read_optional_json, sync_dir, validate_upgrade_id,
    PersistedFormat, PublishMode,
};
use crate::{
    ActiveRelease, SystemVersion, UpgradeError, UpgradeStatus, UpgradeTask, UpgradeTaskStore,
};

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
    pub finished_at: i64,
}

impl UpgradeResult {
    pub fn is_business_log_importable(&self) -> bool {
        matches!(
            self.status,
            UpgradeStatus::Committed | UpgradeStatus::ScheduleFailed | UpgradeStatus::Failed
        )
    }

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
            finished_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UpgradeResultStore {
    root: PathBuf,
}

impl UpgradeResultStore {
    pub fn new(root: PathBuf) -> Result<Self, UpgradeError> {
        UpgradeTaskStore::new(root.clone())?;
        create_private_dir_all(&root.join("results"))?;
        Ok(Self { root })
    }

    pub fn get(&self, upgrade_id: &str) -> Result<Option<UpgradeResult>, UpgradeError> {
        validate_upgrade_id(upgrade_id)?;
        read_optional_json(&self.path(upgrade_id))
    }

    pub fn write(&self, result: &UpgradeResult) -> Result<(), UpgradeError> {
        result.validate_persisted()?;
        atomic_write_json(&self.path(&result.upgrade_id), result, PublishMode::Replace)?;
        self.prune()
    }

    fn path(&self, upgrade_id: &str) -> PathBuf {
        self.root
            .join("results")
            .join(format!("{upgrade_id}.result.json"))
    }

    fn prune(&self) -> Result<(), UpgradeError> {
        const LIMIT: usize = 20;
        let directory = self.root.join("results");
        let mut entries = Vec::new();
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            if !path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.ends_with(".result.json"))
            {
                continue;
            }
            if let Ok(Some(result)) = read_optional_json::<UpgradeResult>(&path) {
                entries.push((result.finished_at, result.upgrade_id, path));
            }
        }
        entries.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
        let remove_count = entries.len().saturating_sub(LIMIT);
        for (_, _, path) in entries.into_iter().take(remove_count) {
            fs::remove_file(path)?;
        }
        if remove_count > 0 {
            sync_dir(&directory)?;
        }
        Ok(())
    }
}

impl PersistedFormat for UpgradeResult {
    fn validate_persisted(&self) -> Result<(), UpgradeError> {
        if self.format_version != 1 || self.finished_at <= 0 {
            return Err(UpgradeError::State("升级结果字段非法".into()));
        }
        validate_upgrade_id(&self.upgrade_id)?;
        match self.status {
            UpgradeStatus::Committed => {
                if self.failed_stage.is_some() || self.original_error.is_some() {
                    return Err(UpgradeError::State("成功结果包含失败字段".into()));
                }
            }
            UpgradeStatus::ScheduleFailed | UpgradeStatus::Failed => {
                if self.failed_stage.as_deref().is_none_or(str::is_empty)
                    || self.original_error.as_deref().is_none_or(str::is_empty)
                {
                    return Err(UpgradeError::State("失败结果缺少阶段或原因".into()));
                }
            }
            _ => return Err(UpgradeError::State("升级结果不是允许的终态".into())),
        }
        Ok(())
    }
}
