//! 在线安装标记、直接 DEB 安装收尾和正式 CLI 命令模型。

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use system_upgrade::{read_installed_release, ActiveRelease, ActiveReleaseStore, UpgradeStateLock};

use crate::executor::run_command;
use crate::health::{check_health, read_restart_count, HealthExpectation};
use crate::migration::run_migration;
use crate::{Clock, CommandRunner, UpdaterError, UpgradePaths};

/// 在线升级期间供 DEB 维护脚本识别 updater 管理安装的短期标记。
pub struct ManagedInstallGuard {
    path: PathBuf,
}

impl ManagedInstallGuard {
    pub fn create(path: &Path) -> Result<Self, UpdaterError> {
        let parent = path
            .parent()
            .ok_or_else(|| UpdaterError::TaskInvalid("managed marker 缺少父目录".into()))?;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        writeln!(file, "{}", std::process::id())?;
        file.sync_all()?;
        File::open(parent)?.sync_all()?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl Drop for ManagedInstallGuard {
    fn drop(&mut self) {
        if fs::remove_file(&self.path).is_ok() {
            if let Some(parent) = self.path.parent() {
                let _ = File::open(parent).and_then(|directory| directory.sync_all());
            }
        }
    }
}

/// 直接安装或手动 DEB 升级后的固定收尾编排。
pub struct InstallFinalizer<R, C> {
    paths: UpgradePaths,
    runner: R,
    clock: C,
}

impl<R, C> InstallFinalizer<R, C> {
    pub fn new(paths: UpgradePaths, runner: R, clock: C) -> Self {
        Self {
            paths,
            runner,
            clock,
        }
    }
}

impl<R: CommandRunner, C: Clock> InstallFinalizer<R, C> {
    pub fn finalize(&self) -> Result<(), UpdaterError> {
        let lock = UpgradeStateLock::acquire(&self.paths.root)?;
        let installed = read_installed_release(&self.paths.installed_release)?;
        run_migration(
            &self.runner,
            &self.paths.migrator,
            &self.paths.database,
            &self.paths.sql_root,
        )?;
        run_command(
            &self.runner,
            "finalizing_install",
            "systemctl",
            ["daemon-reload"],
            60,
        )?;
        let restarts_before = read_restart_count(&self.runner, "finalizing_install")?;
        let start_attempt_at = self.clock.now()?;
        run_command(
            &self.runner,
            "finalizing_install",
            "systemctl",
            ["start", "usb-control.service"],
            60,
        )?;
        check_health(
            &self.runner,
            &self.paths,
            &HealthExpectation {
                release: installed.clone(),
                schema_version: installed.supported_schema_max,
                start_attempt_at,
                restarts_before,
            },
        )?;
        let committed_at = self.clock.now()?;
        ActiveReleaseStore::new(self.paths.root.clone())?
            .commit(
                &lock,
                &ActiveRelease {
                    format_version: 1,
                    version: installed.version,
                    schema_version: installed.supported_schema_max,
                    committed_at,
                    online_upgrade_id: None,
                },
            )
            .map_err(|error| {
                UpdaterError::TaskInvalid(format!("直接安装活动发布提交失败: {error}"))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdaterCommand {
    Run { root: PathBuf },
    FinalizeInstall,
    Version,
}

pub fn parse_command<I, S>(args: I) -> Result<UpdaterCommand, UpdaterError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    match args.as_slice() {
        [_, flag] if matches!(flag.as_str(), "--version" | "-V") => Ok(UpdaterCommand::Version),
        [_, command] if command == "finalize-install" => Ok(UpdaterCommand::FinalizeInstall),
        [_, command, option, root]
            if command == "run" && option == "--root" && !root.is_empty() =>
        {
            Ok(UpdaterCommand::Run {
                root: PathBuf::from(root),
            })
        }
        _ => Err(UpdaterError::TaskInvalid(
            "usage: usb-control-updater run --root <upgrade-root> | finalize-install | --version"
                .into(),
        )),
    }
}
