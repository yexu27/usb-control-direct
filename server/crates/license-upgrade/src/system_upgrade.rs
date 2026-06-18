//! 系统升级管理。
//!
//! 负责升级包校验、文件替换和服务重启。升级过程中会自动备份旧版本，
//! 替换失败时回滚至备份版本。

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::error::LicenseUpgradeError;

/// 升级包校验结果（内部中间结构）。
pub struct UpgradeValidation {
    /// 升级数据。
    pub data: Vec<u8>,
    /// 目标版本。
    pub target_version: String,
}

/// 系统升级管理器。
pub struct SystemUpgradeManager {
    /// 安装目录。
    install_dir: PathBuf,
    /// 服务名称（用于 systemctl 重启）。
    service_name: String,
}

impl SystemUpgradeManager {
    /// 创建系统升级管理器。
    ///
    /// 参数:
    /// - `install_dir`: 安装目录路径。
    /// - `service_name`: systemd 服务名称。
    pub fn new(install_dir: impl Into<PathBuf>, service_name: impl Into<String>) -> Self {
        Self {
            install_dir: install_dir.into(),
            service_name: service_name.into(),
        }
    }

    /// 校验升级包。
    ///
    /// 参数:
    /// - `upgrade_data`: 升级包字节数据。
    /// - `target_version`: 目标版本号。
    /// - `current_version`: 当前版本号。
    ///
    /// 返回:
    /// - 成功时返回 [`UpgradeValidation`]；失败时返回 [`LicenseUpgradeError`]。
    pub fn validate_upgrade(
        &self,
        upgrade_data: Vec<u8>,
        target_version: &str,
        current_version: &str,
    ) -> Result<UpgradeValidation, LicenseUpgradeError> {
        if upgrade_data.is_empty() {
            return Err(LicenseUpgradeError::UpgradeFormatError);
        }

        // 路径遍历检查：target_version 会用于文件路径构造
        if target_version.contains("..")
            || target_version.contains('/')
            || target_version.contains('\\')
            || target_version.is_empty()
        {
            return Err(LicenseUpgradeError::UpgradeFormatError);
        }

        if !is_version_greater(target_version, current_version) {
            return Err(LicenseUpgradeError::VersionTooLow);
        }

        Ok(UpgradeValidation {
            data: upgrade_data,
            target_version: target_version.to_string(),
        })
    }

    /// 执行升级安装。
    ///
    /// 将升级数据写入临时文件，备份旧版本后原子替换。
    /// 替换失败时回滚至备份版本。
    ///
    /// 参数:
    /// - `validation`: 升级校验通过的数据。
    ///
    /// 返回:
    /// - 成功时返回 `()`；失败时返回 [`LicenseUpgradeError`]。
    pub fn apply_upgrade(
        &self,
        validation: UpgradeValidation,
    ) -> Result<(), LicenseUpgradeError> {
        let target_path = self.install_dir.join(&validation.target_version);
        let tmp_path = self.install_dir.join(format!("{}.tmp", &validation.target_version));
        let backup_path = self.install_dir.join(format!("{}.bak", &validation.target_version));

        // 写入临时文件
        fs::write(&tmp_path, &validation.data).map_err(|e| {
            LicenseUpgradeError::UpgradeApplyFailed(format!("写入临时文件失败: {e}"))
        })?;

        // 备份旧版本（如果存在）
        if target_path.exists() {
            if let Err(e) = fs::rename(&target_path, &backup_path) {
                let _ = fs::remove_file(&tmp_path);
                return Err(LicenseUpgradeError::UpgradeApplyFailed(format!(
                    "备份旧版本失败: {e}"
                )));
            }
        }

        // 原子替换
        if let Err(e) = fs::rename(&tmp_path, &target_path) {
            // 回滚
            if backup_path.exists() {
                let _ = fs::rename(&backup_path, &target_path);
            }
            return Err(LicenseUpgradeError::UpgradeApplyFailed(format!(
                "替换文件失败: {e}"
            )));
        }

        // 设置可执行权限
        let _ = fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755));

        // 清理备份
        let _ = fs::remove_file(&backup_path);

        Ok(())
    }

    /// 重启服务。
    ///
    /// 调用 `systemctl restart` 重启关联服务。
    ///
    /// 返回:
    /// - 成功时返回 `()`；失败时返回 [`LicenseUpgradeError`]。
    pub fn restart_service(&self) -> Result<(), LicenseUpgradeError> {
        let output = Command::new("systemctl")
            .arg("restart")
            .arg(&self.service_name)
            .output()
            .map_err(|e| {
                LicenseUpgradeError::UpgradeApplyFailed(format!(
                    "执行 systemctl restart 失败: {e}"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LicenseUpgradeError::UpgradeApplyFailed(format!(
                "服务重启失败: {stderr}"
            )));
        }

        Ok(())
    }
}

/// 比较两个语义版本号是否 new > current。
///
/// 版本号格式：`[vV]major.minor.patch`，前缀 V/v 可省略。
/// 按 major、minor、patch 逐段比较。
///
/// 参数:
/// - `new_version`: 新版本号。
/// - `current_version`: 当前版本号。
///
/// 返回:
/// - 新版本大于当前版本时返回 `true`。
pub fn is_version_greater(new_version: &str, current_version: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        let v = v.trim_start_matches(|c: char| c == 'v' || c == 'V');
        v.split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };

    let new_parts = parse(new_version);
    let cur_parts = parse(current_version);

    for i in 0..new_parts.len().max(cur_parts.len()) {
        let n = new_parts.get(i).copied().unwrap_or(0);
        let c = cur_parts.get(i).copied().unwrap_or(0);
        if n > c {
            return true;
        }
        if n < c {
            return false;
        }
    }

    false
}

/// 计算 SHA256 校验和。
///
/// 参数:
/// - `data`: 待计算校验和的数据。
///
/// 返回:
/// - 32 字节 SHA256 摘要。
pub fn sha256_checksum(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_greater_basic() {
        assert!(is_version_greater("1.1.0", "1.0.0"));
        assert!(is_version_greater("2.0.0", "1.9.9"));
        assert!(is_version_greater("1.0.1", "1.0.0"));
    }

    #[test]
    fn version_not_greater_when_equal() {
        assert!(!is_version_greater("1.0.0", "1.0.0"));
    }

    #[test]
    fn version_not_greater_when_lower() {
        assert!(!is_version_greater("1.0.0", "1.0.1"));
        assert!(!is_version_greater("0.9.0", "1.0.0"));
    }

    #[test]
    fn version_greater_with_v_prefix() {
        assert!(is_version_greater("v2.0.0", "V1.0.0"));
        assert!(is_version_greater("V1.1.0", "v1.0.0"));
    }

    #[test]
    fn version_greater_different_segment_count() {
        assert!(is_version_greater("1.0.1", "1.0"));
        assert!(!is_version_greater("1.0", "1.0.1"));
    }

    #[test]
    fn sha256_checksum_deterministic() {
        let data = b"test data";
        let sum1 = sha256_checksum(data);
        let sum2 = sha256_checksum(data);
        assert_eq!(sum1, sum2);
        assert_eq!(sum1.len(), 32);
    }

    #[test]
    fn validate_upgrade_empty_data_returns_error() {
        let mgr = SystemUpgradeManager::new("/tmp/test-install", "test-service");
        let result = mgr.validate_upgrade(vec![], "2.0.0", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn validate_upgrade_lower_version_returns_error() {
        let mgr = SystemUpgradeManager::new("/tmp/test-install", "test-service");
        let result = mgr.validate_upgrade(vec![1, 2, 3], "0.9.0", "1.0.0");
        assert!(matches!(result.unwrap_err(), LicenseUpgradeError::VersionTooLow));
    }

    #[test]
    fn validate_upgrade_success() {
        let mgr = SystemUpgradeManager::new("/tmp/test-install", "test-service");
        let result = mgr.validate_upgrade(vec![1, 2, 3], "2.0.0", "1.0.0");
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert_eq!(validation.target_version, "2.0.0");
        assert_eq!(validation.data, vec![1, 2, 3]);
    }
}
