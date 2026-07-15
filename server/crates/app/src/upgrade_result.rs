//! 系统升级终态结果的幂等业务日志导入。

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use common::audit_const::{action_type, log_type};
use storage::model::OperationLogInsert;
use storage::{InsertOnceResult, Storage};
use system_upgrade::{
    read_active_release, read_upgrade_result, ReleaseStateStore, UpgradeResult, UpgradeStatus,
    UpgradeTask, UpgradeTaskStore,
};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportDisposition {
    Imported,
    AlreadyImported,
    NotImportable,
}

pub trait UpgradeImportStorage: Send + Sync {
    fn insert_once(&self, item: &OperationLogInsert) -> Result<InsertOnceResult, String>;
    fn sync_system_version(&self, version: &str) -> Result<(), String>;
}

/// 为已接受的升级任务注册终态结果观察。
pub trait UpgradeResultObserver: Send + Sync {
    fn observe(&self, task: UpgradeTask);
}

/// 在 Tokio 运行时中观察升级结果的生产适配器。
pub struct TokioUpgradeResultObserver {
    importer: Arc<UpgradeResultImporter>,
}

impl TokioUpgradeResultObserver {
    pub fn new(importer: Arc<UpgradeResultImporter>) -> Self {
        Self { importer }
    }
}

impl UpgradeResultObserver for TokioUpgradeResultObserver {
    fn observe(&self, task: UpgradeTask) {
        let importer = Arc::clone(&self.importer);
        tokio::spawn(async move {
            if let Err(error) = importer.monitor_active_task(task).await {
                tracing::error!(reason = %error, "系统升级终态结果观察器退出");
            }
        });
    }
}

impl UpgradeImportStorage for Storage {
    fn insert_once(&self, item: &OperationLogInsert) -> Result<InsertOnceResult, String> {
        self.operation_log_insert_once_by_request_id(item)
            .map_err(|error| error.to_string())
    }

    fn sync_system_version(&self, version: &str) -> Result<(), String> {
        self.config_set("system_version", version)
            .map_err(|error| error.to_string())
    }
}

pub struct UpgradeResultImporter {
    root: PathBuf,
    storage: Arc<dyn UpgradeImportStorage>,
}

impl UpgradeResultImporter {
    pub fn new(root: PathBuf, storage: Arc<Storage>) -> Self {
        Self { root, storage }
    }

    pub fn with_storage(root: PathBuf, storage: Arc<dyn UpgradeImportStorage>) -> Self {
        Self { root, storage }
    }

    pub fn import_result(&self, result: &UpgradeResult) -> Result<ImportDisposition, String> {
        if !result.is_business_log_importable() {
            return Ok(ImportDisposition::NotImportable);
        }
        let done = self.done_path(&result.upgrade_id);
        if done.is_file() {
            return Ok(ImportDisposition::AlreadyImported);
        }

        let active = read_active_release(&self.root.join("active-release.json"))
            .map_err(|error| error.to_string())?;
        if active.version != result.effective_version
            || (result.status == UpgradeStatus::Committed && active.upgrade_id != result.upgrade_id)
        {
            return Err("升级结果与当前有效发布不一致".into());
        }

        let insert = self.storage.insert_once(&operation_log(result))?;
        self.storage
            .sync_system_version(&active.version.to_string())?;
        write_done_marker(&done, &result.upgrade_id).map_err(|error| error.to_string())?;

        Ok(match insert {
            InsertOnceResult::Inserted(_) => ImportDisposition::Imported,
            InsertOnceResult::AlreadyExists(_) => ImportDisposition::AlreadyImported,
        })
    }

    pub fn scan_pending(&self) -> Result<usize, String> {
        let history = self.root.join("history");
        if !history.exists() {
            return Ok(0);
        }
        let mut paths = fs::read_dir(&history)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".result.json"))
            })
            .collect::<Vec<_>>();
        paths.sort();

        let mut imported = 0;
        for path in paths {
            let result = read_upgrade_result(&path).map_err(|error| error.to_string())?;
            if self.done_path(&result.upgrade_id).is_file() {
                continue;
            }
            if self.import_result(&result)? == ImportDisposition::Imported {
                imported += 1;
            }
        }
        Ok(imported)
    }

    /// 启动扫描失败后，每秒重试现有待导入历史；全部处理成功后退出。
    pub async fn retry_pending_until_done(&self) -> Result<(), String> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            match self.scan_pending() {
                Ok(_) => return Ok(()),
                Err(error) => {
                    warn!(reason = %error, "系统升级终态结果仍待导入");
                }
            }
        }
    }

    /// 仅观察启动时存在的单个活动任务；任务终结后立即退出。
    pub async fn monitor_active_task(&self, task: UpgradeTask) -> Result<(), String> {
        let tasks = UpgradeTaskStore::new(self.root.clone()).map_err(|error| error.to_string())?;
        let releases =
            ReleaseStateStore::new(self.root.clone()).map_err(|error| error.to_string())?;
        loop {
            if let Some(result) = releases
                .result(&task.upgrade_id)
                .map_err(|error| error.to_string())?
            {
                if result.status == UpgradeStatus::RollbackFailed {
                    return Ok(());
                }
                if result.is_business_log_importable() {
                    match self.import_result(&result) {
                        Ok(_) => return Ok(()),
                        Err(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            continue;
                        }
                    }
                }
            }

            if let Some(active) = releases
                .active_release()
                .map_err(|error| error.to_string())?
            {
                if active.upgrade_id == task.upgrade_id {
                    let result = UpgradeResult::committed_from_active(
                        &task,
                        &active,
                        common::time::now_unix(),
                    )
                    .map_err(|error| error.to_string())?;
                    releases
                        .write_result(&result)
                        .map_err(|error| error.to_string())?;
                    match self.import_result(&result) {
                        Ok(_) => return Ok(()),
                        Err(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            continue;
                        }
                    }
                }
            }

            let history = tasks
                .history(&task.upgrade_id)
                .map_err(|error| error.to_string())?;
            if history
                .as_ref()
                .is_some_and(|value| value.status == UpgradeStatus::RollbackFailed)
            {
                return Ok(());
            }
            match tasks.current().map_err(|error| error.to_string())? {
                Some(current) if current.upgrade_id == task.upgrade_id => {}
                Some(_) => return Ok(()),
                None if history.is_none() => return Ok(()),
                None => {}
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    fn done_path(&self, upgrade_id: &str) -> PathBuf {
        self.root.join("imports").join(format!("{upgrade_id}.done"))
    }
}

fn operation_log(result: &UpgradeResult) -> OperationLogInsert {
    let succeeded = result.status == UpgradeStatus::Committed;
    let detail = serde_json::json!({
        "status": status_name(result.status),
        "failed_stage": result.failed_stage,
        "original_error": result.original_error,
        "rollback_error": result.rollback_error,
    })
    .to_string();
    OperationLogInsert {
        op_time: result.finished_at,
        username: result.username.clone(),
        role: result.role,
        log_type: log_type::PROGRAM_UPGRADE.into(),
        action_type: Some(action_type::SYSTEM_UPGRADE.into()),
        target: Some(result.target_version.to_string()),
        before_value: Some(result.source_version.to_string()),
        after_value: Some(result.effective_version.to_string()),
        related_file: None,
        related_version: Some(result.target_version.to_string()),
        result: if succeeded { 0 } else { 1 },
        fail_reason: result.original_error.clone(),
        source_ip: Some(result.source_ip.clone()),
        app_version: Some(result.effective_version.to_string()),
        session_id: None,
        request_id: Some(format!("system-upgrade:{}:result", result.upgrade_id)),
        detail: Some(detail),
    }
}

fn status_name(status: UpgradeStatus) -> &'static str {
    match status {
        UpgradeStatus::Committed => "committed",
        UpgradeStatus::RolledBack => "rolled_back",
        UpgradeStatus::ScheduleFailed => "schedule_failed",
        _ => "not_importable",
    }
}

fn write_done_marker(path: &Path, upgrade_id: &str) -> std::io::Result<()> {
    static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("导入标记缺少父目录"))?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)?;
    if path.is_file() {
        return Ok(());
    }
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = path.with_file_name(format!(
        ".{upgrade_id}.{}.{}.tmp",
        std::process::id(),
        sequence
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(b"imported\n")?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
