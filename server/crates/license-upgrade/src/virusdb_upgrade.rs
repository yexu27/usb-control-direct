//! 病毒库升级管理。
//!
//! 负责病毒库版本校验、zip 解压、文件替换和 clamd 重新加载。
//! 升级失败时自动回滚至备份版本。

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::LicenseUpgradeError;

/// 病毒库升级管理器。
pub struct VirusdbUpgradeManager {
    /// 病毒库目录。
    db_dir: PathBuf,
}

impl VirusdbUpgradeManager {
    /// 创建病毒库升级管理器。
    ///
    /// 参数:
    /// - `db_dir`: 病毒库目录路径，默认 `/var/lib/clamav`。
    pub fn new(db_dir: impl Into<PathBuf>) -> Self {
        Self {
            db_dir: db_dir.into(),
        }
    }

    /// 使用默认路径创建病毒库升级管理器。
    pub fn with_default_path() -> Self {
        Self::new("/var/lib/clamav")
    }

    /// 校验病毒库版本号。
    ///
    /// 检查目标版本是否大于当前版本，且版本号各段不包含数字 4。
    ///
    /// 参数:
    /// - `target_version`: 目标版本号。
    /// - `current_version`: 当前版本号。
    ///
    /// 返回:
    /// - 成功时返回 `()`；失败时返回 [`LicenseUpgradeError`]。
    pub fn validate_upgrade(
        &self,
        target_version: &str,
        current_version: &str,
    ) -> Result<(), LicenseUpgradeError> {
        if contains_digit_4(target_version) {
            return Err(LicenseUpgradeError::VersionNumberForbidden);
        }

        if !crate::system_upgrade::is_version_greater(target_version, current_version) {
            return Err(LicenseUpgradeError::VersionTooLow);
        }

        Ok(())
    }

    /// 执行病毒库升级。
    ///
    /// 解压 zip 数据 → 备份当前病毒库 → 替换文件 → 重新加载 clamd。
    /// clamd 重新加载失败时回滚至备份版本。
    ///
    /// 参数:
    /// - `upgrade_data`: zip 格式的病毒库升级包字节数据。
    ///
    /// 返回:
    /// - 成功时返回 `()`；失败时返回 [`LicenseUpgradeError`]。
    pub fn apply_upgrade(&self, upgrade_data: &[u8]) -> Result<(), LicenseUpgradeError> {
        let extracted_files = self.extract_zip(upgrade_data)?;

        if extracted_files.is_empty() {
            return Err(LicenseUpgradeError::VirusdbIntegrityError);
        }

        let backup_dir = self.db_dir.join(".backup");
        self.backup_current(&backup_dir)?;

        if let Err(e) = self.replace_files(&extracted_files) {
            self.restore_from_backup(&backup_dir);
            return Err(e);
        }

        if let Err(e) = self.reload_clamd() {
            self.restore_from_backup(&backup_dir);
            return Err(e);
        }

        // 升级成功，清理备份
        let _ = fs::remove_dir_all(&backup_dir);

        Ok(())
    }

    /// 解压 zip 数据。
    fn extract_zip(
        &self,
        data: &[u8],
    ) -> Result<Vec<(String, Vec<u8>)>, LicenseUpgradeError> {
        let cursor = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            LicenseUpgradeError::VirusdbApplyFailed(format!("zip 解压初始化失败: {e}"))
        })?;

        let mut files = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                LicenseUpgradeError::VirusdbApplyFailed(format!("读取 zip 条目失败: {e}"))
            })?;

            // 跳过目录
            if file.is_dir() {
                continue;
            }

            let name = file.name().to_string();

            // 安全检查：跳过包含路径遍历或绝对路径的条目
            if name.contains("..") || name.starts_with('/') {
                continue;
            }

            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(|e| {
                LicenseUpgradeError::VirusdbApplyFailed(format!("读取 zip 文件内容失败: {e}"))
            })?;

            // 只取文件名，不保留目录结构
            let file_name = match Path::new(&name).file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            files.push((file_name, content));
        }

        Ok(files)
    }

    /// 备份当前病毒库文件。
    fn backup_current(&self, backup_dir: &Path) -> Result<(), LicenseUpgradeError> {
        if backup_dir.exists() {
            fs::remove_dir_all(backup_dir).map_err(|e| {
                LicenseUpgradeError::VirusdbApplyFailed(format!("清理旧备份失败: {e}"))
            })?;
        }

        fs::create_dir_all(backup_dir).map_err(|e| {
            LicenseUpgradeError::VirusdbApplyFailed(format!("创建备份目录失败: {e}"))
        })?;

        if !self.db_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.db_dir).map_err(|e| {
            LicenseUpgradeError::VirusdbApplyFailed(format!("读取病毒库目录失败: {e}"))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                LicenseUpgradeError::VirusdbApplyFailed(format!("遍历病毒库目录失败: {e}"))
            })?;

            let path = entry.path();
            let file_name_str = entry.file_name().to_string_lossy().to_string();
            if file_name_str.starts_with('.') {
                continue;
            }
            if path.is_file() {
                let file_name = entry.file_name();
                let dest = backup_dir.join(&file_name);
                fs::copy(&path, &dest).map_err(|e| {
                    LicenseUpgradeError::VirusdbApplyFailed(format!(
                        "备份文件 {} 失败: {e}",
                        file_name.to_string_lossy()
                    ))
                })?;
            }
        }

        Ok(())
    }

    /// 替换病毒库文件。
    fn replace_files(
        &self,
        files: &[(String, Vec<u8>)],
    ) -> Result<(), LicenseUpgradeError> {
        fs::create_dir_all(&self.db_dir).map_err(|e| {
            LicenseUpgradeError::VirusdbApplyFailed(format!("创建病毒库目录失败: {e}"))
        })?;

        for (name, content) in files {
            let target_path = self.db_dir.join(name);
            fs::write(&target_path, content).map_err(|e| {
                LicenseUpgradeError::VirusdbApplyFailed(format!(
                    "写入文件 {name} 失败: {e}"
                ))
            })?;
        }

        Ok(())
    }

    /// 从备份目录恢复病毒库文件。
    fn restore_from_backup(&self, backup_dir: &Path) {
        if !backup_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(backup_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let dest = self.db_dir.join(entry.file_name());
                    let _ = fs::copy(&path, &dest);
                }
            }
        }

        let _ = fs::remove_dir_all(backup_dir);
    }

    /// 重新加载 clamd 病毒库。
    fn reload_clamd(&self) -> Result<(), LicenseUpgradeError> {
        let output = Command::new("clamdscan")
            .arg("--reload")
            .output()
            .map_err(|e| {
                LicenseUpgradeError::VirusdbApplyFailed(format!(
                    "执行 clamdscan --reload 失败: {e}"
                ))
            })?;

        if !output.status.success() {
            return Err(LicenseUpgradeError::ClamdReloadFailed);
        }

        Ok(())
    }
}

/// 检查版本号各段是否包含数字 4。
///
/// 版本号格式：`[vV]major.minor.patch`，前缀 V/v 可省略。
/// 检查每一段（以 `.` 分割）的字符串表示中是否包含字符 '4'。
///
/// 参数:
/// - `version`: 版本号字符串。
///
/// 返回:
/// - 任一段包含数字 4 时返回 `true`。
pub fn contains_digit_4(version: &str) -> bool {
    let v = version.trim_start_matches(|c: char| c == 'v' || c == 'V');
    v.split('.').any(|segment| segment.contains('4'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_digit_4_positive_cases() {
        assert!(contains_digit_4("1.4.0"));
        assert!(contains_digit_4("4.0.0"));
        assert!(contains_digit_4("1.0.4"));
        assert!(contains_digit_4("14.0.0"));
        assert!(contains_digit_4("1.0.42"));
        assert!(contains_digit_4("v1.4.0"));
        assert!(contains_digit_4("V4.0.0"));
    }

    #[test]
    fn contains_digit_4_negative_cases() {
        assert!(!contains_digit_4("1.0.0"));
        assert!(!contains_digit_4("1.2.3"));
        assert!(!contains_digit_4("5.6.7"));
        assert!(!contains_digit_4("v1.0.0"));
        assert!(!contains_digit_4("10.20.30"));
    }

    #[test]
    fn validate_upgrade_version_with_4_returns_forbidden() {
        let mgr = VirusdbUpgradeManager::new("/tmp/test-virusdb");
        let result = mgr.validate_upgrade("1.4.0", "1.0.0");
        assert!(matches!(
            result.unwrap_err(),
            LicenseUpgradeError::VersionNumberForbidden
        ));
    }

    #[test]
    fn validate_upgrade_lower_version_returns_error() {
        let mgr = VirusdbUpgradeManager::new("/tmp/test-virusdb");
        let result = mgr.validate_upgrade("1.0.0", "2.0.0");
        assert!(matches!(
            result.unwrap_err(),
            LicenseUpgradeError::VersionTooLow
        ));
    }

    #[test]
    fn validate_upgrade_success() {
        let mgr = VirusdbUpgradeManager::new("/tmp/test-virusdb");
        let result = mgr.validate_upgrade("2.0.0", "1.0.0");
        assert!(result.is_ok());
    }
}
