//! 主服务侧系统升级受理与发送后调度协调器。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::state::TASK_FORMAT_VERSION;
use crate::{
    PackageStager, PackageVerifier, SystemVersion, UpgradeError, UpgradeResult, UpgradeResultStore,
    UpgradeStateLock, UpgradeStatus, UpgradeTask, UpgradeTaskStore, VerificationContext,
};

/// 每次受理时从持久化存储读取的升级源状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpgradeSourceState {
    pub current_version: SystemVersion,
    pub current_schema: u32,
}

/// 升级源状态读取端口；由应用装配层实现数据库适配。
pub trait UpgradeSourceReader: Send + Sync {
    fn read(&self) -> Result<UpgradeSourceState, UpgradeError>;
}

/// 已通过网关授权检查的升级受理参数。
pub struct PrepareUpgradeRequest {
    pub package_bytes: Vec<u8>,
    pub client_target_version: String,
    pub client_sha256: String,
    pub username: String,
    pub role: i32,
    pub source_ip: String,
}

/// 校验并持久化成功后返回给网关的任务信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedUpgrade {
    pub upgrade_id: String,
    pub target_version: SystemVersion,
}

/// 响应成功发送后启动短周期 updater 的基础设施端口。
pub trait UpgradeScheduler: Send + Sync {
    fn start(&self, upgrade_id: &str) -> Result<(), UpgradeError>;
}

/// 已完成包校验后传给装置环境预检的不可变事实。
#[derive(Debug, Clone)]
pub struct UpgradePreflightRequest {
    pub package_size: u64,
    pub deb_size: u64,
    pub expanded_size: u64,
}

/// 主服务受理前的只读装置环境预检端口。
pub trait UpgradePreflight: Send + Sync {
    fn check(&self, request: &UpgradePreflightRequest) -> Result<(), UpgradeError>;
}

/// 串行化系统升级受理并维护任务状态。
pub struct UpgradeCoordinator {
    stager: PackageStager,
    verifier: PackageVerifier,
    source_reader: Arc<dyn UpgradeSourceReader>,
    protocol_version: u32,
    preflight: Arc<dyn UpgradePreflight>,
    scheduler: Arc<dyn UpgradeScheduler>,
    store: UpgradeTaskStore,
    results: UpgradeResultStore,
}

impl UpgradeCoordinator {
    /// 创建协调器。生产时间和任务标识均在协调器内部生成。
    pub fn new(
        root: PathBuf,
        stager: PackageStager,
        verifier: PackageVerifier,
        source_reader: Arc<dyn UpgradeSourceReader>,
        protocol_version: u32,
        preflight: Arc<dyn UpgradePreflight>,
        scheduler: Arc<dyn UpgradeScheduler>,
    ) -> Result<Self, UpgradeError> {
        let store = UpgradeTaskStore::new(root.clone())?;
        let results = UpgradeResultStore::new(root)?;
        Ok(Self {
            stager,
            verifier,
            source_reader,
            protocol_version,
            preflight,
            scheduler,
            store,
            results,
        })
    }

    /// 完成安全落盘、验签和 prepared 任务的原子持久化。
    pub fn prepare(&self, request: PrepareUpgradeRequest) -> Result<PreparedUpgrade, UpgradeError> {
        let guard = UpgradeStateLock::acquire(self.store.root())?;
        if self.store.current()?.is_some() {
            return Err(UpgradeError::Busy);
        }
        let source = self.source_reader.read()?;

        let upgrade_id = generate_upgrade_id();
        let now = unix_timestamp()?;
        let requested_target_version = parse_client_target_version(&request.client_target_version);
        let package_sha256 = hex::encode(Sha256::digest(&request.package_bytes));

        let staged = match self.stager.stage(&upgrade_id, &request.package_bytes) {
            Ok(staged) => staged,
            Err(error) => {
                if let Ok(target_version) = requested_target_version {
                    let mut task = self.new_task(
                        &upgrade_id,
                        &request,
                        source,
                        target_version,
                        package_sha256,
                        now,
                    );
                    let _ = self.reject_and_clean(&guard, &mut task);
                }
                return Err(error);
            }
        };
        let mut task = self.new_task(
            &upgrade_id,
            &request,
            source,
            staged.manifest.package_version,
            package_sha256,
            now,
        );
        if let Err(error) = requested_target_version {
            let _ = self.reject_and_clean(&guard, &mut task);
            return Err(error);
        }
        if staged.manifest.package_version <= source.current_version {
            let _ = self.reject_and_clean(&guard, &mut task);
            return Err(UpgradeError::VersionNotGreater);
        }
        let context = VerificationContext {
            current_schema: source.current_schema,
            protocol_version: self.protocol_version,
            client_target_version: request.client_target_version,
            client_sha256: request.client_sha256,
        };
        let verified = match self.verifier.verify(staged, &context) {
            Ok(verified) => verified,
            Err(error) => {
                let _ = self.reject_and_clean(&guard, &mut task);
                return Err(error);
            }
        };
        let preflight_request = UpgradePreflightRequest {
            package_size: request.package_bytes.len() as u64,
            deb_size: verified.staged.manifest.deb_size,
            expanded_size: verified.deb_metadata.expanded_size,
        };
        if let Err(error) = self.preflight.check(&preflight_request) {
            let _ = self.reject_and_clean(&guard, &mut task);
            return Err(error);
        }
        task.target_version = verified.staged.manifest.package_version;
        task.transition_to(UpgradeStatus::Prepared, unix_timestamp()?)?;
        if let Err(error) = self.store.create(&guard, &task) {
            let _ = self.store.remove_staging(&guard, &upgrade_id);
            return Err(error);
        }

        Ok(PreparedUpgrade {
            upgrade_id,
            target_version: task.target_version,
        })
    }

    fn new_task(
        &self,
        upgrade_id: &str,
        request: &PrepareUpgradeRequest,
        source: UpgradeSourceState,
        target_version: SystemVersion,
        package_sha256: String,
        created_at: i64,
    ) -> UpgradeTask {
        UpgradeTask {
            format_version: TASK_FORMAT_VERSION,
            upgrade_id: upgrade_id.to_string(),
            status: UpgradeStatus::Validating,
            username: request.username.clone(),
            role: request.role,
            source_ip: request.source_ip.clone(),
            source_version: source.current_version,
            target_version,
            package_sha256,
            created_at,
            updated_at: created_at,
        }
    }

    /// 仅在响应已完成发送后把 prepared 转为 accepted。
    pub fn accept_after_response(&self, upgrade_id: &str) -> Result<UpgradeTask, UpgradeError> {
        let guard = UpgradeStateLock::acquire(self.store.root())?;
        self.store.transition(
            &guard,
            upgrade_id,
            UpgradeStatus::Accepted,
            unix_timestamp()?,
        )
    }

    /// 仅调度 accepted 任务；调度失败落入终态且保留 staging。
    pub fn schedule(&self, upgrade_id: &str) -> Result<(), UpgradeError> {
        let task = {
            let _guard = UpgradeStateLock::acquire(self.store.root())?;
            let task = self
                .store
                .current()?
                .ok_or_else(|| UpgradeError::State("当前没有升级任务".into()))?;
            if task.upgrade_id != upgrade_id || task.status != UpgradeStatus::Accepted {
                return Err(UpgradeError::State(
                    "只有 accepted 任务可以调度 updater".into(),
                ));
            }
            task
        };

        match self.scheduler.start(upgrade_id) {
            Ok(()) => Ok(()),
            Err(schedule_error) => {
                let guard = UpgradeStateLock::acquire(self.store.root())?;
                let finished_at = unix_timestamp()?;
                let result = UpgradeResult {
                    format_version: 1,
                    upgrade_id: task.upgrade_id.clone(),
                    status: UpgradeStatus::ScheduleFailed,
                    username: task.username.clone(),
                    role: task.role,
                    source_ip: task.source_ip.clone(),
                    source_version: task.source_version,
                    target_version: task.target_version,
                    effective_version: task.source_version,
                    failed_stage: Some("scheduling".into()),
                    original_error: Some(schedule_error.to_string()),
                    finished_at,
                };
                let result_error = self.results.write(&guard, &result).err();
                let state_error = self
                    .store
                    .ensure_terminal(
                        &guard,
                        upgrade_id,
                        UpgradeStatus::ScheduleFailed,
                        finished_at,
                    )
                    .err();
                let persistence_errors = [result_error, state_error]
                    .into_iter()
                    .flatten()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>();
                if !persistence_errors.is_empty() {
                    return Err(UpgradeError::State(format!(
                        "updater 调度失败且终态持久化不完整: {schedule_error}; {}",
                        persistence_errors.join("；")
                    )));
                }
                Err(schedule_error)
            }
        }
    }

    /// 响应发送失败时取消 prepared 任务并删除其 staging。
    pub fn response_failed(&self, upgrade_id: &str) -> Result<(), UpgradeError> {
        let guard = UpgradeStateLock::acquire(self.store.root())?;
        let task = self
            .store
            .current()?
            .ok_or_else(|| UpgradeError::State("当前没有升级任务".into()))?;
        if task.upgrade_id != upgrade_id || task.status != UpgradeStatus::Prepared {
            return Err(UpgradeError::State("只有 prepared 任务可以取消响应".into()));
        }
        self.store.remove_staging(&guard, upgrade_id)?;
        self.store.transition(
            &guard,
            upgrade_id,
            UpgradeStatus::Cancelled,
            unix_timestamp()?,
        )?;
        Ok(())
    }

    fn reject_and_clean(
        &self,
        lock: &UpgradeStateLock,
        task: &mut UpgradeTask,
    ) -> Result<(), UpgradeError> {
        task.transition_to(UpgradeStatus::Rejected, unix_timestamp()?)?;
        let history_result = self.store.record_rejected(lock, task);
        let cleanup_result = self.store.remove_staging(lock, &task.upgrade_id);
        history_result.and(cleanup_result)
    }
}

fn parse_client_target_version(value: &str) -> Result<SystemVersion, UpgradeError> {
    let version = value
        .strip_prefix('v')
        .or_else(|| value.strip_prefix('V'))
        .unwrap_or(value);
    SystemVersion::parse(version)
}

fn generate_upgrade_id() -> String {
    let mut random = [0u8; 16];
    OsRng.fill_bytes(&mut random);
    format!("upgrade-{}", hex::encode(random))
}

fn unix_timestamp() -> Result<i64, UpgradeError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| UpgradeError::State("系统时间早于 UNIX epoch".into()))?;
    i64::try_from(duration.as_secs())
        .map_err(|_| UpgradeError::State("系统时间超出可持久化范围".into()))
}
