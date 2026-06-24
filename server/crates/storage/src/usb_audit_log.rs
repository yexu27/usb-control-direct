//! T05 USB 审计日志表 CRUD。

use rusqlite::params;
use tracing::{debug, trace};

use crate::error::StorageError;
use crate::model::{LogQueryParams, UsbAuditLog, UsbAuditLogInsert};
use crate::{escape_like_keyword, MAX_PAGE_SIZE, Storage};

impl Storage {
    /// 插入 USB 审计日志。
    pub fn usb_audit_insert(&self, item: &UsbAuditLogInsert) -> Result<i64, StorageError> {
        if item.event_type.is_empty() {
            return Err(StorageError::Validation("event_type 不能为空".into()));
        }
        if item.result.is_empty() {
            return Err(StorageError::Validation("result 不能为空".into()));
        }
        trace!(event_type = %item.event_type, device_sn = ?item.device_sn, "写入 USB 审计日志");
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO usb_audit_log (event_time, device_type, interface_type, interface_class, interface_subclass, interface_protocol, device_name, device_sn, vid, pid, event_type, permission, capacity_bytes, file_path, matched_policy, result, fail_reason, detail) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    item.event_time, item.device_type, item.interface_type,
                    item.interface_class, item.interface_subclass, item.interface_protocol,
                    item.device_name, item.device_sn, item.vid, item.pid,
                    item.event_type, item.permission, item.capacity_bytes,
                    item.file_path, item.matched_policy, item.result,
                    item.fail_reason, item.detail,
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 按时间范围查询 USB 审计日志。
    pub fn usb_audit_query_by_time(&self, from: i64, to: i64) -> Result<Vec<UsbAuditLog>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, event_time, device_type, interface_type, interface_class, interface_subclass, interface_protocol, device_name, device_sn, vid, pid, event_type, permission, capacity_bytes, file_path, matched_policy, result, fail_reason, detail \
                 FROM usb_audit_log WHERE event_time >= ?1 AND event_time <= ?2 ORDER BY event_time DESC",
            )?;
            let rows = stmt.query_map(params![from, to], |row| {
                Ok(UsbAuditLog {
                    id: row.get(0)?, event_time: row.get(1)?, device_type: row.get(2)?,
                    interface_type: row.get(3)?, interface_class: row.get(4)?,
                    interface_subclass: row.get(5)?, interface_protocol: row.get(6)?,
                    device_name: row.get(7)?, device_sn: row.get(8)?,
                    vid: row.get(9)?, pid: row.get(10)?, event_type: row.get(11)?,
                    permission: row.get(12)?, capacity_bytes: row.get(13)?,
                    file_path: row.get(14)?, matched_policy: row.get(15)?,
                    result: row.get(16)?, fail_reason: row.get(17)?, detail: row.get(18)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 删除最旧的一条 USB 审计日志，返回被删除记录的时间。
    pub fn usb_audit_delete_oldest(&self) -> Result<Option<i64>, StorageError> {
        self.pool().with_transaction(|tx| {
            let result: Result<i64, _> = tx.query_row(
                "DELETE FROM usb_audit_log WHERE id = (SELECT id FROM usb_audit_log ORDER BY event_time ASC LIMIT 1) RETURNING event_time",
                [], |row| row.get(0),
            );
            match result {
                Ok(time) => Ok(Some(time)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }

    /// USB 审计日志总数。
    pub fn usb_audit_count(&self) -> Result<i64, StorageError> {
        self.pool().with_read(|conn| {
            let count = conn.query_row("SELECT COUNT(*) FROM usb_audit_log", [], |row| row.get(0))?;
            Ok(count)
        })
    }

    /// 多条件分页查询 USB 审计日志。
    ///
    /// 参数:
    ///   - `params`: 查询参数，包含时间范围、关键字、事件类型和分页。
    ///
    /// 返回:
    ///   - `(Vec<UsbAuditLog>, i64)`: (分页结果, 符合条件的总数)。
    pub fn usb_audit_query_paged(
        &self,
        params: &LogQueryParams,
    ) -> Result<(Vec<UsbAuditLog>, i64), StorageError> {
        debug!(page = params.page, page_size = params.page_size, "分页查询 USB 审计日志");
        self.pool().with_read(|conn| {
            let mut conditions = Vec::new();
            let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(start) = params.start_time {
                if start > 0 {
                    conditions.push(format!("event_time >= ?{}", bind_values.len() + 1));
                    bind_values.push(Box::new(start));
                }
            }
            if let Some(end) = params.end_time {
                if end > 0 {
                    conditions.push(format!("event_time <= ?{}", bind_values.len() + 1));
                    bind_values.push(Box::new(end));
                }
            }
            if let Some(ref keyword) = params.keyword {
                if !keyword.is_empty() {
                    let idx = bind_values.len() + 1;
                    conditions.push(format!(
                        "(device_name LIKE ?{idx} ESCAPE '\\' OR device_sn LIKE ?{idx} ESCAPE '\\' OR detail LIKE ?{idx} ESCAPE '\\')"
                    ));
                    bind_values.push(Box::new(escape_like_keyword(keyword)));
                }
            }
            if let Some(ref event_type) = params.event_type {
                if !event_type.is_empty() {
                    conditions.push(format!("event_type = ?{}", bind_values.len() + 1));
                    bind_values.push(Box::new(event_type.clone()));
                }
            }

            let where_clause = if conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", conditions.join(" AND "))
            };

            // 查总数
            let count_sql = format!("SELECT COUNT(*) FROM usb_audit_log {}", where_clause);
            let params_ref: Vec<&dyn rusqlite::types::ToSql> =
                bind_values.iter().map(|b| b.as_ref()).collect();
            let total: i64 =
                conn.query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))?;

            // 分页查询
            let page = if params.page < 1 { 1 } else { params.page };
            let page_size = if params.page_size < 1 {
                50
            } else {
                params.page_size.min(MAX_PAGE_SIZE)
            };
            let offset = (page - 1) as i64 * page_size as i64;

            let query_sql = format!(
                "SELECT id, event_time, device_type, interface_type, interface_class, \
                 interface_subclass, interface_protocol, device_name, device_sn, vid, pid, \
                 event_type, permission, capacity_bytes, file_path, matched_policy, result, \
                 fail_reason, detail \
                 FROM usb_audit_log {} ORDER BY event_time DESC LIMIT ?{} OFFSET ?{}",
                where_clause,
                bind_values.len() + 1,
                bind_values.len() + 2
            );

            let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = bind_values;
            all_params.push(Box::new(page_size as i64));
            all_params.push(Box::new(offset));
            let all_ref: Vec<&dyn rusqlite::types::ToSql> =
                all_params.iter().map(|b| b.as_ref()).collect();

            let mut stmt = conn.prepare(&query_sql)?;
            let rows = stmt.query_map(all_ref.as_slice(), |row| {
                Ok(UsbAuditLog {
                    id: row.get(0)?,
                    event_time: row.get(1)?,
                    device_type: row.get(2)?,
                    interface_type: row.get(3)?,
                    interface_class: row.get(4)?,
                    interface_subclass: row.get(5)?,
                    interface_protocol: row.get(6)?,
                    device_name: row.get(7)?,
                    device_sn: row.get(8)?,
                    vid: row.get(9)?,
                    pid: row.get(10)?,
                    event_type: row.get(11)?,
                    permission: row.get(12)?,
                    capacity_bytes: row.get(13)?,
                    file_path: row.get(14)?,
                    matched_policy: row.get(15)?,
                    result: row.get(16)?,
                    fail_reason: row.get(17)?,
                    detail: row.get(18)?,
                })
            })?;
            let results = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(StorageError::Sqlite)?;
            Ok((results, total))
        })
    }

    /// 删除指定时间范围内的 USB 审计日志。
    ///
    /// 参数:
    ///   - `start_time`: 起始时间（含），Unix 秒级时间戳。
    ///   - `end_time`: 结束时间（含），Unix 秒级时间戳。
    ///
    /// 返回:
    ///   - 实际删除的记录数。
    pub fn usb_audit_delete_by_time(
        &self,
        start_time: i64,
        end_time: i64,
    ) -> Result<i64, StorageError> {
        debug!(start = start_time, end = end_time, "按时间范围删除 USB 审计日志");
        self.pool().with_transaction(|tx| {
            let affected = tx.execute(
                "DELETE FROM usb_audit_log WHERE event_time >= ?1 AND event_time <= ?2",
                params![start_time, end_time],
            )?;
            Ok(affected as i64)
        })
    }
}
