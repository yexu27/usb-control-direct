//! T10 操作日志表 CRUD。

use rusqlite::params;

use crate::error::StorageError;
use crate::model::{OperationLog, OperationLogInsert};
use crate::Storage;

impl Storage {
    /// 插入操作日志。
    pub fn operation_log_insert(&self, item: &OperationLogInsert) -> Result<i64, StorageError> {
        if item.username.is_empty() {
            return Err(StorageError::Validation("username 不能为空".into()));
        }
        if item.log_type.is_empty() {
            return Err(StorageError::Validation("log_type 不能为空".into()));
        }
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO operation_log (op_time, username, role, log_type, action_type, target, before_value, after_value, related_file, related_version, result, fail_reason, source_ip, app_version, session_id, request_id, detail) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    item.op_time, item.username, item.role, item.log_type,
                    item.action_type, item.target, item.before_value, item.after_value,
                    item.related_file, item.related_version, item.result,
                    item.fail_reason, item.source_ip, item.app_version,
                    item.session_id, item.request_id, item.detail,
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 按时间范围查询操作日志。
    pub fn operation_log_query_by_time(&self, from: i64, to: i64) -> Result<Vec<OperationLog>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, op_time, username, role, log_type, action_type, target, before_value, after_value, related_file, related_version, result, fail_reason, source_ip, app_version, session_id, request_id, detail \
                 FROM operation_log WHERE op_time >= ?1 AND op_time <= ?2 ORDER BY op_time DESC",
            )?;
            let rows = stmt.query_map(params![from, to], |row| {
                Ok(OperationLog {
                    id: row.get(0)?, op_time: row.get(1)?, username: row.get(2)?,
                    role: row.get(3)?, log_type: row.get(4)?, action_type: row.get(5)?,
                    target: row.get(6)?, before_value: row.get(7)?, after_value: row.get(8)?,
                    related_file: row.get(9)?, related_version: row.get(10)?,
                    result: row.get(11)?, fail_reason: row.get(12)?,
                    source_ip: row.get(13)?, app_version: row.get(14)?,
                    session_id: row.get(15)?, request_id: row.get(16)?, detail: row.get(17)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 删除最旧的一条操作日志，返回被删除记录的时间。
    pub fn operation_log_delete_oldest(&self) -> Result<Option<i64>, StorageError> {
        self.pool().with_transaction(|tx| {
            let oldest: Result<i64, _> = tx.query_row(
                "SELECT op_time FROM operation_log ORDER BY op_time ASC LIMIT 1",
                [], |row| row.get(0),
            );
            match oldest {
                Ok(time) => {
                    tx.execute(
                        "DELETE FROM operation_log WHERE id = (SELECT id FROM operation_log ORDER BY op_time ASC LIMIT 1)",
                        [],
                    )?;
                    Ok(Some(time))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }

    /// 操作日志总数。
    pub fn operation_log_count(&self) -> Result<i64, StorageError> {
        self.pool().with_read(|conn| {
            let count = conn.query_row("SELECT COUNT(*) FROM operation_log", [], |row| row.get(0))?;
            Ok(count)
        })
    }
}
