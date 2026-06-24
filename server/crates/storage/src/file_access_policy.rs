//! T03 文件访问控制开关表 CRUD。

use rusqlite::params;
use tracing::debug;

use crate::error::StorageError;
use crate::model::FileAccessPolicy;
use crate::Storage;

impl Storage {
    /// 查询指定策略开关。
    pub fn policy_query(&self, policy_key: &str) -> Result<Option<FileAccessPolicy>, StorageError> {
        debug!(key = %policy_key, "查询文件策略配置");
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT policy_key, enabled, updated_at FROM file_access_policy WHERE policy_key = ?1",
            )?;
            let result = stmt.query_row(params![policy_key], |row| {
                Ok(FileAccessPolicy {
                    policy_key: row.get(0)?,
                    enabled: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            });
            match result {
                Ok(item) => Ok(Some(item)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }

    /// 查询全部策略开关。
    pub fn policy_query_all(&self) -> Result<Vec<FileAccessPolicy>, StorageError> {
        debug!("查询全部策略开关");
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT policy_key, enabled, updated_at FROM file_access_policy ORDER BY policy_key",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(FileAccessPolicy {
                    policy_key: row.get(0)?,
                    enabled: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 更新策略开关状态。
    pub fn policy_update(&self, policy_key: &str, enabled: bool) -> Result<(), StorageError> {
        if policy_key.is_empty() {
            return Err(StorageError::Validation("policy_key 不能为空".into()));
        }
        debug!(key = %policy_key, enabled = enabled, "更新文件策略开关");
        let now = crate::now_unix();
        let enabled_int = if enabled { 1 } else { 0 };
        self.pool().with_transaction(|tx| {
            let affected = tx.execute(
                "UPDATE file_access_policy SET enabled = ?1, updated_at = ?2 WHERE policy_key = ?3",
                params![enabled_int, now, policy_key],
            )?;
            if affected == 0 {
                return Err(StorageError::NotFound(format!("policy_key={}", policy_key)));
            }
            Ok(())
        })
    }
}
