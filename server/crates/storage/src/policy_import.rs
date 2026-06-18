//! 策略导入事务操作。
//!
//! 在单个事务中清空并重写白名单、文件访问策略和文件类型黑名单，
//! 保证导入操作的原子性。

use rusqlite::params;

use crate::error::StorageError;
use crate::model::UsbWhitelistInsert;
use crate::Storage;

impl Storage {
    /// 在单个事务中导入策略数据。
    ///
    /// 操作步骤：
    /// 1. 清空白名单表并插入全部白名单条目
    /// 2. 更新文件访问控制策略开关
    /// 3. 清空自定义黑名单条目（保留内置默认），再插入导入数据中的自定义条目
    ///
    /// 参数:
    /// - `whitelist`: 白名单条目列表。
    /// - `policies`: 文件访问策略列表，每项为 `(policy_key, enabled)`。
    /// - `blacklist`: 黑名单条目列表，每项为 `(extension, description, is_default)`。
    ///
    /// 返回:
    /// - 成功时返回 `()`；失败时事务自动回滚。
    pub fn policy_import_transaction(
        &self,
        whitelist: &[UsbWhitelistInsert],
        policies: &[(String, bool)],
        blacklist: &[(String, Option<String>, i32)],
    ) -> Result<(), StorageError> {
        let now = crate::now_unix();

        self.pool().with_transaction(|conn| {
            // 1. 清空白名单并重新插入
            conn.execute("DELETE FROM usb_whitelist", [])?;

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
                }
            }

            // 2. 更新文件访问控制策略开关
            {
                let mut stmt = conn.prepare(
                    "UPDATE file_access_policy SET enabled = ?1, updated_at = ?2 \
                     WHERE policy_key = ?3",
                )?;
                for (policy_key, enabled) in policies {
                    let enabled_int: i32 = if *enabled { 1 } else { 0 };
                    stmt.execute(params![enabled_int, now, policy_key])?;
                }
            }

            // 3. 清空自定义黑名单条目，保留内置默认
            conn.execute(
                "DELETE FROM file_type_blacklist WHERE is_default = 0",
                [],
            )?;

            // 插入导入数据中的自定义条目
            {
                let mut stmt = conn.prepare(
                    "INSERT OR IGNORE INTO file_type_blacklist \
                     (extension, description, is_default, created_at) \
                     VALUES (?1, ?2, ?3, ?4)",
                )?;
                for (extension, description, is_default) in blacklist {
                    // 仅插入非默认条目（默认条目已存在于库中）
                    if *is_default == 0 {
                        stmt.execute(params![extension, description, is_default, now])?;
                    }
                }
            }

            Ok(())
        })
    }
}
