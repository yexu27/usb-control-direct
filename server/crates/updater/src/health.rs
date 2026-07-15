//! 主服务启动后的严格健康判定。

use std::fs;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use system_upgrade::{
    certificate_sha256 as shared_certificate_sha256, read_installed_release, InstalledRelease,
    ServiceReady,
};

use crate::executor::command;
use crate::{CommandRunner, UpdaterError, UpgradePaths};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthExpectation {
    pub release: InstalledRelease,
    pub schema_version: u32,
    pub start_attempt_at: i64,
    pub restarts_before: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceSnapshot {
    pub active: bool,
    pub main_pid: u32,
    pub restarts_after: u64,
    pub ready: ServiceReady,
    pub installed_release: InstalledRelease,
    pub tls_cert_sha256: String,
    pub tls_handshake_ok: bool,
}

pub fn validate_health_snapshot(
    expected: &HealthExpectation,
    actual: &ServiceSnapshot,
) -> Result<(), UpdaterError> {
    if !actual.active {
        return Err(UpdaterError::HealthFailed("服务未进入 active".into()));
    }
    if actual.ready.format_version != 1 {
        return Err(UpdaterError::HealthFailed("ready 格式版本不受支持".into()));
    }
    if actual.ready.pid != actual.main_pid || actual.main_pid == 0 {
        return Err(UpdaterError::HealthFailed(
            "ready PID 与 systemd MainPID 不一致".into(),
        ));
    }
    if actual.ready.started_at < expected.start_attempt_at {
        return Err(UpdaterError::HealthFailed("ready 早于本次启动尝试".into()));
    }
    if actual.ready.version != expected.release.version
        || actual.installed_release != expected.release
    {
        return Err(UpdaterError::HealthFailed(
            "服务或安装元数据版本不匹配".into(),
        ));
    }
    if actual.ready.schema_version != expected.schema_version {
        return Err(UpdaterError::HealthFailed(
            "数据库 Schema 版本不匹配".into(),
        ));
    }
    if actual.tls_cert_sha256 != expected.release.tls_cert_sha256 || !actual.tls_handshake_ok {
        return Err(UpdaterError::HealthFailed("TLS 身份或握手不匹配".into()));
    }
    if actual.restarts_after != expected.restarts_before {
        return Err(UpdaterError::HealthFailed("健康窗口内服务发生重启".into()));
    }
    Ok(())
}

pub fn certificate_sha256(pem: &[u8]) -> Result<String, UpdaterError> {
    shared_certificate_sha256(pem).map_err(UpdaterError::Domain)
}

pub(crate) fn read_restart_count(
    runner: &dyn CommandRunner,
    stage: &str,
) -> Result<u64, UpdaterError> {
    let output = runner.run(&command(
        stage,
        "systemctl",
        [
            "show",
            "--property=NRestarts",
            "--value",
            "usb-control.service",
        ],
        Duration::from_secs(10),
    ))?;
    parse_number(&output.stdout, "NRestarts")
}

pub(crate) fn check_health(
    runner: &dyn CommandRunner,
    paths: &UpgradePaths,
    expected: &HealthExpectation,
) -> Result<(), UpdaterError> {
    let started = Instant::now();
    loop {
        match check_health_once(runner, paths, expected) {
            Ok(()) => return Ok(()),
            Err(error) if started.elapsed() < paths.health_timeout => {
                std::thread::sleep(Duration::from_millis(500));
                let _ = error;
            }
            Err(error) => return Err(error),
        }
    }
}

fn check_health_once(
    runner: &dyn CommandRunner,
    paths: &UpgradePaths,
    expected: &HealthExpectation,
) -> Result<(), UpdaterError> {
    let active_output = runner.run(&command(
        "health_checking",
        "systemctl",
        ["is-active", "usb-control.service"],
        Duration::from_secs(10),
    ))?;
    let active = trim_ascii(&active_output.stdout) == b"active";
    let pid_output = runner.run(&command(
        "health_checking",
        "systemctl",
        [
            "show",
            "--property=MainPID",
            "--value",
            "usb-control.service",
        ],
        Duration::from_secs(10),
    ))?;
    let main_pid = u32::try_from(parse_number(&pid_output.stdout, "MainPID")?)
        .map_err(|_| UpdaterError::HealthFailed("MainPID 超出范围".into()))?;
    let restarts_after = read_restart_count(runner, "health_checking")?;
    let ready: ServiceReady = serde_json::from_slice(&fs::read(&paths.ready_file)?)?;
    let installed_release = read_installed_release(&paths.installed_release)?;
    let tls_cert_sha256 = certificate_sha256(&fs::read(&paths.tls_certificate)?)?;
    let tls_output = runner.run(&command(
        "health_checking",
        "openssl",
        [
            "s_client",
            "-connect",
            "127.0.0.1:9600",
            "-CAfile",
            paths
                .tls_certificate
                .to_str()
                .ok_or_else(|| UpdaterError::HealthFailed("TLS 证书路径不是 UTF-8".into()))?,
            "-verify_return_error",
            "-brief",
        ],
        Duration::from_secs(10),
    ))?;
    let snapshot = ServiceSnapshot {
        active,
        main_pid,
        restarts_after,
        ready,
        installed_release,
        tls_cert_sha256,
        tls_handshake_ok: tls_output.success,
    };
    validate_health_snapshot(expected, &snapshot)
}

fn parse_number(bytes: &[u8], field: &str) -> Result<u64, UpdaterError> {
    std::str::from_utf8(trim_ascii(bytes))
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| UpdaterError::HealthFailed(format!("{field} 输出非法")))
}

fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[1..];
    }
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}
