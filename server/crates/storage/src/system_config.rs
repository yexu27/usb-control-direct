//! T07 系统配置表 CRUD。

use rusqlite::params;
use tracing::debug;

use crate::error::StorageError;
use crate::model::SystemConfig;
use crate::Storage;

impl Storage {
    /// 获取配置值。
    pub fn config_get(&self, config_key: &str) -> Result<Option<SystemConfig>, StorageError> {
        debug!(key = %config_key, "读取系统配置");
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT config_key, config_value, updated_at FROM system_config WHERE config_key = ?1",
            )?;
            let result = stmt.query_row(params![config_key], |row| {
                Ok(SystemConfig {
                    config_key: row.get(0)?,
                    config_value: row.get(1)?,
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

    /// 设置配置值（upsert）。
    pub fn config_set(&self, config_key: &str, config_value: &str) -> Result<(), StorageError> {
        if config_key.is_empty() {
            return Err(StorageError::Validation("config_key 不能为空".into()));
        }
        debug!(key = %config_key, "写入系统配置");
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO system_config (config_key, config_value, updated_at) VALUES (?1, ?2, ?3) \
                 ON CONFLICT(config_key) DO UPDATE SET config_value = ?2, updated_at = ?3",
                params![config_key, config_value, now],
            )?;
            Ok(())
        })
    }

    /// 查询全部配置。
    pub fn config_query_all(&self) -> Result<Vec<SystemConfig>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT config_key, config_value, updated_at FROM system_config ORDER BY config_key",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(SystemConfig {
                    config_key: row.get(0)?,
                    config_value: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }
}
