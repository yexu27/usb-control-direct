//! 升级命令边界、单向安装顺序和有效发布提交点。

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use system_upgrade::{
    read_installed_release, DebInspector, DpkgDebInspector, InstalledRelease, PackageStager,
    PackageVerifier, SystemVersion, UpgradeManifest, UpgradeResult, UpgradeResultStore,
    UpgradeStateLock, UpgradeStatus, UpgradeTask, UpgradeTaskStore, VerificationContext,
};
use usb_control_db_migrate::UpgradeDatabaseState;
use wait_timeout::ChildExt;

use crate::health::{check_health, read_restart_count, HealthExpectation};
use crate::migration::run_migration;
use crate::{ManagedInstallGuard, UpdaterError, UpgradeDatabase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub stage: String,
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

pub trait CommandRunner {
    fn run(&self, command: &CommandSpec) -> Result<CommandOutput, UpdaterError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run(&self, command: &CommandSpec) -> Result<CommandOutput, UpdaterError> {
        let program = command.program.to_string_lossy().into_owned();
        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| UpdaterError::CommandSpawn {
                stage: command.stage.clone(),
                program: program.clone(),
                source,
            })?;
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(UpdaterError::CommandSpawn {
                    stage: command.stage.clone(),
                    program: program.clone(),
                    source: std::io::Error::other("无法接管命令 stdout"),
                });
            }
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(UpdaterError::CommandSpawn {
                    stage: command.stage.clone(),
                    program: program.clone(),
                    source: std::io::Error::other("无法接管命令 stderr"),
                });
            }
        };
        let stdout_reader = match std::thread::Builder::new()
            .name("updater-stdout".into())
            .spawn(move || read_bounded_output(stdout))
        {
            Ok(reader) => reader,
            Err(source) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(UpdaterError::CommandSpawn {
                    stage: command.stage.clone(),
                    program: program.clone(),
                    source,
                });
            }
        };
        let stderr_reader = match std::thread::Builder::new()
            .name("updater-stderr".into())
            .spawn(move || read_bounded_output(stderr))
        {
            Ok(reader) => reader,
            Err(source) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                return Err(UpdaterError::CommandSpawn {
                    stage: command.stage.clone(),
                    program,
                    source,
                });
            }
        };
        let wait_result = child.wait_timeout(command.timeout);
        let timed_out = matches!(&wait_result, Ok(None));
        let mut wait_error = None;
        let status = match wait_result {
            Ok(Some(status)) => Some(status),
            Ok(None) => {
                let _ = child.kill();
                child.wait().ok()
            }
            Err(source) => {
                wait_error = Some(source);
                let _ = child.kill();
                child.wait().ok()
            }
        };
        let (stdout, stdout_truncated) = join_reader(stdout_reader, command, "stdout")?;
        let (stderr, stderr_truncated) = join_reader(stderr_reader, command, "stderr")?;
        if let Some(source) = wait_error {
            return Err(UpdaterError::CommandSpawn {
                stage: command.stage.clone(),
                program,
                source,
            });
        }
        if timed_out {
            return Err(UpdaterError::CommandTimeout {
                stage: command.stage.clone(),
                program,
            });
        }
        let status = status.ok_or_else(|| UpdaterError::CommandSpawn {
            stage: command.stage.clone(),
            program: program.clone(),
            source: std::io::Error::other("命令退出状态不可用"),
        })?;
        let output = CommandOutput {
            success: status.success(),
            stdout,
            stderr,
            stdout_truncated,
            stderr_truncated,
        };
        if output.success {
            Ok(output)
        } else {
            Err(UpdaterError::CommandFailed {
                stage: command.stage.clone(),
                program,
                status: status.code(),
            })
        }
    }
}

const MAX_COMMAND_OUTPUT: usize = 64 * 1024;

fn read_bounded_output(mut reader: impl Read) -> std::io::Result<(Vec<u8>, bool)> {
    let mut kept = Vec::with_capacity(MAX_COMMAND_OUTPUT);
    let mut buffer = [0u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok((kept, truncated));
        }
        let keep = MAX_COMMAND_OUTPUT.saturating_sub(kept.len()).min(read);
        kept.extend_from_slice(&buffer[..keep]);
        truncated |= keep < read;
    }
}

fn join_reader(
    reader: std::thread::JoinHandle<std::io::Result<(Vec<u8>, bool)>>,
    command: &CommandSpec,
    stream: &str,
) -> Result<(Vec<u8>, bool), UpdaterError> {
    reader
        .join()
        .map_err(|_| UpdaterError::CommandSpawn {
            stage: command.stage.clone(),
            program: command.program.to_string_lossy().into_owned(),
            source: std::io::Error::other(format!("{stream} reader thread panic")),
        })?
        .map_err(|source| UpdaterError::CommandSpawn {
            stage: command.stage.clone(),
            program: command.program.to_string_lossy().into_owned(),
            source,
        })
}

pub(crate) fn command(
    stage: &str,
    program: impl Into<PathBuf>,
    args: impl IntoIterator<Item = impl Into<OsString>>,
    timeout: Duration,
) -> CommandSpec {
    CommandSpec {
        stage: stage.into(),
        program: program.into(),
        args: args.into_iter().map(Into::into).collect(),
        timeout,
    }
}

#[derive(Debug, Clone)]
pub struct UpgradePaths {
    pub root: PathBuf,
    pub staging_dir: PathBuf,
    pub ready_file: PathBuf,
    pub installed_release: PathBuf,
    pub tls_certificate: PathBuf,
    pub database: PathBuf,
    pub sql_root: PathBuf,
    pub migrator: PathBuf,
    pub managed_marker: PathBuf,
    pub health_timeout: Duration,
}

impl UpgradePaths {
    pub fn for_root(root: PathBuf) -> Self {
        Self {
            staging_dir: root.join("staging"),
            ready_file: root.join("run/ready.json"),
            installed_release: root.join("install-meta/release.json"),
            tls_certificate: root.join("tls/server.crt"),
            database: root.join("device.db"),
            sql_root: PathBuf::from("/opt/usb-control/db"),
            migrator: PathBuf::from("/opt/usb-control/bin/usb-control-db-migrate"),
            managed_marker: root.join("run/managed"),
            health_timeout: Duration::ZERO,
            root,
        }
    }

    pub fn production(root: PathBuf) -> Self {
        let mut paths = Self::for_root(root);
        paths.ready_file = PathBuf::from("/run/usb-control/ready.json");
        paths.installed_release = PathBuf::from("/opt/usb-control/install-meta/release.json");
        paths.tls_certificate = PathBuf::from("/etc/usb-control/tls/server.crt");
        paths.database = PathBuf::from("/var/lib/usb-control/device.db");
        paths.managed_marker = PathBuf::from("/run/usb-control-updater/managed");
        paths.health_timeout = Duration::from_secs(30);
        paths
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeExecutionReport {
    pub post_commit_warning: Option<String>,
}

pub trait Clock {
    fn now(&self) -> Result<i64, UpdaterError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Result<i64, UpdaterError> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| UpdaterError::TaskInvalid("系统时间早于 UNIX epoch".into()))?
            .as_secs();
        i64::try_from(seconds).map_err(|_| UpdaterError::TaskInvalid("系统时间超出范围".into()))
    }
}

#[derive(Debug, Clone)]
pub struct RevalidatedPackage {
    pub manifest: UpgradeManifest,
    pub candidate_deb: PathBuf,
    pub target_release: InstalledRelease,
}

pub trait PackageRevalidator {
    fn revalidate(
        &self,
        paths: &UpgradePaths,
        task: &UpgradeTask,
        database_state: &UpgradeDatabaseState,
    ) -> Result<RevalidatedPackage, UpdaterError>;
}

pub struct SharedPackageRevalidator {
    verify_key_dir: PathBuf,
    installed_release: PathBuf,
    active_key_id: PathBuf,
    max_package_size: u64,
    deb_inspector: Arc<dyn DebInspector>,
}

impl SharedPackageRevalidator {
    pub fn new(
        verify_key_dir: PathBuf,
        installed_release: PathBuf,
        active_key_id: PathBuf,
    ) -> Self {
        Self::with_deb_inspector(
            verify_key_dir,
            installed_release,
            active_key_id,
            128 * 1024 * 1024,
            Arc::new(DpkgDebInspector::default()),
        )
    }

    pub fn with_deb_inspector(
        verify_key_dir: PathBuf,
        installed_release: PathBuf,
        active_key_id: PathBuf,
        max_package_size: u64,
        deb_inspector: Arc<dyn DebInspector>,
    ) -> Self {
        Self {
            verify_key_dir,
            installed_release,
            active_key_id,
            max_package_size,
            deb_inspector,
        }
    }

    pub fn production() -> Self {
        Self::new(
            PathBuf::from("/etc/usb-control/keys"),
            PathBuf::from("/opt/usb-control/install-meta/release.json"),
            PathBuf::from("/etc/usb-control/keys/upgrade_verify.id"),
        )
    }
}

impl PackageRevalidator for SharedPackageRevalidator {
    fn revalidate(
        &self,
        paths: &UpgradePaths,
        task: &UpgradeTask,
        database_state: &UpgradeDatabaseState,
    ) -> Result<RevalidatedPackage, UpdaterError> {
        let installed = read_installed_release(&self.installed_release)?;
        let active_key_text = fs::read_to_string(&self.active_key_id)?;
        let active_key_id = strict_line(&active_key_text)?;
        let database_version = SystemVersion::parse(&database_state.system_version)?;
        if database_version != task.source_version
            || installed.version != task.source_version
            || installed.supported_schema_min > database_state.schema_version
            || installed.supported_schema_max < database_state.schema_version
            || installed.upgrade_signing_key_id != active_key_id
        {
            return Err(UpdaterError::TaskInvalid(
                "数据库源状态、信任根和安装元数据不一致".into(),
            ));
        }
        let package = PackageStager::new(paths.root.clone(), self.max_package_size)
            .reopen(&task.upgrade_id, &task.package_sha256)?;
        let verified =
            PackageVerifier::new(self.verify_key_dir.clone(), self.deb_inspector.clone()).verify(
                package,
                &VerificationContext {
                    current_version: database_version,
                    current_schema: database_state.schema_version,
                    supported_schema_max: installed.supported_schema_max,
                    protocol_version: 1,
                    client_target_version: task.target_version.to_string(),
                    client_sha256: task.package_sha256.clone(),
                },
            )?;
        if verified.staged.manifest.package_version != task.target_version {
            return Err(UpdaterError::TaskInvalid(
                "二次验证目标版本与任务不一致".into(),
            ));
        }
        let manifest = verified.staged.manifest;
        let target_release = InstalledRelease {
            format_version: 1,
            product: manifest.product.clone(),
            version: manifest.package_version,
            architecture: manifest.architecture.clone(),
            supported_schema_min: verified.deb_metadata.supported_schema_min,
            supported_schema_max: verified.deb_metadata.supported_schema_max,
            tls_cert_sha256: verified.deb_metadata.tls_cert_sha256,
            upgrade_signing_key_id: verified.deb_metadata.upgrade_signing_key_id,
        };
        Ok(RevalidatedPackage {
            manifest,
            candidate_deb: verified.staged.deb_path,
            target_release,
        })
    }
}

fn strict_line(value: &str) -> Result<&str, UpdaterError> {
    let line = value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix('\n'))
        .unwrap_or(value);
    if line.is_empty()
        || line.len() > 64
        || line.contains(['\r', '\n'])
        || !line
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(UpdaterError::TaskInvalid("活动升级公钥标识非法".into()));
    }
    Ok(line)
}

pub struct UpgradeExecutor<R, V, C> {
    paths: UpgradePaths,
    runner: R,
    revalidator: V,
    database: Arc<dyn UpgradeDatabase>,
    clock: C,
}

struct LockedUpgradeStores<'a> {
    tasks: &'a UpgradeTaskStore,
    results: &'a UpgradeResultStore,
    lock: &'a UpgradeStateLock,
}

impl<R, V, C> UpgradeExecutor<R, V, C> {
    pub fn new(
        paths: UpgradePaths,
        runner: R,
        revalidator: V,
        database: Arc<dyn UpgradeDatabase>,
        clock: C,
    ) -> Self {
        Self {
            paths,
            runner,
            revalidator,
            database,
            clock,
        }
    }
}

impl<R: CommandRunner, V: PackageRevalidator, C: Clock> UpgradeExecutor<R, V, C> {
    pub fn execute(&self, upgrade_id: &str) -> Result<UpgradeExecutionReport, UpdaterError> {
        let lock = UpgradeStateLock::acquire(&self.paths.root)?;
        let tasks = UpgradeTaskStore::new(self.paths.root.clone())?;
        let results = UpgradeResultStore::new(self.paths.root.clone())?;
        let stores = LockedUpgradeStores {
            tasks: &tasks,
            results: &results,
            lock: &lock,
        };
        let task = tasks
            .current()?
            .ok_or_else(|| UpdaterError::TaskInvalid("current.json 不存在".into()))?;
        validate_task(&task, upgrade_id)?;
        let mut last_timestamp = task.updated_at;
        let database_state = match self.database.read_state() {
            Ok(state) => state,
            Err(error) => {
                return Err(self.finish_failed(
                    &stores,
                    &task,
                    "revalidating",
                    error,
                    &mut last_timestamp,
                ));
            }
        };
        let database_version = match SystemVersion::parse(&database_state.system_version) {
            Ok(version) if version == task.source_version => version,
            Ok(_) => {
                return Err(self.finish_failed(
                    &stores,
                    &task,
                    "revalidating",
                    UpdaterError::TaskInvalid("数据库业务版本与任务源版本不一致".into()),
                    &mut last_timestamp,
                ));
            }
            Err(error) => {
                return Err(self.finish_failed(
                    &stores,
                    &task,
                    "revalidating",
                    UpdaterError::Domain(error),
                    &mut last_timestamp,
                ));
            }
        };
        let verified = match self
            .revalidator
            .revalidate(&self.paths, &task, &database_state)
        {
            Ok(value) => value,
            Err(error) => {
                return Err(self.finish_failed(
                    &stores,
                    &task,
                    "revalidating",
                    error,
                    &mut last_timestamp,
                ));
            }
        };
        let target_release = verified.target_release;
        let manifest = verified.manifest;
        let candidate = verified.candidate_deb;
        let _managed_install = match ManagedInstallGuard::create(&self.paths.managed_marker) {
            Ok(guard) => guard,
            Err(error) => {
                return Err(self.finish_failed(
                    &stores,
                    &task,
                    "preparing_install",
                    error,
                    &mut last_timestamp,
                ));
            }
        };

        let install_result = (|| -> Result<(), UpdaterError> {
            transition(
                &tasks,
                &lock,
                upgrade_id,
                UpgradeStatus::Stopping,
                next_time(&self.clock, &mut last_timestamp)?,
            )?;
            run_command(
                &self.runner,
                "stopping",
                "systemctl",
                ["stop", "usb-control.service"],
                60,
            )?;
            transition(
                &tasks,
                &lock,
                upgrade_id,
                UpgradeStatus::Installing,
                next_time(&self.clock, &mut last_timestamp)?,
            )?;
            install_deb(&self.runner, &candidate)?;
            transition(
                &tasks,
                &lock,
                upgrade_id,
                UpgradeStatus::Migrating,
                next_time(&self.clock, &mut last_timestamp)?,
            )?;
            run_migration(
                &self.runner,
                &self.paths.migrator,
                &self.paths.database,
                &self.paths.sql_root,
            )?;
            transition(
                &tasks,
                &lock,
                upgrade_id,
                UpgradeStatus::Starting,
                next_time(&self.clock, &mut last_timestamp)?,
            )?;
            configure_and_reload(&self.runner)?;
            let restarts_before = read_restart_count(&self.runner, "starting")?;
            let start_attempt_at = next_time(&self.clock, &mut last_timestamp)?;
            run_command(
                &self.runner,
                "starting",
                "systemctl",
                ["start", "usb-control.service"],
                60,
            )?;
            transition(
                &tasks,
                &lock,
                upgrade_id,
                UpgradeStatus::HealthChecking,
                next_time(&self.clock, &mut last_timestamp)?,
            )?;
            check_health(
                &self.runner,
                &self.paths,
                &HealthExpectation {
                    release: target_release,
                    schema_version: manifest.schema_to,
                    start_attempt_at,
                    restarts_before,
                },
            )
        })();
        if let Err(error) = install_result {
            let stage = stage_of(&error);
            return Err(self.finish_failed(&stores, &task, &stage, error, &mut last_timestamp));
        }

        let committed_at = next_time(&self.clock, &mut last_timestamp)?;
        let mut warnings = Vec::new();
        if let Err(error) = self.database.compare_and_set_version(
            &database_version.to_string(),
            &manifest.package_version.to_string(),
            committed_at,
        ) {
            return Err(self.finish_failed(
                &stores,
                &task,
                "committing",
                error,
                &mut last_timestamp,
            ));
        }
        let finished_at = next_time_or_fallback(&self.clock, &mut last_timestamp);
        let committed = UpgradeResult {
            format_version: 1,
            upgrade_id: task.upgrade_id.clone(),
            status: UpgradeStatus::Committed,
            username: task.username.clone(),
            role: task.role,
            source_ip: task.source_ip.clone(),
            source_version: task.source_version,
            target_version: task.target_version,
            effective_version: task.target_version,
            failed_stage: None,
            original_error: None,
            finished_at,
        };
        if let Err(error) = results.write(&lock, &committed) {
            warnings.push(error.to_string());
        } else if let Err(error) =
            tasks.ensure_terminal(&lock, upgrade_id, UpgradeStatus::Committed, finished_at)
        {
            warnings.push(error.to_string());
        }
        if let Err(error) = remove_staging(&lock, &self.paths.staging_dir, upgrade_id) {
            warnings.push(error.to_string());
        }
        Ok(UpgradeExecutionReport {
            post_commit_warning: (!warnings.is_empty()).then(|| warnings.join("；")),
        })
    }

    fn finish_failed(
        &self,
        stores: &LockedUpgradeStores<'_>,
        task: &UpgradeTask,
        failed_stage: &str,
        original: UpdaterError,
        last_timestamp: &mut i64,
    ) -> UpdaterError {
        let finished_at = next_time_or_fallback(&self.clock, last_timestamp);
        let original_text = original.to_string();
        let result = UpgradeResult {
            format_version: 1,
            upgrade_id: task.upgrade_id.clone(),
            status: UpgradeStatus::Failed,
            username: task.username.clone(),
            role: task.role,
            source_ip: task.source_ip.clone(),
            source_version: task.source_version,
            target_version: task.target_version,
            effective_version: task.source_version,
            failed_stage: Some(failed_stage.into()),
            original_error: Some(original_text.clone()),
            finished_at,
        };
        let mut persistence_errors = Vec::new();
        if let Err(error) = stores.results.write(stores.lock, &result) {
            persistence_errors.push(error.to_string());
        }
        if let Err(error) = stores.tasks.ensure_terminal(
            stores.lock,
            &task.upgrade_id,
            UpgradeStatus::Failed,
            finished_at,
        ) {
            persistence_errors.push(error.to_string());
        }
        if persistence_errors.is_empty() {
            original
        } else {
            UpdaterError::TaskInvalid(format!(
                "{original_text}；失败终态持久化错误: {}",
                persistence_errors.join("；")
            ))
        }
    }
}

fn next_time(clock: &dyn Clock, last: &mut i64) -> Result<i64, UpdaterError> {
    *last = (*last).max(clock.now()?);
    Ok(*last)
}

fn next_time_or_fallback(clock: &dyn Clock, last: &mut i64) -> i64 {
    if let Ok(now) = clock.now() {
        *last = (*last).max(now);
    }
    *last
}

fn transition(
    store: &UpgradeTaskStore,
    lock: &UpgradeStateLock,
    upgrade_id: &str,
    status: UpgradeStatus,
    now: i64,
) -> Result<(), UpdaterError> {
    store.transition(lock, upgrade_id, status, now)?;
    Ok(())
}

fn validate_task(task: &UpgradeTask, upgrade_id: &str) -> Result<(), UpdaterError> {
    if task.format_version != 1
        || task.upgrade_id != upgrade_id
        || task.status != UpgradeStatus::Accepted
    {
        return Err(UpdaterError::TaskInvalid(
            "任务格式、标识或状态不允许执行".into(),
        ));
    }
    Ok(())
}

fn stage_of(error: &UpdaterError) -> String {
    match error {
        UpdaterError::CommandSpawn { stage, .. }
        | UpdaterError::CommandTimeout { stage, .. }
        | UpdaterError::CommandFailed { stage, .. } => stage.clone(),
        UpdaterError::MigrationFailed(_) => "migrating".into(),
        UpdaterError::HealthFailed(_) => "health_checking".into(),
        _ => "updater".into(),
    }
}

pub(crate) fn run_command(
    runner: &dyn CommandRunner,
    stage: &str,
    program: &str,
    args: impl IntoIterator<Item = impl Into<OsString>>,
    timeout_secs: u64,
) -> Result<(), UpdaterError> {
    runner.run(&command(
        stage,
        program,
        args,
        Duration::from_secs(timeout_secs),
    ))?;
    Ok(())
}

fn install_deb(runner: &dyn CommandRunner, deb: &Path) -> Result<(), UpdaterError> {
    run_command(
        runner,
        "installing",
        "dpkg",
        [OsString::from("--unpack"), deb.as_os_str().to_os_string()],
        300,
    )
}

fn configure_and_reload(runner: &dyn CommandRunner) -> Result<(), UpdaterError> {
    run_command(
        runner,
        "starting",
        "dpkg",
        ["--configure", "usb-control"],
        300,
    )?;
    run_command(runner, "starting", "systemctl", ["daemon-reload"], 60)
}

fn remove_staging(
    _lock: &UpgradeStateLock,
    parent: &Path,
    upgrade_id: &str,
) -> Result<(), UpdaterError> {
    let path = parent.join(upgrade_id);
    if path.exists() {
        fs::remove_dir_all(&path)?;
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
