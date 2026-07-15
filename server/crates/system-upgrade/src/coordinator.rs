//! 主服务侧系统升级受理与发送后调度协调器。

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::state::TASK_FORMAT_VERSION;
use crate::{
    PackageStager, PackageVerifier, ReleaseStateStore, SystemVersion, UpgradeError, UpgradeResult,
    UpgradeStatus, UpgradeTask, UpgradeTaskStore, VerificationContext,
};

/// 装置端固定的升级兼容性上下文。
#[derive(Debug, Clone, Copy)]
pub struct UpgradeEnvironment {
    pub current_version: SystemVersion,
    pub current_schema: u32,
    pub supported_schema_max: u32,
    pub protocol_version: u32,
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
    pub source_version: SystemVersion,
    pub target_version: SystemVersion,
    pub schema_from: u32,
    pub schema_to: u32,
}

/// 主服务受理前的只读装置环境预检端口。
pub trait UpgradePreflight: Send + Sync {
    fn check(&self, request: &UpgradePreflightRequest) -> Result<(), UpgradeError>;
}

/// 串行化系统升级受理并维护任务状态。
pub struct UpgradeCoordinator {
    stager: PackageStager,
    verifier: PackageVerifier,
    environment: UpgradeEnvironment,
    preflight: Arc<dyn UpgradePreflight>,
    scheduler: Arc<dyn UpgradeScheduler>,
    store: UpgradeTaskStore,
    results: ReleaseStateStore,
    admission_lock: Mutex<()>,
}

impl UpgradeCoordinator {
    /// 创建协调器。生产时间和任务标识均在协调器内部生成。
    pub fn new(
        root: PathBuf,
        stager: PackageStager,
        verifier: PackageVerifier,
        environment: UpgradeEnvironment,
        preflight: Arc<dyn UpgradePreflight>,
        scheduler: Arc<dyn UpgradeScheduler>,
    ) -> Result<Self, UpgradeError> {
        let store = UpgradeTaskStore::new(root.clone())?;
        let results = ReleaseStateStore::new(root)?;
        Ok(Self {
            stager,
            verifier,
            environment,
            preflight,
            scheduler,
            store,
            results,
            admission_lock: Mutex::new(()),
        })
    }

    /// 完成安全落盘、验签和 prepared 任务的原子持久化。
    pub fn prepare(&self, request: PrepareUpgradeRequest) -> Result<PreparedUpgrade, UpgradeError> {
        let _guard = self.lock()?;
        if self.store.current()?.is_some() {
            return Err(UpgradeError::Busy);
        }

        let upgrade_id = generate_upgrade_id();
        let now = unix_timestamp()?;
        let requested_target_version = parse_client_target_version(&request.client_target_version);
        let package_sha256 = hex::encode(Sha256::digest(&request.package_bytes));

        let staged = match self.stager.stage(&upgrade_id, &request.package_bytes) {
            Ok(staged) => staged,
            Err(error) => {
                if let Ok(target_version) = requested_target_version {
                    let mut task =
                        self.new_task(&upgrade_id, &request, target_version, package_sha256, now);
                    let _ = self.reject_and_clean(&mut task);
                }
                return Err(error);
            }
        };
        let mut task = self.new_task(
            &upgrade_id,
            &request,
            staged.manifest.package_version,
            package_sha256,
            now,
        );
        if let Err(error) = requested_target_version {
            let _ = self.reject_and_clean(&mut task);
            return Err(error);
        }
        let context = VerificationContext {
            current_version: self.environment.current_version,
            current_schema: self.environment.current_schema,
            supported_schema_max: self.environment.supported_schema_max,
            protocol_version: self.environment.protocol_version,
            client_target_version: request.client_target_version,
            client_sha256: request.client_sha256,
        };
        let verified = match self.verifier.verify(staged, &context) {
            Ok(verified) => verified,
            Err(error) => {
                let _ = self.reject_and_clean(&mut task);
                return Err(error);
            }
        };
        let preflight_request = UpgradePreflightRequest {
            package_size: request.package_bytes.len() as u64,
            deb_size: verified.staged.manifest.deb_size,
            expanded_size: verified.deb_metadata.expanded_size,
            source_version: self.environment.current_version,
            target_version: verified.staged.manifest.package_version,
            schema_from: verified.staged.manifest.schema_from,
            schema_to: verified.staged.manifest.schema_to,
        };
        if let Err(error) = self.preflight.check(&preflight_request) {
            let _ = self.reject_and_clean(&mut task);
            return Err(error);
        }
        task.target_version = verified.staged.manifest.package_version;
        task.transition_to(UpgradeStatus::Prepared, unix_timestamp()?)?;
        if let Err(error) = self.store.create(&task) {
            let _ = self.store.remove_staging(&upgrade_id);
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
            source_version: self.environment.current_version,
            target_version,
            package_sha256,
            created_at,
            updated_at: created_at,
        }
    }

    /// 仅在响应已完成发送后把 prepared 转为 accepted。
    pub fn accept_after_response(&self, upgrade_id: &str) -> Result<UpgradeTask, UpgradeError> {
        let _guard = self.lock()?;
        self.store
            .transition(upgrade_id, UpgradeStatus::Accepted, unix_timestamp()?)
    }

    /// 仅调度 accepted 任务；调度失败落入终态且保留 staging。
    pub fn schedule(&self, upgrade_id: &str) -> Result<(), UpgradeError> {
        let _guard = self.lock()?;
        let task = self
            .store
            .current()?
            .ok_or_else(|| UpgradeError::State("当前没有升级任务".into()))?;
        if task.upgrade_id != upgrade_id || task.status != UpgradeStatus::Accepted {
            return Err(UpgradeError::State(
                "只有 accepted 任务可以调度 updater".into(),
            ));
        }

        match self.scheduler.start(upgrade_id) {
            Ok(()) => Ok(()),
            Err(schedule_error) => {
                let failed = self.store.transition(
                    upgrade_id,
                    UpgradeStatus::ScheduleFailed,
                    unix_timestamp()?,
                )?;
                self.results.write_result(&UpgradeResult {
                    format_version: 1,
                    upgrade_id: failed.upgrade_id.clone(),
                    status: UpgradeStatus::ScheduleFailed,
                    username: failed.username.clone(),
                    role: failed.role,
                    source_ip: failed.source_ip.clone(),
                    source_version: failed.source_version,
                    target_version: failed.target_version,
                    effective_version: failed.source_version,
                    failed_stage: Some("scheduling".into()),
                    original_error: Some(schedule_error.to_string()),
                    rollback_error: None,
                    finished_at: failed.updated_at,
                })?;
                Err(schedule_error)
            }
        }
    }

    /// 响应发送失败时取消 prepared 任务并删除其 staging。
    pub fn response_failed(&self, upgrade_id: &str) -> Result<(), UpgradeError> {
        let _guard = self.lock()?;
        let task = self
            .store
            .current()?
            .ok_or_else(|| UpgradeError::State("当前没有升级任务".into()))?;
        if task.upgrade_id != upgrade_id || task.status != UpgradeStatus::Prepared {
            return Err(UpgradeError::State("只有 prepared 任务可以取消响应".into()));
        }
        self.store.remove_staging(upgrade_id)?;
        self.store
            .transition(upgrade_id, UpgradeStatus::Cancelled, unix_timestamp()?)?;
        Ok(())
    }

    fn reject_and_clean(&self, task: &mut UpgradeTask) -> Result<(), UpgradeError> {
        task.transition_to(UpgradeStatus::Rejected, unix_timestamp()?)?;
        let history_result = self.store.record_rejected(task);
        let cleanup_result = self.store.remove_staging(&task.upgrade_id);
        history_result.and(cleanup_result)
    }

    fn lock(&self) -> Result<MutexGuard<'_, ()>, UpgradeError> {
        self.admission_lock
            .lock()
            .map_err(|_| UpgradeError::State("升级受理锁已损坏".into()))
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
