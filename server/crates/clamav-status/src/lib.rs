use std::process::Command;

use chrono::{DateTime, NaiveDateTime, Utc};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClamavStatus {
    pub engine_version: String,
    pub virus_db_version: String,
    pub virus_db_updated_at: i64,
    pub raw_output: String,
}

#[derive(Debug, Error)]
pub enum ClamavStatusError {
    #[error("invalid ClamAV version output: {0}")]
    InvalidOutput(String),
    #[error("parse ClamAV database time failed: {0}")]
    InvalidDatabaseTime(String),
    #[error("execute {command} --version failed: {source}")]
    CommandFailed {
        command: String,
        source: std::io::Error,
    },
    #[error("{command} --version exited with {status}: {stderr}")]
    CommandUnsuccessful {
        command: String,
        status: String,
        stderr: String,
    },
}

pub fn parse_clamav_version_output(output: &str) -> Result<ClamavStatus, ClamavStatusError> {
    let raw_output = output.trim().to_string();
    let rest = raw_output
        .strip_prefix("ClamAV ")
        .ok_or_else(|| ClamavStatusError::InvalidOutput(raw_output.clone()))?;
    let mut parts = rest.splitn(3, '/');
    let engine_version = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ClamavStatusError::InvalidOutput(raw_output.clone()))?;
    let virus_db_version = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ClamavStatusError::InvalidOutput(raw_output.clone()))?;
    let database_time = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ClamavStatusError::InvalidOutput(raw_output.clone()))?;
    let parsed_time =
        NaiveDateTime::parse_from_str(database_time, "%a %b %d %H:%M:%S %Y").map_err(|_| {
            ClamavStatusError::InvalidDatabaseTime(database_time.to_string())
        })?;

    Ok(ClamavStatus {
        engine_version: engine_version.to_string(),
        virus_db_version: virus_db_version.to_string(),
        virus_db_updated_at: DateTime::<Utc>::from_naive_utc_and_offset(parsed_time, Utc)
            .timestamp(),
        raw_output,
    })
}

pub fn read_clamav_status(command_path: &str) -> Result<ClamavStatus, ClamavStatusError> {
    let output = Command::new(command_path)
        .arg("--version")
        .output()
        .map_err(|source| ClamavStatusError::CommandFailed {
            command: command_path.to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(ClamavStatusError::CommandUnsuccessful {
            command: command_path.to_string(),
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    parse_clamav_version_output(&String::from_utf8_lossy(&output.stdout))
}
