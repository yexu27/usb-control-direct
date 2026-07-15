//! 系统升级受理前的只读 Linux 环境预检。

use std::ffi::{CString, OsStr};
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use system_upgrade::{
    certificate_sha256, read_active_release, read_last_known_good, DebInspector, UpgradeError,
    UpgradePreflight, UpgradePreflightFailure, UpgradePreflightRequest,
};
use wait_timeout::ChildExt;

const SAFETY_MARGIN: u64 = 256 * 1024 * 1024;
const PRODUCTION_TIMEOUT: Duration = Duration::from_secs(10);
const PRODUCTION_OUTPUT_LIMIT: usize = 64 * 1024;

/// Linux 状态查询端口，隔离领域预检决策与操作系统探测。
pub trait UpgradeHostProbe: Send + Sync {
    fn available_bytes(&self, path: &Path) -> Result<u64, String>;
    fn dpkg_locks_available(&self) -> Result<bool, String>;
    fn dpkg_audit_clean(&self) -> Result<bool, String>;
    fn main_service_active(&self) -> Result<bool, String>;
    fn clamav_available(&self) -> Result<bool, String>;
    fn platform_compatible(&self) -> Result<bool, String>;
}

/// 有界命令输出。
#[derive(Debug)]
pub struct BoundedCommandOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// 无 shell、带超时且并发排空输出的固定命令执行器。
#[derive(Debug, Clone, Copy)]
pub struct BoundedCommandRunner {
    timeout: Duration,
    output_limit: usize,
}

impl BoundedCommandRunner {
    pub fn new(timeout: Duration, output_limit: usize) -> Self {
        Self {
            timeout,
            output_limit,
        }
    }

    pub fn run(&self, program: &Path, args: &[&OsStr]) -> Result<BoundedCommandOutput, String> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("命令启动失败: {error}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "无法读取命令 stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "无法读取命令 stderr".to_string())?;
        let limit = self.output_limit;
        let stdout_reader = thread::spawn(move || read_capped(stdout, limit));
        let stderr_reader = thread::spawn(move || read_capped(stderr, limit));

        let status = match child.wait_timeout(self.timeout) {
            Ok(Some(status)) => status,
            Ok(None) => {
                terminate_and_reap(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err("命令执行超时".into());
            }
            Err(error) => {
                terminate_and_reap(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(format!("等待命令失败: {error}"));
            }
        };
        let stdout = stdout_reader
            .join()
            .map_err(|_| "stdout 读取线程异常".to_string())??;
        let stderr = stderr_reader
            .join()
            .map_err(|_| "stderr 读取线程异常".to_string())??;
        Ok(BoundedCommandOutput {
            status,
            stdout,
            stderr,
        })
    }
}

fn read_capped(mut reader: impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut retained = Vec::with_capacity(limit);
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("读取命令输出失败: {error}"))?;
        if read == 0 {
            return Ok(retained);
        }
        let remaining = limit.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..read.min(remaining)]);
    }
}

fn terminate_and_reap(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// 返回文件系统可用字节数。
pub fn available_bytes(path: &Path) -> Result<u64, String> {
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| "空间探测路径包含 NUL".to_string())?;
    let mut value = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    let result = unsafe { libc::statvfs(path.as_ptr(), value.as_mut_ptr()) };
    if result != 0 {
        return Err(format!("statvfs 失败: {}", std::io::Error::last_os_error()));
    }
    let value = unsafe { value.assume_init() };
    value
        .f_bavail
        .checked_mul(value.f_frsize)
        .ok_or_else(|| "可用空间计算溢出".into())
}

/// 依次尝试非阻塞 POSIX 写锁；函数返回时自动释放已取得的锁。
pub fn record_locks_available(paths: &[PathBuf]) -> Result<bool, String> {
    let mut locked = Vec::with_capacity(paths.len());
    for path in paths {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|error| format!("打开 dpkg 锁失败: {error}"))?;
        let mut lock = libc::flock {
            l_type: libc::F_WRLCK as i16,
            l_whence: libc::SEEK_SET as i16,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };
        let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLK, &mut lock) };
        if result != 0 {
            let error = std::io::Error::last_os_error();
            if matches!(
                error.raw_os_error(),
                Some(libc::EACCES) | Some(libc::EAGAIN)
            ) {
                return Ok(false);
            }
            return Err(format!("dpkg 锁探测失败: {error}"));
        }
        locked.push(file);
    }
    Ok(true)
}

/// 严格匹配 RK3568 正式运行平台。
pub fn platform_matches(os_release: &str, machine: &str, kernel: &str) -> bool {
    let mut id = None;
    let mut version = None;
    for line in os_release.lines() {
        if let Some(value) = line.strip_prefix("ID=") {
            id = Some(value.trim_matches(['\"', '\'']));
        } else if let Some(value) = line.strip_prefix("VERSION_ID=") {
            version = Some(value.trim_matches(['\"', '\'']));
        }
    }
    id == Some("ubuntu")
        && version == Some("22.04")
        && machine.trim() == "aarch64"
        && kernel.trim().starts_with("4.19.")
}

/// 生产 Linux 环境探针；命令和 systemd unit 均不可由升级包覆盖。
pub struct LinuxUpgradeHostProbe {
    clamdscan_path: PathBuf,
    runner: BoundedCommandRunner,
}

impl LinuxUpgradeHostProbe {
    pub fn production(clamdscan_path: PathBuf) -> Self {
        Self {
            clamdscan_path,
            runner: BoundedCommandRunner::new(PRODUCTION_TIMEOUT, PRODUCTION_OUTPUT_LIMIT),
        }
    }

    fn command(&self, program: &Path, args: &[&OsStr]) -> Result<BoundedCommandOutput, String> {
        self.runner.run(program, args)
    }
}

impl UpgradeHostProbe for LinuxUpgradeHostProbe {
    fn available_bytes(&self, path: &Path) -> Result<u64, String> {
        available_bytes(path)
    }

    fn dpkg_locks_available(&self) -> Result<bool, String> {
        record_locks_available(&[
            PathBuf::from("/var/lib/dpkg/lock-frontend"),
            PathBuf::from("/var/lib/dpkg/lock"),
        ])
    }

    fn dpkg_audit_clean(&self) -> Result<bool, String> {
        let output = self.command(Path::new("/usr/bin/dpkg"), &[OsStr::new("--audit")])?;
        Ok(output.status.success()
            && output.stdout.iter().all(u8::is_ascii_whitespace)
            && output.stderr.iter().all(u8::is_ascii_whitespace))
    }

    fn main_service_active(&self) -> Result<bool, String> {
        let output = self.command(
            Path::new("/usr/bin/systemctl"),
            &[
                OsStr::new("is-active"),
                OsStr::new("--quiet"),
                OsStr::new("usb-control.service"),
            ],
        )?;
        Ok(output.status.success())
    }

    fn clamav_available(&self) -> Result<bool, String> {
        Ok(self
            .command(&self.clamdscan_path, &[OsStr::new("--version")])?
            .status
            .success())
    }

    fn platform_compatible(&self) -> Result<bool, String> {
        let os_release = fs::read_to_string("/etc/os-release")
            .map_err(|error| format!("读取 os-release 失败: {error}"))?;
        let machine = self.command(Path::new("/usr/bin/uname"), &[OsStr::new("-m")])?;
        let kernel = self.command(Path::new("/usr/bin/uname"), &[OsStr::new("-r")])?;
        if !machine.status.success() || !kernel.status.success() {
            return Ok(false);
        }
        Ok(platform_matches(
            &os_release,
            std::str::from_utf8(&machine.stdout).unwrap_or_default(),
            std::str::from_utf8(&kernel.stdout).unwrap_or_default(),
        ))
    }
}

/// 将严格发布/LKG 一致性与 Linux 状态探测组合成领域预检端口。
pub struct SystemUpgradePreflight {
    probe: Arc<dyn UpgradeHostProbe>,
    deb_inspector: Arc<dyn DebInspector>,
    upgrade_root: PathBuf,
    active_release_path: PathBuf,
    lkg_metadata_path: PathBuf,
    lkg_deb_path: PathBuf,
    tls_cert_path: PathBuf,
}

impl SystemUpgradePreflight {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        probe: Arc<dyn UpgradeHostProbe>,
        deb_inspector: Arc<dyn DebInspector>,
        upgrade_root: PathBuf,
        active_release_path: PathBuf,
        lkg_metadata_path: PathBuf,
        lkg_deb_path: PathBuf,
        tls_cert_path: PathBuf,
    ) -> Self {
        Self {
            probe,
            deb_inspector,
            upgrade_root,
            active_release_path,
            lkg_metadata_path,
            lkg_deb_path,
            tls_cert_path,
        }
    }

    fn rollback_unavailable<T>(
        result: Result<T, impl std::fmt::Display>,
    ) -> Result<T, UpgradeError> {
        result.map_err(|_| UpgradeError::Preflight(UpgradePreflightFailure::RollbackUnavailable))
    }

    fn probe_failure<T>(result: Result<T, String>) -> Result<T, UpgradeError> {
        result.map_err(|error| UpgradeError::Preflight(UpgradePreflightFailure::ProbeFailed(error)))
    }
}

impl UpgradePreflight for SystemUpgradePreflight {
    fn check(&self, request: &UpgradePreflightRequest) -> Result<(), UpgradeError> {
        let active = Self::rollback_unavailable(read_active_release(&self.active_release_path))?;
        let lkg = Self::rollback_unavailable(read_last_known_good(
            &self.lkg_metadata_path,
            &self.lkg_deb_path,
        ))?;
        let deb = Self::rollback_unavailable(self.deb_inspector.inspect(&self.lkg_deb_path))?;
        let installed_tls_bytes = Self::rollback_unavailable(fs::read(&self.tls_cert_path))?;
        let installed_tls = Self::rollback_unavailable(certificate_sha256(&installed_tls_bytes))?;
        if active.version != request.source_version
            || active.schema_version != request.schema_from
            || lkg.version != request.source_version
            || lkg.schema_version != request.schema_from
            || active.deb_sha256 != lkg.deb_sha256
            || deb.package != "usb-control"
            || deb.architecture != "arm64"
            || deb.version != request.source_version
            || deb.migration_schema_to != request.schema_from
            || deb.supported_schema_min > request.schema_from
            || deb.supported_schema_max < request.schema_from
            || lkg.tls_cert_sha256 != deb.tls_cert_sha256
            || lkg.tls_cert_sha256 != installed_tls
        {
            return Err(UpgradeError::Preflight(
                UpgradePreflightFailure::RollbackUnavailable,
            ));
        }

        let lkg_size = Self::rollback_unavailable(fs::metadata(&self.lkg_deb_path))?.len();
        let required = request
            .package_size
            .checked_add(request.deb_size)
            .and_then(|value| value.checked_add(lkg_size))
            .and_then(|value| value.checked_add(request.expanded_size))
            .and_then(|value| value.checked_add(SAFETY_MARGIN))
            .ok_or_else(|| {
                UpgradeError::Preflight(UpgradePreflightFailure::ProbeFailed(
                    "升级空间预算溢出".into(),
                ))
            })?;
        let available = Self::probe_failure(self.probe.available_bytes(&self.upgrade_root))?;
        if available < required {
            return Err(UpgradeError::Preflight(
                UpgradePreflightFailure::InsufficientSpace {
                    required,
                    available,
                },
            ));
        }
        if !Self::probe_failure(self.probe.dpkg_locks_available())? {
            return Err(UpgradeError::Preflight(UpgradePreflightFailure::DpkgBusy));
        }
        if !Self::probe_failure(self.probe.dpkg_audit_clean())? {
            return Err(UpgradeError::Preflight(
                UpgradePreflightFailure::DpkgDamaged,
            ));
        }
        if !Self::probe_failure(self.probe.main_service_active())? {
            return Err(UpgradeError::Preflight(
                UpgradePreflightFailure::ServiceUnavailable,
            ));
        }
        if !Self::probe_failure(self.probe.clamav_available())? {
            return Err(UpgradeError::Preflight(
                UpgradePreflightFailure::ClamAvUnavailable,
            ));
        }
        if !Self::probe_failure(self.probe.platform_compatible())? {
            return Err(UpgradeError::Preflight(
                UpgradePreflightFailure::PlatformIncompatible,
            ));
        }
        Ok(())
    }
}
