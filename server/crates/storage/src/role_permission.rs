//! T09 角色权限表 CRUD。

use rusqlite::params;

use crate::error::StorageError;
use crate::model::RolePermission;
use crate::Storage;

impl Storage {
    /// 查询指定角色的权限列表。
    pub fn role_permission_query_by_role(&self, role: i32) -> Result<Vec<RolePermission>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, role, page_key FROM role_permission WHERE role = ?1 ORDER BY id",
            )?;
            let rows = stmt.query_map(params![role], |row| {
                Ok(RolePermission {
                    id: row.get(0)?,
                    role: row.get(1)?,
                    page_key: row.get(2)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 查询全部角色权限。
    pub fn role_permission_query_all(&self) -> Result<Vec<RolePermission>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, role, page_key FROM role_permission ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(RolePermission {
                    id: row.get(0)?,
                    role: row.get(1)?,
                    page_key: row.get(2)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }
}
