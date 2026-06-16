//! T04 可执行程序控制表 CRUD。

use crate::error::StorageError;
use crate::model::ExecType;
use crate::Storage;

impl Storage {
    /// 查询全部可执行程序类型。
    pub fn exec_type_query_all(&self) -> Result<Vec<ExecType>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, type_name, description FROM exec_type ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ExecType {
                    id: row.get(0)?,
                    type_name: row.get(1)?,
                    description: row.get(2)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 按类型名查询。
    pub fn exec_type_query_by_name(&self, type_name: &str) -> Result<Option<ExecType>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, type_name, description FROM exec_type WHERE type_name = ?1",
            )?;
            let result = stmt.query_row(rusqlite::params![type_name], |row| {
                Ok(ExecType {
                    id: row.get(0)?,
                    type_name: row.get(1)?,
                    description: row.get(2)?,
                })
            });
            match result {
                Ok(item) => Ok(Some(item)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }
}
