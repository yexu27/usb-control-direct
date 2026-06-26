//! T10 操作日志表 CRUD。

use rusqlite::params;
use tracing::{debug, trace};

use crate::error::StorageError;
use crate::model::{LogQueryParams, OperationLog, OperationLogInsert};
use crate::{escape_like_keyword, MAX_PAGE_SIZE, Storage};

impl Storage {
    /// 插入操作日志。
    pub fn operation_log_insert(&self, item: &OperationLogInsert) -> Result<i64, StorageError> {
        if item.username.is_empty() {
            return Err(StorageError::Validation("username 不能为空".into()));
        }
        if item.log_type.is_empty() {
            return Err(StorageError::Validation("log_type 不能为空".into()));
        }
        trace!(user = %item.username, action = ?item.action_type, "写入操作日志");
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
            let result: Result<i64, _> = tx.query_row(
                "DELETE FROM operation_log WHERE id = (SELECT id FROM operation_log ORDER BY op_time ASC LIMIT 1) RETURNING op_time",
                [], |row| row.get(0),
            );
            match result {
                Ok(time) => Ok(Some(time)),
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

    /// 多条件分页查询操作日志。
    ///
    /// 参数:
    ///   - `params`: 查询参数，包含时间范围、关键字、日志类型、操作类型和分页。
    ///
    /// 返回:
    ///   - `(Vec<OperationLog>, i64)`: (分页结果, 符合条件的总数)。
    pub fn operation_log_query_paged(
        &self,
        params: &LogQueryParams,
    ) -> Result<(Vec<OperationLog>, i64), StorageError> {
        debug!(page = params.page, page_size = params.page_size, "分页查询操作日志");
        self.pool().with_read(|conn| {
            let mut conditions = Vec::new();
            let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(start) = params.start_time {
                if start > 0 {
                    conditions.push(format!("op_time >= ?{}", bind_values.len() + 1));
                    bind_values.push(Box::new(start));
                }
            }
            if let Some(end) = params.end_time {
                if end > 0 {
                    conditions.push(format!("op_time <= ?{}", bind_values.len() + 1));
                    bind_values.push(Box::new(end));
                }
            }
            if let Some(ref keyword) = params.keyword {
                if !keyword.is_empty() {
                    let idx = bind_values.len() + 1;
                    conditions.push(format!(
                        "(username LIKE ?{idx} ESCAPE '\\' OR target LIKE ?{idx} ESCAPE '\\' OR detail LIKE ?{idx} ESCAPE '\\')"
                    ));
                    bind_values.push(Box::new(escape_like_keyword(keyword)));
                }
            }

            let where_clause = if conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", conditions.join(" AND "))
            };

            // 查总数
            let count_sql = format!("SELECT COUNT(*) FROM operation_log {}", where_clause);
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
                "SELECT id, op_time, username, role, log_type, action_type, target, \
                 before_value, after_value, related_file, related_version, result, fail_reason, \
                 source_ip, app_version, session_id, request_id, detail \
                 FROM operation_log {} ORDER BY op_time DESC LIMIT ?{} OFFSET ?{}",
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
                Ok(OperationLog {
                    id: row.get(0)?,
                    op_time: row.get(1)?,
                    username: row.get(2)?,
                    role: row.get(3)?,
                    log_type: row.get(4)?,
                    action_type: row.get(5)?,
                    target: row.get(6)?,
                    before_value: row.get(7)?,
                    after_value: row.get(8)?,
                    related_file: row.get(9)?,
                    related_version: row.get(10)?,
                    result: row.get(11)?,
                    fail_reason: row.get(12)?,
                    source_ip: row.get(13)?,
                    app_version: row.get(14)?,
                    session_id: row.get(15)?,
                    request_id: row.get(16)?,
                    detail: row.get(17)?,
                })
            })?;
            let results = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(StorageError::Sqlite)?;
            Ok((results, total))
        })
    }

    /// 删除指定时间范围内的操作日志。
    ///
    /// 参数:
    ///   - `start_time`: 起始时间（含），Unix 秒级时间戳。
    ///   - `end_time`: 结束时间（含），Unix 秒级时间戳。
    ///
    /// 返回:
    ///   - 实际删除的记录数。
    pub fn operation_log_delete_by_time(
        &self,
        start_time: i64,
        end_time: i64,
    ) -> Result<i64, StorageError> {
        debug!(start = start_time, end = end_time, "按时间范围删除操作日志");
        self.pool().with_transaction(|tx| {
            let affected = tx.execute(
                "DELETE FROM operation_log WHERE op_time >= ?1 AND op_time <= ?2",
                params![start_time, end_time],
            )?;
            Ok(affected as i64)
        })
    }
}
