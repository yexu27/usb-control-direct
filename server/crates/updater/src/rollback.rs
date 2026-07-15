//! 提交点之前的单一路径自动回滚。

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use sha2::{Digest, Sha256};
pub use system_upgrade::LastKnownGoodRelease;
use system_upgrade::{read_last_known_good, UpgradeManifest};

use crate::executor::{configure_and_reload, install_deb, run_command, Clock, CommandRunner};
use crate::health::{check_health, read_restart_count, HealthExpectation};
use crate::migration::run_migration;
use crate::{UpdaterError, UpgradePaths};

pub trait LkgRepository {
    fn prepare(
        &self,
        paths: &UpgradePaths,
        candidate: &Path,
        candidate_sha: &str,
    ) -> Result<(), UpdaterError>;
    fn promote(
        &self,
        paths: &UpgradePaths,
        manifest: &UpgradeManifest,
        candidate_sha: &str,
    ) -> Result<(), UpdaterError>;
    fn restore(
        &self,
        paths: &UpgradePaths,
        metadata: &LastKnownGoodRelease,
    ) -> Result<(), UpdaterError>;
    fn abort_prepared(&self, paths: &UpgradePaths) -> Result<(), UpdaterError>;
    fn cleanup_rollback(&self, paths: &UpgradePaths) -> Result<(), UpdaterError>;
    fn cleanup_committed(&self, paths: &UpgradePaths) -> Result<(), UpdaterError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FileLkgRepository;

impl LkgRepository for FileLkgRepository {
    fn prepare(
        &self,
        paths: &UpgradePaths,
        candidate: &Path,
        candidate_sha: &str,
    ) -> Result<(), UpdaterError> {
        prepare_transaction(paths, candidate, candidate_sha)
    }

    fn promote(
        &self,
        paths: &UpgradePaths,
        manifest: &UpgradeManifest,
        candidate_sha: &str,
    ) -> Result<(), UpdaterError> {
        promote_lkg(paths, manifest, candidate_sha)
    }

    fn restore(
        &self,
        paths: &UpgradePaths,
        metadata: &LastKnownGoodRelease,
    ) -> Result<(), UpdaterError> {
        restore_lkg_layout(paths, metadata)
    }

    fn abort_prepared(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        abort_prepared(paths)
    }

    fn cleanup_rollback(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        cleanup_rollback(paths)
    }

    fn cleanup_committed(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        cleanup_committed(paths)
    }
}

pub(crate) fn abort_prepared(paths: &UpgradePaths) -> Result<(), UpdaterError> {
    cleanup_paths([
        paths.next_last_known_good_deb.as_path(),
        paths.managed_marker.as_path(),
    ])
}

pub(crate) fn prepare_transaction(
    paths: &UpgradePaths,
    candidate: &Path,
    candidate_sha: &str,
) -> Result<(), UpdaterError> {
    if paths.managed_marker.exists()
        || paths.next_last_known_good_deb.exists()
        || paths.previous_deb.exists()
    {
        return Err(UpdaterError::TaskInvalid(
            "检测到未清理的升级事务文件，需人工确认".into(),
        ));
    }
    let result = (|| {
        copy_file_synced(candidate, &paths.next_last_known_good_deb)?;
        if sha256_file(&paths.next_last_known_good_deb)? != candidate_sha {
            return Err(UpdaterError::TaskInvalid(
                "已同步候选 DEB 摘要不匹配".into(),
            ));
        }
        create_marker(&paths.managed_marker)
    })();
    match result {
        Ok(()) => Ok(()),
        Err(original) => match cleanup_paths([
            paths.next_last_known_good_deb.as_path(),
            paths.managed_marker.as_path(),
        ]) {
            Ok(()) => Err(original),
            Err(cleanup) => Err(UpdaterError::TaskInvalid(format!(
                "升级准备失败；原始错误: {original}；清理错误: {cleanup}"
            ))),
        },
    }
}

pub(crate) fn cleanup_rollback(paths: &UpgradePaths) -> Result<(), UpdaterError> {
    cleanup_paths([paths.managed_marker.as_path()])
}

pub(crate) fn cleanup_committed(paths: &UpgradePaths) -> Result<(), UpdaterError> {
    // 不使用短路表达式：无论 previous 删除是否失败，都继续尝试 managed marker。
    cleanup_paths([paths.previous_deb.as_path(), paths.managed_marker.as_path()])
}

pub(crate) fn read_and_validate_lkg(
    paths: &UpgradePaths,
) -> Result<LastKnownGoodRelease, UpdaterError> {
    read_last_known_good(&paths.last_known_good_metadata, &paths.last_known_good_deb)
        .map_err(|error| UpdaterError::TaskInvalid(error.to_string()))
}

pub(crate) fn promote_lkg(
    paths: &UpgradePaths,
    manifest: &UpgradeManifest,
    candidate_sha: &str,
) -> Result<(), UpdaterError> {
    fs::rename(&paths.last_known_good_deb, &paths.previous_deb)?;
    sync_parent(&paths.previous_deb)?;
    fs::rename(&paths.next_last_known_good_deb, &paths.last_known_good_deb)?;
    sync_parent(&paths.last_known_good_deb)?;
    atomic_write_lkg(
        &paths.last_known_good_metadata,
        &LastKnownGoodRelease {
            format_version: 1,
            version: manifest.package_version,
            deb_sha256: candidate_sha.to_string(),
            schema_version: manifest.schema_to,
            tls_cert_sha256: manifest.tls_cert_sha256.clone(),
        },
    )
}

pub(crate) fn restore_lkg_layout(
    paths: &UpgradePaths,
    metadata: &LastKnownGoodRelease,
) -> Result<(), UpdaterError> {
    if paths.previous_deb.is_file() {
        if paths.last_known_good_deb.exists() {
            fs::remove_file(&paths.last_known_good_deb)?;
        }
        fs::rename(&paths.previous_deb, &paths.last_known_good_deb)?;
        sync_parent(&paths.last_known_good_deb)?;
    }
    if paths.next_last_known_good_deb.is_file() {
        fs::remove_file(&paths.next_last_known_good_deb)?;
        sync_parent(&paths.next_last_known_good_deb)?;
    }
    atomic_write_lkg(&paths.last_known_good_metadata, metadata)
}

pub(crate) fn rollback(
    runner: &dyn CommandRunner,
    paths: &UpgradePaths,
    rollback_deb: &Path,
    lkg: &LastKnownGoodRelease,
    clock: &dyn Clock,
    not_before: i64,
) -> Result<i64, UpdaterError> {
    run_command(
        runner,
        "rollback_stop",
        "systemctl",
        ["stop", "usb-control.service"],
        60,
    )?;
    install_deb(runner, "rollback_install", rollback_deb)?;
    run_migration(runner, &paths.migrator, &paths.database, &paths.sql_root)?;
    configure_and_reload(runner)?;
    let restarts_before = read_restart_count(runner, "rollback_start")?;
    let start_attempt_at = clock.now()?.max(not_before);
    run_command(
        runner,
        "rollback_start",
        "systemctl",
        ["start", "usb-control.service"],
        60,
    )?;
    check_health(
        runner,
        paths,
        &HealthExpectation {
            version: lkg.version,
            schema_version: lkg.schema_version,
            tls_cert_sha256: lkg.tls_cert_sha256.clone(),
            start_attempt_at,
            restarts_before,
        },
    )?;
    Ok(start_attempt_at)
}

fn sha256_file(path: &Path) -> Result<String, UpdaterError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn copy_file_synced(source: &Path, target: &Path) -> Result<(), UpdaterError> {
    let parent = target
        .parent()
        .ok_or_else(|| UpdaterError::TaskInvalid("候选副本路径缺少父目录".into()))?;
    create_private_dir_all(parent)?;
    let mut input = File::open(source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(target)?;
    std::io::copy(&mut input, &mut output)?;
    output.sync_all()?;
    sync_parent(target)
}

fn create_marker(path: &Path) -> Result<(), UpdaterError> {
    if let Some(parent) = path.parent() {
        create_private_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(b"managed\n")?;
    file.sync_all()?;
    sync_parent(path)
}

fn cleanup_paths<'a>(paths: impl IntoIterator<Item = &'a Path>) -> Result<(), UpdaterError> {
    let mut errors = Vec::new();
    for path in paths {
        if let Err(error) = retry_remove(path) {
            errors.push(format!("{}: {error}", path.display()));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(UpdaterError::TaskInvalid(format!(
            "升级事务清理失败: {}",
            errors.join("; ")
        )))
    }
}

fn retry_remove(path: &Path) -> Result<(), UpdaterError> {
    let mut last = None;
    for _ in 0..3 {
        match fs::remove_file(path) {
            Ok(()) => return sync_parent(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => last = Some(error),
        }
    }
    Err(last
        .expect("remove retry loop has at least one iteration")
        .into())
}

fn create_private_dir_all(path: &Path) -> Result<(), UpdaterError> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn atomic_write_lkg(path: &Path, value: &LastKnownGoodRelease) -> Result<(), UpdaterError> {
    let parent = path
        .parent()
        .ok_or_else(|| UpdaterError::TaskInvalid("LKG 元数据路径缺少父目录".into()))?;
    let temporary = path.with_file_name(format!(".last-known-good.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    let result = (|| -> Result<(), UpdaterError> {
        file.write_all(&serde_json::to_vec(value)?)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn sync_parent(path: &Path) -> Result<(), UpdaterError> {
    let parent = path
        .parent()
        .ok_or_else(|| UpdaterError::TaskInvalid("LKG 路径缺少父目录".into()))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
