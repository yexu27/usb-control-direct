//! SQLite 连接池。
//!
//! 采用"单写多读"模型：
//! - 写连接唯一，通过 `Mutex<Connection>` 串行化所有写操作；
//! - 读连接池固定大小，通过 `Mutex<Vec<Connection>>` 管理，池空时临时创建连接。

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OpenFlags};

use crate::StorageError;

/// SQLite 连接池。
///
/// 持有一个写连接和若干个只读连接。调用方通过
/// [`Pool::with_write`]、[`Pool::with_read`]、[`Pool::with_transaction`]
/// 执行数据库操作，连接管理细节对外透明。
pub struct Pool {
    /// 写连接（串行化所有写操作）。
    write_conn: Mutex<Connection>,
    /// 只读连接池（可复用的读连接列表）。
    read_pool: Mutex<Vec<Connection>>,
    /// 数据库文件路径，用于池空时临时创建读连接。
    db_path: String,
}

/// 连接池默认只读连接数量。
const DEFAULT_READ_POOL_SIZE: usize = 4;

impl Pool {
    /// 打开数据库文件并初始化连接池。
    ///
    /// 参数:
    /// - `path`: 数据库文件路径；不存在时自动创建。
    /// - `read_pool_size`: 只读连接池大小；传 `0` 时使用默认值 `4`。
    ///
    /// 返回:
    /// - 成功时返回初始化完成的连接池；失败时返回 [`StorageError`]。
    pub fn open(path: impl AsRef<Path>, read_pool_size: usize) -> Result<Self, StorageError> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| StorageError::Validation("数据库路径包含非 UTF-8 字符".into()))?
            .to_owned();

        let pool_size = if read_pool_size == 0 {
            DEFAULT_READ_POOL_SIZE
        } else {
            read_pool_size
        };

        // 初始化写连接
        let write_conn = Self::open_write_connection(&path_str)?;

        // 初始化只读连接池
        let mut read_conns = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            let conn = Self::open_read_connection(&path_str)?;
            read_conns.push(conn);
        }

        Ok(Self {
            write_conn: Mutex::new(write_conn),
            read_pool: Mutex::new(read_conns),
            db_path: path_str,
        })
    }

    /// 通过写连接执行闭包。
    ///
    /// 持有写连接锁期间执行 `f`，操作串行化，适用于 INSERT / UPDATE / DELETE。
    ///
    /// 参数:
    /// - `f`: 接受 `&Connection` 引用的闭包，返回 `Result<T, StorageError>`。
    ///
    /// 返回:
    /// - 闭包的返回值。
    #[allow(dead_code)]
    pub fn with_write<T, F>(&self, f: F) -> Result<T, StorageError>
    where
        F: FnOnce(&Connection) -> Result<T, StorageError>,
    {
        let conn = self.lock_write()?;
        f(&conn)
    }

    /// 通过只读连接执行闭包。
    ///
    /// 优先从连接池取出一个连接，用完后放回；池为空时临时创建连接（不放回池）。
    ///
    /// 参数:
    /// - `f`: 接受 `&Connection` 引用的闭包，返回 `Result<T, StorageError>`。
    ///
    /// 返回:
    /// - 闭包的返回值。
    pub fn with_read<T, F>(&self, f: F) -> Result<T, StorageError>
    where
        F: FnOnce(&Connection) -> Result<T, StorageError>,
    {
        // 尝试从池中取一个连接
        let pooled = {
            let mut pool = self
                .read_pool
                .lock()
                .map_err(|_| StorageError::Validation("读连接池锁中毒".into()))?;
            pool.pop()
        };

        let (conn, from_pool) = match pooled {
            Some(c) => (c, true),
            None => {
                // 池为空，临时创建一个
                let c = Self::open_read_connection(&self.db_path)?;
                (c, false)
            }
        };

        let result = f(&conn);

        // 只将来自池的连接归还
        if from_pool {
            let mut pool = self
                .read_pool
                .lock()
                .map_err(|_| StorageError::Validation("读连接池锁中毒".into()))?;
            pool.push(conn);
        }

        result
    }

    /// 通过写连接执行事务。
    ///
    /// 使用 `rusqlite::Transaction` 管理事务生命周期：
    /// 闭包返回 `Ok` 时自动 COMMIT，返回 `Err` 或 panic 时通过 Drop 自动 ROLLBACK。
    ///
    /// 参数:
    /// - `f`: 接受 `&Connection` 引用的闭包，返回 `Result<T, StorageError>`。
    ///
    /// 返回:
    /// - 事务提交后闭包的返回值；或事务回滚后的错误。
    pub fn with_transaction<T, F>(&self, f: F) -> Result<T, StorageError>
    where
        F: FnOnce(&Connection) -> Result<T, StorageError>,
    {
        let mut conn = self.lock_write()?;
        let tx = conn.transaction()?;
        let result = f(&tx);
        match result {
            Ok(value) => {
                tx.commit()?;
                Ok(value)
            }
            Err(err) => {
                // tx 在此处 Drop，自动 ROLLBACK
                Err(err)
            }
        }
    }

    // ------------------------------------------------------------------ 私有方法

    /// 锁定写连接，返回 `MutexGuard`。
    fn lock_write(&self) -> Result<MutexGuard<'_, Connection>, StorageError> {
        self.write_conn
            .lock()
            .map_err(|_| StorageError::Validation("写连接锁中毒".into()))
    }

    /// 打开并初始化写连接。
    fn open_write_connection(path: &str) -> Result<Connection, StorageError> {
        let conn = Connection::open(path)?;
        // WAL 模式提升并发读写性能
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // 锁等待超时 5000ms，避免写冲突时立即报错
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;
        // 启用自动清理，保持数据库文件紧凑（须在建表前设置）
        conn.execute_batch("PRAGMA auto_vacuum=FULL;")?;
        // 启动时截断 WAL 文件，避免 WAL 无限增长
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(conn)
    }

    /// 打开并初始化只读连接。
    fn open_read_connection(path: &str) -> Result<Connection, StorageError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;
        Ok(conn)
    }
}

// ====================================================================== 单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    /// 创建临时数据库并初始化测试表。
    fn make_pool() -> (Pool, NamedTempFile) {
        let tmp = NamedTempFile::new().expect("创建临时文件失败");
        let pool = Pool::open(tmp.path(), 2).expect("打开连接池失败");
        pool.with_write(|conn| {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
            )?;
            Ok(())
        })
        .expect("建表失败");
        (pool, tmp)
    }

    #[test]
    fn write_and_read_roundtrip() {
        let (pool, _tmp) = make_pool();

        pool.with_write(|conn| {
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)",
                rusqlite::params!["hello", "world"],
            )?;
            Ok(())
        })
        .expect("写入失败");

        let value: String = pool
            .with_read(|conn| {
                let v = conn.query_row(
                    "SELECT value FROM kv WHERE key = ?1",
                    rusqlite::params!["hello"],
                    |row| row.get(0),
                )?;
                Ok(v)
            })
            .expect("读取失败");

        assert_eq!(value, "world");
    }

    #[test]
    fn transaction_commit() {
        let (pool, _tmp) = make_pool();

        pool.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)",
                rusqlite::params!["tx_key", "tx_value"],
            )?;
            Ok(())
        })
        .expect("事务提交失败");

        let found: bool = pool
            .with_read(|conn| {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM kv WHERE key = ?1",
                    rusqlite::params!["tx_key"],
                    |row| row.get(0),
                )?;
                Ok(count > 0)
            })
            .expect("读取失败");

        assert!(found, "提交后数据应存在");
    }

    #[test]
    fn transaction_rollback_on_error() {
        let (pool, _tmp) = make_pool();

        let result: Result<(), StorageError> = pool.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)",
                rusqlite::params!["rb_key", "rb_value"],
            )?;
            // 强制返回错误，触发回滚
            Err(StorageError::Validation("强制回滚".into()))
        });

        assert!(result.is_err(), "事务应返回错误");

        let found: bool = pool
            .with_read(|conn| {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM kv WHERE key = ?1",
                    rusqlite::params!["rb_key"],
                    |row| row.get(0),
                )?;
                Ok(count > 0)
            })
            .expect("读取失败");

        assert!(!found, "回滚后数据不应存在");
    }

    #[test]
    fn wal_mode_enabled() {
        let (pool, _tmp) = make_pool();

        let mode: String = pool
            .with_read(|conn| {
                let m = conn.query_row(
                    "PRAGMA journal_mode;",
                    [],
                    |row| row.get(0),
                )?;
                Ok(m)
            })
            .expect("读取 journal_mode 失败");

        assert_eq!(mode, "wal", "journal_mode 应为 WAL");
    }
}
