//! T11 日志覆盖追溯表 CRUD。

use rusqlite::params;

use crate::error::StorageError;
use crate::model::{LogRetentionEvent, LogRetentionEventInsert};
use crate::Storage;

impl Storage {
    /// 插入日志覆盖追溯记录。
    pub fn retention_event_insert(&self, item: &LogRetentionEventInsert) -> Result<i64, StorageError> {
        if item.log_category.is_empty() {
            return Err(StorageError::Validation("log_category 不能为空".into()));
        }
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO log_retention_event (trigger_time, log_category, storage_usage_percent, covered_from_time, covered_to_time, covered_count, result, fail_reason) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    item.trigger_time, item.log_category, item.storage_usage_percent,
                    item.covered_from_time, item.covered_to_time, item.covered_count,
                    item.result, item.fail_reason,
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 按时间范围查询覆盖追溯记录。
    pub fn retention_event_query_by_time(&self, from: i64, to: i64) -> Result<Vec<LogRetentionEvent>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, trigger_time, log_category, storage_usage_percent, covered_from_time, covered_to_time, covered_count, result, fail_reason \
                 FROM log_retention_event WHERE trigger_time >= ?1 AND trigger_time <= ?2 ORDER BY trigger_time DESC",
            )?;
            let rows = stmt.query_map(params![from, to], |row| {
                Ok(LogRetentionEvent {
                    id: row.get(0)?, trigger_time: row.get(1)?, log_category: row.get(2)?,
                    storage_usage_percent: row.get(3)?, covered_from_time: row.get(4)?,
                    covered_to_time: row.get(5)?, covered_count: row.get(6)?,
                    result: row.get(7)?, fail_reason: row.get(8)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }
}
