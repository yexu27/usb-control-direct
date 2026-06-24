//! T08 用户表 CRUD。

use rusqlite::params;
use tracing::debug;

use crate::error::StorageError;
use crate::model::{User, UserInsert};
use crate::Storage;

impl Storage {
    /// 创建用户。
    pub fn user_insert(&self, item: &UserInsert) -> Result<i64, StorageError> {
        if item.username.is_empty() {
            return Err(StorageError::Validation("username 不能为空".into()));
        }
        if item.password_hash.is_empty() {
            return Err(StorageError::Validation("password_hash 不能为空".into()));
        }
        debug!(user = %item.username, "插入用户");
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO users (username, password_hash, role, status, is_builtin, login_fail_count, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, 0, 0, 0, ?4, ?4)",
                params![item.username, item.password_hash, item.role, now],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 按用户名查询用户。
    pub fn user_query_by_username(&self, username: &str) -> Result<Option<User>, StorageError> {
        debug!(user = %username, "查询用户");
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, username, password_hash, role, status, is_builtin, login_fail_count, lock_until, created_at, updated_at, password_changed_at \
                 FROM users WHERE username = ?1",
            )?;
            let result = stmt.query_row(params![username], |row| {
                Ok(User {
                    id: row.get(0)?, username: row.get(1)?, password_hash: row.get(2)?,
                    role: row.get(3)?, status: row.get(4)?, is_builtin: row.get(5)?,
                    login_fail_count: row.get(6)?, lock_until: row.get(7)?,
                    created_at: row.get(8)?, updated_at: row.get(9)?,
                    password_changed_at: row.get(10)?,
                })
            });
            match result {
                Ok(u) => Ok(Some(u)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }

    /// 查询全部活跃用户（status != 2）。
    pub fn user_query_active(&self) -> Result<Vec<User>, StorageError> {
        debug!("查询活跃用户列表");
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, username, password_hash, role, status, is_builtin, login_fail_count, lock_until, created_at, updated_at, password_changed_at \
                 FROM users WHERE status != 2 ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(User {
                    id: row.get(0)?, username: row.get(1)?, password_hash: row.get(2)?,
                    role: row.get(3)?, status: row.get(4)?, is_builtin: row.get(5)?,
                    login_fail_count: row.get(6)?, lock_until: row.get(7)?,
                    created_at: row.get(8)?, updated_at: row.get(9)?,
                    password_changed_at: row.get(10)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 更新用户密码。
    pub fn user_update_password(&self, id: i64, password_hash: &str) -> Result<(), StorageError> {
        debug!(user_id = id, "更新用户密码");
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            let affected = tx.execute(
                "UPDATE users SET password_hash = ?1, updated_at = ?2, password_changed_at = ?2 WHERE id = ?3",
                params![password_hash, now, id],
            )?;
            if affected == 0 {
                return Err(StorageError::NotFound(format!("user id={}", id)));
            }
            Ok(())
        })
    }

    /// 更新登录失败计数和锁定时间。
    pub fn user_update_login_fail(&self, id: i64, fail_count: i32, lock_until: Option<i64>) -> Result<(), StorageError> {
        debug!(user_id = id, fail_count = fail_count, "更新登录失败计数");
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            let affected = tx.execute(
                "UPDATE users SET login_fail_count = ?1, lock_until = ?2, updated_at = ?3 WHERE id = ?4",
                params![fail_count, lock_until, now, id],
            )?;
            if affected == 0 {
                return Err(StorageError::NotFound(format!("user id={}", id)));
            }
            Ok(())
        })
    }

    /// 软删除用户（status=2）。
    pub fn user_soft_delete(&self, id: i64) -> Result<(), StorageError> {
        debug!(user_id = id, "软删除用户");
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            let affected = tx.execute(
                "UPDATE users SET status = 2, updated_at = ?1 WHERE id = ?2 AND is_builtin = 0",
                params![now, id],
            )?;
            if affected == 0 {
                return Err(StorageError::NotFound(format!("user id={} (可能为内置用户)", id)));
            }
            Ok(())
        })
    }

    /// 用户总数（不含已删除）。
    pub fn user_count_active(&self) -> Result<i64, StorageError> {
        self.pool().with_read(|conn| {
            let count = conn.query_row("SELECT COUNT(*) FROM users WHERE status != 2", [], |row| row.get(0))?;
            Ok(count)
        })
    }
}
