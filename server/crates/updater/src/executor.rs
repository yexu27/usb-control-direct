//! 升级命令边界、唯一事务顺序和有效发布提交点。

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use system_upgrade::{
    ActiveCommitError, ActiveRelease, DebInspector, DpkgDebInspector, PackageStager,
    PackageVerifier, ReleaseStateStore, SystemVersion, UpgradeManifest, UpgradeResult,
    UpgradeStatus, UpgradeTask, UpgradeTaskStore, VerificationContext,
};
use wait_timeout::ChildExt;

use crate::health::{check_health, read_restart_count, HealthExpectation};
use crate::migration::run_migration;
use crate::rollback::{
    read_and_validate_lkg, rollback, FileLkgRepository, LastKnownGoodRelease, LkgRepository,
};
use crate::UpdaterError;

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
        let (stdout, stdout_truncated) =
            join_reader(stdout_reader, &command.stage, &program, "stdout")?;
        let (stderr, stderr_truncated) =
            join_reader(stderr_reader, &command.stage, &program, "stderr")?;
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
            break;
        }
        let remaining = MAX_COMMAND_OUTPUT.saturating_sub(kept.len());
        let keep = remaining.min(read);
        kept.extend_from_slice(&buffer[..keep]);
        truncated |= keep < read;
    }
    Ok((kept, truncated))
}

fn join_reader(
    reader: std::thread::JoinHandle<std::io::Result<(Vec<u8>, bool)>>,
    stage: &str,
    program: &str,
    stream: &str,
) -> Result<(Vec<u8>, bool), UpdaterError> {
    reader
        .join()
        .map_err(|_| UpdaterError::CommandSpawn {
            stage: stage.to_string(),
            program: program.to_string(),
            source: std::io::Error::other(format!("{stream} reader thread panic")),
        })?
        .map_err(|source| UpdaterError::CommandSpawn {
            stage: stage.to_string(),
            program: program.to_string(),
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
        stage: stage.to_string(),
        program: program.into(),
        args: args.into_iter().map(Into::into).collect(),
        timeout,
    }
}

#[derive(Debug, Clone)]
pub struct UpgradePaths {
    pub root: PathBuf,
    pub current_task: PathBuf,
    pub history_dir: PathBuf,
    pub staging_dir: PathBuf,
    pub rollback_dir: PathBuf,
    pub last_known_good_deb: PathBuf,
    pub last_known_good_metadata: PathBuf,
    pub next_last_known_good_deb: PathBuf,
    pub previous_deb: PathBuf,
    pub active_release: PathBuf,
    pub managed_marker: PathBuf,
    pub ready_file: PathBuf,
    pub install_version_file: PathBuf,
    pub tls_certificate: PathBuf,
    pub database: PathBuf,
    pub sql_root: PathBuf,
    pub migrator: PathBuf,
    pub health_timeout: Duration,
}

impl UpgradePaths {
    pub fn for_root(root: PathBuf) -> Self {
        let rollback_dir = root.join("rollback");
        Self {
            current_task: root.join("current.json"),
            history_dir: root.join("history"),
            staging_dir: root.join("staging"),
            last_known_good_deb: rollback_dir.join("last-known-good.deb"),
            last_known_good_metadata: rollback_dir.join("last-known-good.json"),
            next_last_known_good_deb: rollback_dir.join("next-last-known-good.deb"),
            previous_deb: rollback_dir.join("previous.deb"),
            rollback_dir,
            active_release: root.join("active-release.json"),
            managed_marker: root.join("run/upgrade-managed"),
            ready_file: root.join("run/ready.json"),
            install_version_file: root.join("install-meta/VERSION"),
            tls_certificate: root.join("tls/server.crt"),
            database: root.join("device.db"),
            sql_root: PathBuf::from("/opt/usb-control/db"),
            migrator: PathBuf::from("/opt/usb-control/bin/usb-control-db-migrate"),
            health_timeout: Duration::ZERO,
            root,
        }
    }

    pub fn production(root: PathBuf) -> Self {
        let mut paths = Self::for_root(root);
        paths.managed_marker = PathBuf::from("/run/usb-control/upgrade-managed");
        paths.ready_file = PathBuf::from("/run/usb-control/ready.json");
        paths.install_version_file = PathBuf::from("/opt/usb-control/install-meta/VERSION");
        paths.tls_certificate = PathBuf::from("/etc/usb-control/tls/server.crt");
        paths.database = PathBuf::from("/var/lib/usb-control/device.db");
        paths.health_timeout = Duration::from_secs(30);
        paths
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionDisposition {
    Committed,
    CommittedResultPending,
    RolledBack,
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
    pub lkg: LastKnownGoodRelease,
}

pub trait PackageRevalidator {
    fn revalidate(
        &self,
        paths: &UpgradePaths,
        task: &UpgradeTask,
    ) -> Result<RevalidatedPackage, UpdaterError>;
}

pub trait ActiveReleasePublisher {
    fn commit(&self, root: &Path, release: &ActiveRelease) -> Result<(), ActiveCommitError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SharedActiveReleasePublisher;

impl ActiveReleasePublisher for SharedActiveReleasePublisher {
    fn commit(&self, root: &Path, release: &ActiveRelease) -> Result<(), ActiveCommitError> {
        ReleaseStateStore::new(root.to_path_buf())
            .map_err(ActiveCommitError::BeforeRename)?
            .commit_active_release(release)
    }
}

pub struct SharedPackageRevalidator {
    verify_key_dir: PathBuf,
    installed_release: PathBuf,
    active_key_id: PathBuf,
    max_package_size: u64,
    deb_inspector: Arc<dyn DebInspector>,
}

impl SharedPackageRevalidator {
    /// 使用明确的信任文件路径创建共享包重验证器。
    ///
    /// 参数:
    /// - `verify_key_dir`: 升级验签公钥目录。
    /// - `installed_release`: 当前安装发布元数据路径。
    /// - `active_key_id`: 当前活动升级公钥标识路径。
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

    /// 使用明确的路径、包大小上限和 DEB 检查器创建共享重验证器。
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InstalledRelease {
    format_version: u32,
    product: String,
    version: SystemVersion,
    architecture: String,
    supported_schema_min: u32,
    supported_schema_max: u32,
    tls_cert_sha256: String,
    upgrade_signing_key_id: String,
}

impl PackageRevalidator for SharedPackageRevalidator {
    fn revalidate(
        &self,
        paths: &UpgradePaths,
        task: &UpgradeTask,
    ) -> Result<RevalidatedPackage, UpdaterError> {
        let release_store = ReleaseStateStore::new(paths.root.clone())?;
        let active = release_store
            .active_release()?
            .ok_or_else(|| UpdaterError::TaskInvalid("active-release.json 不存在".into()))?;
        let installed: InstalledRelease =
            serde_json::from_slice(&fs::read(&self.installed_release)?)?;
        let active_key_text = fs::read_to_string(&self.active_key_id)?;
        let active_key_id = strict_line(&active_key_text)?;
        let lkg = read_and_validate_lkg(paths)?;
        if active.version != task.source_version
            || active.schema_version != lkg.schema_version
            || active.version != lkg.version
            || active.deb_sha256 != lkg.deb_sha256
            || installed.format_version != 1
            || installed.product != "usb-control"
            || installed.architecture != "arm64"
            || installed.version != active.version
            || installed.supported_schema_min > active.schema_version
            || installed.supported_schema_max < active.schema_version
            || installed.supported_schema_min > installed.supported_schema_max
            || installed.upgrade_signing_key_id != active_key_id
            || !is_lower_hex_64(&installed.tls_cert_sha256)
            || installed.tls_cert_sha256 != lkg.tls_cert_sha256
        {
            return Err(UpdaterError::TaskInvalid(
                "当前有效发布、LKG、信任根和安装元数据不一致".into(),
            ));
        }

        let package = PackageStager::new(paths.root.clone(), self.max_package_size)
            .reopen(&task.upgrade_id, &task.package_sha256)?;
        let verifier =
            PackageVerifier::new(self.verify_key_dir.clone(), self.deb_inspector.clone());
        let context = VerificationContext {
            current_version: active.version,
            current_schema: active.schema_version,
            supported_schema_max: installed.supported_schema_max,
            protocol_version: 1,
            client_target_version: task.target_version.to_string(),
            client_sha256: task.package_sha256.clone(),
        };
        let verified = verifier.verify(package, &context)?;
        if verified.staged.manifest.package_version != task.target_version {
            return Err(UpdaterError::TaskInvalid(
                "二次验证目标版本与任务不一致".into(),
            ));
        }
        Ok(RevalidatedPackage {
            manifest: verified.staged.manifest,
            candidate_deb: verified.staged.deb_path,
            lkg,
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

pub(crate) fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub struct UpgradeExecutor<R, V, C, F = FileLkgRepository, P = SharedActiveReleasePublisher> {
    pub(crate) paths: UpgradePaths,
    pub(crate) runner: R,
    revalidator: V,
    clock: C,
    files: F,
    active_release_publisher: P,
}

impl<R: CommandRunner, V: PackageRevalidator, C: Clock>
    UpgradeExecutor<R, V, C, FileLkgRepository>
{
    pub fn new(paths: UpgradePaths, runner: R, revalidator: V, clock: C) -> Self {
        Self {
            paths,
            runner,
            revalidator,
            clock,
            files: FileLkgRepository,
            active_release_publisher: SharedActiveReleasePublisher,
        }
    }
}

impl<R: CommandRunner, V: PackageRevalidator, C: Clock, F: LkgRepository>
    UpgradeExecutor<R, V, C, F>
{
    pub fn with_repository(
        paths: UpgradePaths,
        runner: R,
        revalidator: V,
        clock: C,
        files: F,
    ) -> Self {
        Self {
            paths,
            runner,
            revalidator,
            clock,
            files,
            active_release_publisher: SharedActiveReleasePublisher,
        }
    }
}

impl<R, V, C, F, P> UpgradeExecutor<R, V, C, F, P> {
    pub fn with_components(
        paths: UpgradePaths,
        runner: R,
        revalidator: V,
        clock: C,
        files: F,
        active_release_publisher: P,
    ) -> Self {
        Self {
            paths,
            runner,
            revalidator,
            clock,
            files,
            active_release_publisher,
        }
    }
}

impl<
        R: CommandRunner,
        V: PackageRevalidator,
        C: Clock,
        F: LkgRepository,
        P: ActiveReleasePublisher,
    > UpgradeExecutor<R, V, C, F, P>
{
    pub fn execute(&self, upgrade_id: &str) -> Result<ExecutionDisposition, UpdaterError> {
        let _lock = FileLock::acquire(&self.paths.root.join("lock"))?;
        let store = UpgradeTaskStore::new(self.paths.root.clone())?;
        let release_store = ReleaseStateStore::new(self.paths.root.clone())?;
        let task = store
            .current()?
            .ok_or_else(|| UpdaterError::TaskInvalid("current.json 不存在".into()))?;
        validate_task(&task, upgrade_id)?;
        let mut last_timestamp = task.updated_at;
        let verified = match self.revalidator.revalidate(&self.paths, &task) {
            Ok(verified) => verified,
            Err(error) => {
                return self.finish_before_stop_failure(
                    &store,
                    &release_store,
                    &task,
                    PreStopFailure {
                        failed_stage: "revalidating",
                        original: error,
                        prepared_transaction: false,
                    },
                    &mut last_timestamp,
                );
            }
        };
        let manifest = verified.manifest;
        let candidate = verified.candidate_deb;
        let candidate_sha = manifest.deb_sha256.clone();
        let previous_lkg = verified.lkg;
        if let Err(error) = self.files.prepare(&self.paths, &candidate, &candidate_sha) {
            return self.finish_before_stop_failure(
                &store,
                &release_store,
                &task,
                PreStopFailure {
                    failed_stage: "preparing",
                    original: error,
                    prepared_transaction: false,
                },
                &mut last_timestamp,
            );
        }

        let mut service_stopped = false;
        let install_result = (|| {
            let stopping_at = match next_time(&self.clock, &mut last_timestamp) {
                Ok(timestamp) => timestamp,
                Err(error) => {
                    return Err(PreStopBoundary::Finalizable(error));
                }
            };
            if let Err(error) = transition(&store, upgrade_id, UpgradeStatus::Stopping, stopping_at)
            {
                return Err(PreStopBoundary::Persistence(error));
            }
            service_stopped = true;
            run_command(
                &self.runner,
                "stopping",
                "systemctl",
                ["stop", "usb-control.service"],
                60,
            )?;
            let installing_at = next_time(&self.clock, &mut last_timestamp)?;
            transition(&store, upgrade_id, UpgradeStatus::Installing, installing_at)?;
            install_deb(&self.runner, "installing", &candidate)?;
            let migrating_at = next_time(&self.clock, &mut last_timestamp)?;
            transition(&store, upgrade_id, UpgradeStatus::Migrating, migrating_at)?;
            run_migration(
                &self.runner,
                &self.paths.migrator,
                &self.paths.database,
                &self.paths.sql_root,
            )?;
            let starting_at = next_time(&self.clock, &mut last_timestamp)?;
            transition(&store, upgrade_id, UpgradeStatus::Starting, starting_at)?;
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
            let health_checking_at = next_time(&self.clock, &mut last_timestamp)?;
            transition(
                &store,
                upgrade_id,
                UpgradeStatus::HealthChecking,
                health_checking_at,
            )?;
            let health = HealthExpectation {
                version: manifest.package_version,
                schema_version: manifest.schema_to,
                tls_cert_sha256: manifest.tls_cert_sha256.clone(),
                start_attempt_at,
                restarts_before,
            };
            check_health(&self.runner, &self.paths, &health)?;
            self.files.promote(&self.paths, &manifest, &candidate_sha)?;
            let committed_at = next_time(&self.clock, &mut last_timestamp)?;
            let active = ActiveRelease {
                format_version: 1,
                upgrade_id: upgrade_id.to_string(),
                version: manifest.package_version,
                deb_sha256: candidate_sha.clone(),
                schema_version: manifest.schema_to,
                committed_at,
            };
            let active_release_sync_pending = match self
                .active_release_publisher
                .commit(&self.paths.root, &active)
            {
                Ok(()) => false,
                Err(ActiveCommitError::BeforeRename(error)) => {
                    return Err(UpdaterError::Domain(error).into());
                }
                Err(ActiveCommitError::AfterRename(_)) => true,
            };
            Ok::<bool, PreStopBoundary>(active_release_sync_pending)
        })();

        let active_release_sync_pending = match install_result {
            Ok(sync_pending) => sync_pending,
            Err(PreStopBoundary::Finalizable(original)) => {
                return self.finish_before_stop_failure(
                    &store,
                    &release_store,
                    &task,
                    PreStopFailure {
                        failed_stage: "preparing",
                        original,
                        prepared_transaction: true,
                    },
                    &mut last_timestamp,
                );
            }
            Err(PreStopBoundary::Persistence(original)) => {
                return match self.files.abort_prepared(&self.paths) {
                    Ok(()) => Err(original),
                    Err(cleanup) => Err(combine_errors(
                        "Stopping 状态持久化失败",
                        &original,
                        "准备事务清理失败",
                        &cleanup,
                    )),
                };
            }
            Err(PreStopBoundary::Execution(original)) => {
                if !service_stopped {
                    return Err(original);
                }
                let rolling_back_at = next_time_or_fallback(&self.clock, &mut last_timestamp);
                let _ = transition(
                    &store,
                    upgrade_id,
                    UpgradeStatus::RollingBack,
                    rolling_back_at,
                );
                let rollback_source = if self.paths.previous_deb.is_file() {
                    self.paths.previous_deb.clone()
                } else {
                    self.paths.last_known_good_deb.clone()
                };
                let rollback_result = rollback(
                    &self.runner,
                    &self.paths,
                    &rollback_source,
                    &previous_lkg,
                    &self.clock,
                    last_timestamp,
                )
                .and_then(|rollback_started_at| {
                    last_timestamp = last_timestamp.max(rollback_started_at);
                    self.files.restore(&self.paths, &previous_lkg)
                })
                .and_then(|()| self.files.cleanup_rollback(&self.paths));
                match rollback_result {
                    Ok(()) => {
                        let rolled_back_at =
                            next_time_or_fallback(&self.clock, &mut last_timestamp);
                        transition(
                            &store,
                            upgrade_id,
                            UpgradeStatus::RolledBack,
                            rolled_back_at,
                        )?;
                        let finished_at = next_time_or_fallback(&self.clock, &mut last_timestamp);
                        write_result(
                            &release_store,
                            &task,
                            ResultOutcome {
                                status: UpgradeStatus::RolledBack,
                                effective_version: task.source_version,
                                failed_stage: Some(stage_of(&original)),
                                original_error: Some(original.to_string()),
                                rollback_error: None,
                                finished_at,
                            },
                        )?;
                        return Ok(ExecutionDisposition::RolledBack);
                    }
                    Err(rollback_error) => {
                        let combined = UpdaterError::RollbackFailed {
                            original: original.to_string(),
                            rollback: rollback_error.to_string(),
                        };
                        let rollback_failed_at =
                            next_time_or_fallback(&self.clock, &mut last_timestamp);
                        let _ = transition(
                            &store,
                            upgrade_id,
                            UpgradeStatus::RollbackFailed,
                            rollback_failed_at,
                        );
                        let finished_at = next_time_or_fallback(&self.clock, &mut last_timestamp);
                        let _ = write_result(
                            &release_store,
                            &task,
                            ResultOutcome {
                                status: UpgradeStatus::RollbackFailed,
                                effective_version: task.source_version,
                                failed_stage: Some(stage_of(&original)),
                                original_error: Some(original.to_string()),
                                rollback_error: Some(rollback_error.to_string()),
                                finished_at,
                            },
                        );
                        let _ = self.files.cleanup_rollback(&self.paths);
                        return Err(combined);
                    }
                }
            }
        };
        let committed_at = last_timestamp;
        let committed_state_at =
            next_time_or_fallback(&self.clock, &mut last_timestamp).max(committed_at);
        let mut committed_state = None;
        for _ in 0..3 {
            match transition(
                &store,
                upgrade_id,
                UpgradeStatus::Committed,
                committed_state_at,
            ) {
                Ok(()) => {
                    committed_state = Some(Ok(()));
                    break;
                }
                Err(error) => committed_state = Some(Err(error)),
            }
        }
        let committed = committed_state
            .expect("commit retry loop has at least one iteration")
            .and_then(|_| {
                let finished_at =
                    next_time_or_fallback(&self.clock, &mut last_timestamp).max(committed_at);
                write_result(
                    &release_store,
                    &task,
                    ResultOutcome {
                        status: UpgradeStatus::Committed,
                        effective_version: task.target_version,
                        failed_stage: None,
                        original_error: None,
                        rollback_error: None,
                        finished_at,
                    },
                )
            });
        if committed.is_err() {
            let _ = self.files.cleanup_committed(&self.paths);
            return Ok(ExecutionDisposition::CommittedResultPending);
        }
        let cleanup_pending = self.files.cleanup_committed(&self.paths).is_err();
        if active_release_sync_pending || cleanup_pending {
            Ok(ExecutionDisposition::CommittedResultPending)
        } else {
            Ok(ExecutionDisposition::Committed)
        }
    }

    fn finish_before_stop_failure(
        &self,
        store: &UpgradeTaskStore,
        releases: &ReleaseStateStore,
        task: &UpgradeTask,
        failure: PreStopFailure<'_>,
        last_timestamp: &mut i64,
    ) -> Result<ExecutionDisposition, UpdaterError> {
        let rolling_back_at = next_time_or_increment(&self.clock, last_timestamp)?;
        transition(
            store,
            &task.upgrade_id,
            UpgradeStatus::RollingBack,
            rolling_back_at,
        )
        .map_err(|state| {
            combine_errors(
                "停服前失败终态化",
                &failure.original,
                "RollingBack 状态持久化",
                &state,
            )
        })?;

        let cleanup_error = if failure.prepared_transaction {
            self.files.abort_prepared(&self.paths).err()
        } else {
            None
        };
        let rolled_back_at = next_time_or_increment(&self.clock, last_timestamp)?;
        transition(
            store,
            &task.upgrade_id,
            UpgradeStatus::RolledBack,
            rolled_back_at,
        )
        .map_err(|state| {
            combine_errors(
                "停服前失败终态化",
                &failure.original,
                "RolledBack 状态持久化",
                &state,
            )
        })?;
        let finished_at = next_time_or_increment(&self.clock, last_timestamp)?;
        write_result(
            releases,
            task,
            ResultOutcome {
                status: UpgradeStatus::RolledBack,
                effective_version: task.source_version,
                failed_stage: Some(failure.failed_stage.to_string()),
                original_error: Some(failure.original.to_string()),
                rollback_error: cleanup_error.map(|error| error.to_string()),
                finished_at,
            },
        )?;
        Ok(ExecutionDisposition::RolledBack)
    }
}

struct PreStopFailure<'a> {
    failed_stage: &'a str,
    original: UpdaterError,
    prepared_transaction: bool,
}

enum PreStopBoundary {
    Finalizable(UpdaterError),
    Persistence(UpdaterError),
    Execution(UpdaterError),
}

impl From<UpdaterError> for PreStopBoundary {
    fn from(error: UpdaterError) -> Self {
        Self::Execution(error)
    }
}

fn next_time(clock: &dyn Clock, last_timestamp: &mut i64) -> Result<i64, UpdaterError> {
    let now = clock.now()?.max(*last_timestamp);
    *last_timestamp = now;
    Ok(now)
}

fn next_time_or_fallback(clock: &dyn Clock, last_timestamp: &mut i64) -> i64 {
    if let Ok(now) = clock.now() {
        *last_timestamp = (*last_timestamp).max(now);
    }
    *last_timestamp
}

fn next_time_or_increment(
    clock: &dyn Clock,
    last_timestamp: &mut i64,
) -> Result<i64, UpdaterError> {
    match clock.now() {
        Ok(now) => {
            *last_timestamp = (*last_timestamp).max(now);
        }
        Err(_) => {
            *last_timestamp = last_timestamp.checked_add(1).ok_or_else(|| {
                UpdaterError::TaskInvalid("升级任务时间戳溢出，无法持久化终态".into())
            })?;
        }
    }
    Ok(*last_timestamp)
}

fn combine_errors(
    first_context: &str,
    first: &UpdaterError,
    second_context: &str,
    second: &UpdaterError,
) -> UpdaterError {
    UpdaterError::TaskInvalid(format!(
        "{first_context}: {first}；{second_context}: {second}"
    ))
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

pub(crate) fn install_deb(
    runner: &dyn CommandRunner,
    stage: &str,
    deb: &Path,
) -> Result<(), UpdaterError> {
    run_command(
        runner,
        stage,
        "dpkg",
        [OsString::from("--unpack"), deb.as_os_str().to_os_string()],
        300,
    )
}

pub(crate) fn configure_and_reload(runner: &dyn CommandRunner) -> Result<(), UpdaterError> {
    run_command(
        runner,
        "starting",
        "dpkg",
        ["--configure", "usb-control"],
        300,
    )?;
    run_command(runner, "starting", "systemctl", ["daemon-reload"], 60)
}

fn transition(
    store: &UpgradeTaskStore,
    upgrade_id: &str,
    status: UpgradeStatus,
    now: i64,
) -> Result<(), UpdaterError> {
    store.transition(upgrade_id, status, now)?;
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

struct ResultOutcome {
    status: UpgradeStatus,
    effective_version: SystemVersion,
    failed_stage: Option<String>,
    original_error: Option<String>,
    rollback_error: Option<String>,
    finished_at: i64,
}

fn write_result(
    store: &ReleaseStateStore,
    task: &UpgradeTask,
    outcome: ResultOutcome,
) -> Result<(), UpdaterError> {
    let result = UpgradeResult {
        format_version: 1,
        upgrade_id: task.upgrade_id.clone(),
        status: outcome.status,
        username: task.username.clone(),
        role: task.role,
        source_ip: task.source_ip.clone(),
        source_version: task.source_version,
        target_version: task.target_version,
        effective_version: outcome.effective_version,
        failed_stage: outcome.failed_stage,
        original_error: outcome.original_error,
        rollback_error: outcome.rollback_error,
        finished_at: outcome.finished_at,
    };
    let mut last = None;
    for _ in 0..3 {
        match store.write_result(&result) {
            Ok(()) => return Ok(()),
            Err(error) => last = Some(UpdaterError::Domain(error)),
        }
    }
    Err(last.expect("result retry loop has at least one iteration"))
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

fn create_private_dir_all(path: &Path) -> Result<(), UpdaterError> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

struct FileLock(File);

impl FileLock {
    fn acquire(path: &Path) -> Result<Self, UpdaterError> {
        if let Some(parent) = path.parent() {
            create_private_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(path)?;
        // SAFETY: flock only observes the valid descriptor owned by `file`.
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if result != 0 {
            return Err(UpdaterError::TaskInvalid("已有 updater 正在执行".into()));
        }
        Ok(Self(file))
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // SAFETY: descriptor remains valid for the lifetime of this guard.
        let _ = unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) };
    }
}
