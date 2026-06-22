//! T02 文件类型黑名单表 CRUD。

use rusqlite::params;

use crate::error::StorageError;
use crate::extension::normalize_extension;
use crate::model::FileTypeBlacklist;
use crate::Storage;

impl Storage {
    /// 添加文件类型黑名单条目。
    pub fn blacklist_insert(
        &self,
        extension: &str,
        description: Option<&str>,
    ) -> Result<i64, StorageError> {
        let normalized = normalize_extension(extension)?;
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO file_type_blacklist (extension, description, is_default, created_at) VALUES (?1, ?2, 0, ?3)",
                params![normalized, description, now],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 按后缀查询黑名单条目。
    pub fn blacklist_query_by_ext(
        &self,
        extension: &str,
    ) -> Result<Option<FileTypeBlacklist>, StorageError> {
        let normalized = normalize_extension(extension)?;
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, extension, description, is_default, created_at FROM file_type_blacklist WHERE extension = ?1",
            )?;
            let result = stmt.query_row(params![normalized], |row| {
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

    /// 按规范化后缀删除黑名单条目（is_default=1 的不可删除）。
    pub fn blacklist_delete(&self, extension: &str) -> Result<(), StorageError> {
        let normalized = normalize_extension(extension)?;
        self.pool().with_transaction(|tx| {
            let is_default: i32 = tx
                .query_row(
                    "SELECT is_default FROM file_type_blacklist WHERE extension = ?1",
                    params![normalized],
                    |row| row.get(0),
                )
                .map_err(|error| match error {
                    rusqlite::Error::QueryReturnedNoRows => {
                        StorageError::NotFound(format!("blacklist extension={normalized}"))
                    }
                    other => StorageError::Sqlite(other),
                })?;
            if is_default == 1 {
                return Err(StorageError::Validation("内置默认后缀不可删除".into()));
            }
            tx.execute(
                "DELETE FROM file_type_blacklist WHERE extension = ?1",
                params![normalized],
            )?;
            Ok(())
        })
    }

    /// 黑名单总数。
    pub fn blacklist_count(&self) -> Result<i64, StorageError> {
        self.pool().with_read(|conn| {
            let count = conn.query_row(
                "SELECT COUNT(*) FROM file_type_blacklist",
                [],
                |row| row.get(0),
            )?;
            Ok(count)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn setup() -> (Storage, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let storage = Storage::open(file.path()).unwrap();
        (storage, file)
    }

    #[test]
    fn insert_and_query_normalize_extension() {
        let (storage, _file) = setup();
        storage
            .blacklist_insert("  .TEST_Ext  ", Some("测试后缀"))
            .unwrap();

        let item = storage
            .blacklist_query_by_ext(".test_EXT")
            .unwrap()
            .unwrap();
        assert_eq!(item.extension, ".test_ext");
    }

    #[test]
    fn insert_rejects_invalid_extension() {
        let (storage, _file) = setup();
        assert!(matches!(
            storage.blacklist_insert("test", None),
            Err(StorageError::Validation(_))
        ));
    }

    #[test]
    fn delete_by_extension_normalizes_and_keeps_default_protection() {
        let (storage, _file) = setup();
        storage.blacklist_insert(".custom", None).unwrap();
        storage.blacklist_delete("  .CUSTOM ").unwrap();
        assert!(storage
            .blacklist_query_by_ext(".custom")
            .unwrap()
            .is_none());

        assert!(matches!(
            storage.blacklist_delete(".PS1"),
            Err(StorageError::Validation(_))
        ));
    }
}
