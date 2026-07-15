//! 当前系统升级任务终态的幂等业务日志导入。

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
    ActiveReleaseStore, UpgradeResult, UpgradeResultStore, UpgradeStatus, UpgradeTask,
    UpgradeTaskStore,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportDisposition {
    Imported,
    AlreadyImported,
    NotImportable,
}

pub trait UpgradeImportStorage: Send + Sync {
    fn insert_once(&self, item: &OperationLogInsert) -> Result<InsertOnceResult, String>;
}

pub trait UpgradeResultObserver: Send + Sync {
    fn observe(&self, task: UpgradeTask);
}

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
        let active = ActiveReleaseStore::new(self.root.clone())
            .and_then(|store| store.current())
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "active-release.json 不存在".to_string())?;
        if active.version != result.effective_version
            || (result.status == UpgradeStatus::Committed && active.upgrade_id != result.upgrade_id)
        {
            return Err("升级结果与当前有效发布不一致".into());
        }
        let insert = self.storage.insert_once(&operation_log(result))?;
        write_done_marker(&done, &result.upgrade_id).map_err(|error| error.to_string())?;
        Ok(match insert {
            InsertOnceResult::Inserted(_) => ImportDisposition::Imported,
            InsertOnceResult::AlreadyExists(_) => ImportDisposition::AlreadyImported,
        })
    }

    pub async fn monitor_active_task(&self, initial: UpgradeTask) -> Result<(), String> {
        let tasks = UpgradeTaskStore::new(self.root.clone()).map_err(|error| error.to_string())?;
        let results =
            UpgradeResultStore::new(self.root.clone()).map_err(|error| error.to_string())?;
        let releases =
            ActiveReleaseStore::new(self.root.clone()).map_err(|error| error.to_string())?;
        loop {
            if let Some(result) = results
                .get(&initial.upgrade_id)
                .map_err(|e| e.to_string())?
            {
                tasks
                    .ensure_terminal(&initial.upgrade_id, result.status, result.finished_at)
                    .map_err(|error| error.to_string())?;
                match self.import_result(&result) {
                    Ok(_) => return Ok(()),
                    Err(_) => {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }

            let current = tasks.current().map_err(|error| error.to_string())?;
            if let Some(active) = releases.current().map_err(|error| error.to_string())? {
                if active.upgrade_id == initial.upgrade_id {
                    let task = match current
                        .as_ref()
                        .filter(|task| task.upgrade_id == initial.upgrade_id)
                    {
                        Some(task) => task.clone(),
                        None => tasks
                            .history(&initial.upgrade_id)
                            .map_err(|error| error.to_string())?
                            .ok_or_else(|| "已提交发布缺少对应任务".to_string())?,
                    };
                    let result = UpgradeResult::committed_from_active(
                        &task,
                        &active,
                        common::time::now_unix(),
                    )
                    .map_err(|error| error.to_string())?;
                    results.write(&result).map_err(|error| error.to_string())?;
                    tasks
                        .ensure_terminal(
                            &initial.upgrade_id,
                            UpgradeStatus::Committed,
                            result.finished_at,
                        )
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

            match current {
                Some(task) if task.upgrade_id == initial.upgrade_id => {}
                Some(_) => return Ok(()),
                None => {
                    if tasks
                        .history(&initial.upgrade_id)
                        .map_err(|e| e.to_string())?
                        .is_some()
                    {
                        return Ok(());
                    }
                    return Ok(());
                }
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
        UpgradeStatus::ScheduleFailed => "schedule_failed",
        UpgradeStatus::Failed => "failed",
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
