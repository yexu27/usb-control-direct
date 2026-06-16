//! T02 文件类型黑名单表 CRUD。

use rusqlite::params;

use crate::error::StorageError;
use crate::model::FileTypeBlacklist;
use crate::Storage;

impl Storage {
    /// 添加文件类型黑名单条目。
    pub fn blacklist_insert(&self, extension: &str, description: Option<&str>) -> Result<i64, StorageError> {
        if extension.is_empty() {
            return Err(StorageError::Validation("extension 不能为空".into()));
        }
        let ext_lower = extension.to_lowercase();
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO file_type_blacklist (extension, description, is_default, created_at) VALUES (?1, ?2, 0, ?3)",
                params![ext_lower, description, now],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 按后缀查询黑名单条目。
    pub fn blacklist_query_by_ext(&self, extension: &str) -> Result<Option<FileTypeBlacklist>, StorageError> {
        let ext_lower = extension.to_lowercase();
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, extension, description, is_default, created_at FROM file_type_blacklist WHERE extension = ?1",
            )?;
            let result = stmt.query_row(params![ext_lower], |row| {
                Ok(FileTypeBlacklist {
                    id: row.get(0)?,
                    extension: row.get(1)?,
                    description: row.get(2)?,
                    is_default: row.get(3)?,
                    created_at: row.get(4)?,
                })
            });
            match result {
                Ok(item) => Ok(Some(item)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }

    /// 查询全部黑名单。
    pub fn blacklist_query_all(&self) -> Result<Vec<FileTypeBlacklist>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, extension, description, is_default, created_at FROM file_type_blacklist ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(FileTypeBlacklist {
                    id: row.get(0)?,
                    extension: row.get(1)?,
                    description: row.get(2)?,
                    is_default: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 删除黑名单条目（is_default=1 的不可删除）。
    pub fn blacklist_delete(&self, id: i64) -> Result<(), StorageError> {
        self.pool().with_transaction(|tx| {
            let is_default: i32 = tx
                .query_row("SELECT is_default FROM file_type_blacklist WHERE id = ?1", params![id], |row| row.get(0))
                .map_err(|_| StorageError::NotFound(format!("blacklist id={}", id)))?;
            if is_default == 1 {
                return Err(StorageError::Validation("内置默认后缀不可删除".into()));
            }
            tx.execute("DELETE FROM file_type_blacklist WHERE id = ?1", params![id])?;
            Ok(())
        })
    }

    /// 黑名单总数。
    pub fn blacklist_count(&self) -> Result<i64, StorageError> {
        self.pool().with_read(|conn| {
            let count = conn.query_row("SELECT COUNT(*) FROM file_type_blacklist", [], |row| row.get(0))?;
            Ok(count)
        })
    }
}
