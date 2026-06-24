//! 策略导入事务操作。
//!
//! 在单个事务中清空并重写白名单、文件访问策略和文件类型黑名单，
//! 保证导入操作的原子性。

use std::collections::HashSet;

use rusqlite::params;
use tracing::{debug, info};

use crate::error::StorageError;
use crate::extension::normalize_extension;
use crate::model::{UsbWhitelistInsert, WhitelistCacheSnapshotEntry};
use crate::Storage;

const POLICY_KEYS: [&str; 3] = [
    "exec_control",
    "auto_read_control",
    "file_type_blacklist_control",
];

impl Storage {
    /// 在单个事务中导入策略数据。
    ///
    /// 所有输入均在开启事务和清表前完成校验。校验通过后，全量替换白名单、
    /// 三个文件访问策略开关和文件类型黑名单；任一数据库操作失败时全部回滚。
    ///
    /// 参数:
    /// - `whitelist`: 白名单条目列表。
    /// - `policies`: 文件访问策略列表，每项为 `(policy_key, enabled)`，`enabled` 仅允许 0/1。
    /// - `blacklist`: 黑名单条目列表，每项为 `(extension, description, is_default)`。
    ///
    /// 返回:
    /// - 成功时返回事务内构造的完整白名单缓存快照；
    /// - 校验或写入失败时返回错误。
    pub fn policy_import_transaction(
        &self,
        whitelist: &[UsbWhitelistInsert],
        policies: &[(String, i32)],
        blacklist: &[(String, Option<String>, i32)],
    ) -> Result<Vec<WhitelistCacheSnapshotEntry>, StorageError> {
        info!("开始策略数据导入事务");

        validate_whitelist(whitelist)?;
        validate_policies(policies)?;
        let normalized_blacklist = validate_and_normalize_blacklist(blacklist)?;

        debug!(whitelist_count = whitelist.len(), "导入白名单条目");
        debug!(policy_count = policies.len(), "导入策略条目");
        debug!(blacklist_count = normalized_blacklist.len(), "导入黑名单条目");

        let now = crate::now_unix();

        let whitelist_snapshot = self.pool().with_transaction(|conn| {
            conn.execute("DELETE FROM usb_whitelist", [])?;
            let mut whitelist_snapshot = Vec::with_capacity(whitelist.len());
            {
                let mut stmt = conn.prepare(
                    "INSERT INTO usb_whitelist \
                     (serial_number, vid, pid, device_name, capacity_bytes, device_type, \
                      description, permission, add_method, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                )?;
                for item in whitelist {
                    stmt.execute(params![
                        item.serial_number,
                        item.vid,
                        item.pid,
                        item.device_name,
                        item.capacity_bytes,
                        item.device_type,
                        item.description,
                        item.permission,
                        item.add_method,
                        now,
                    ])?;
                    whitelist_snapshot.push(WhitelistCacheSnapshotEntry {
                        id: conn.last_insert_rowid(),
                        serial_number: item.serial_number.clone(),
                        permission: item.permission,
                    });
                }
            }

            {
                let mut stmt = conn.prepare(
                    "UPDATE file_access_policy SET enabled = ?1, updated_at = ?2 \
                     WHERE policy_key = ?3",
                )?;
                for (policy_key, enabled) in policies {
                    let affected = stmt.execute(params![*enabled, now, policy_key])?;
                    if affected != 1 {
                        return Err(StorageError::NotFound(format!(
                            "file access policy key={policy_key}"
                        )));
                    }
                }
            }

            conn.execute("DELETE FROM file_type_blacklist", [])?;
            {
                let mut stmt = conn.prepare(
                    "INSERT INTO file_type_blacklist \
                     (extension, description, is_default, created_at) \
                     VALUES (?1, ?2, ?3, ?4)",
                )?;
                for (extension, description, is_default) in &normalized_blacklist {
                    stmt.execute(params![extension, description, is_default, now])?;
                }
            }

            debug!("策略导入事务提交");
            Ok(whitelist_snapshot)
        })?;

        info!("策略数据导入事务完成");
        Ok(whitelist_snapshot)
    }
}

fn validate_whitelist(whitelist: &[UsbWhitelistInsert]) -> Result<(), StorageError> {
    let mut serial_numbers = HashSet::with_capacity(whitelist.len());
    for item in whitelist {
        if item.serial_number.trim().is_empty() {
            return Err(StorageError::Validation("白名单序列号不能为空".into()));
        }
        if !matches!(item.permission, 0 | 1) {
            return Err(StorageError::Validation("白名单权限必须为 0 或 1".into()));
        }
        if !matches!(item.add_method, 0 | 1) {
            return Err(StorageError::Validation(
                "白名单添加方式必须为 0 或 1".into(),
            ));
        }
        if item.device_type != "storage" {
            return Err(StorageError::Validation(
                "白名单设备类型必须为 storage".into(),
            ));
        }
        if item.capacity_bytes.is_some_and(|capacity| capacity < 0) {
            return Err(StorageError::Validation("白名单容量不得为负数".into()));
        }
        if !serial_numbers.insert(item.serial_number.as_str()) {
            return Err(StorageError::Validation(format!(
                "白名单序列号重复: {}",
                item.serial_number
            )));
        }
    }
    Ok(())
}

fn validate_policies(policies: &[(String, i32)]) -> Result<(), StorageError> {
    let mut policy_keys = HashSet::with_capacity(policies.len());
    for (policy_key, enabled) in policies {
        if !matches!(*enabled, 0 | 1) {
            return Err(StorageError::Validation(
                "文件访问策略开关必须为 0 或 1".into(),
            ));
        }
        if !POLICY_KEYS.contains(&policy_key.as_str()) {
            return Err(StorageError::Validation(format!(
                "未知策略标识: {policy_key}"
            )));
        }
        if !policy_keys.insert(policy_key.as_str()) {
            return Err(StorageError::Validation(format!(
                "策略标识重复: {policy_key}"
            )));
        }
    }
    if policy_keys.len() != POLICY_KEYS.len() {
        let missing = POLICY_KEYS
            .iter()
            .find(|policy_key| !policy_keys.contains(*policy_key))
            .copied()
            .unwrap_or("unknown");
        return Err(StorageError::Validation(format!(
            "缺少策略标识: {missing}"
        )));
    }
    Ok(())
}

fn validate_and_normalize_blacklist(
    blacklist: &[(String, Option<String>, i32)],
) -> Result<Vec<(String, Option<String>, i32)>, StorageError> {
    let mut extensions = HashSet::with_capacity(blacklist.len());
    let mut normalized_blacklist = Vec::with_capacity(blacklist.len());
    for (extension, description, is_default) in blacklist {
        if !matches!(*is_default, 0 | 1) {
            return Err(StorageError::Validation(
                "黑名单默认标记必须为 0 或 1".into(),
            ));
        }
        let normalized = normalize_extension(extension)?;
        if !extensions.insert(normalized.clone()) {
            return Err(StorageError::Validation(format!(
                "文件后缀重复: {normalized}"
            )));
        }
        normalized_blacklist.push((normalized, description.clone(), *is_default));
    }
    Ok(normalized_blacklist)
}
