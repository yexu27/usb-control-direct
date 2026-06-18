//! S11 装置侧数据存储适配。
//!
//! 封装 SQLite 连接池（1 写 + N 读）与 T01-T11 全部 CRUD 接口。
//! 上层通过 `Storage` 句柄调用，禁止暴露 `rusqlite::Connection`。

pub mod error;
pub mod model;

pub(crate) mod pool;
pub(crate) mod schema;
pub(crate) mod usb_whitelist;
pub(crate) mod file_type_blacklist;
pub(crate) mod file_access_policy;
pub(crate) mod exec_type;
pub(crate) mod usb_audit_log;
pub(crate) mod malware_log;
pub(crate) mod system_config;
pub(crate) mod users;
pub(crate) mod role_permission;
pub(crate) mod operation_log;
pub(crate) mod log_retention_event;
pub(crate) mod policy_import;

pub use error::StorageError;

use std::path::Path;

use pool::Pool;

/// 装置侧数据存储门面。
///
/// 所有数据库操作通过本结构体的方法执行，禁止暴露底层连接。
pub struct Storage {
    pool: Pool,
}

impl Storage {
    /// 打开数据库并执行 schema 初始化。
    ///
    /// 参数:
    ///   - `path`: 数据库文件路径。
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let pool = Pool::open(path, 4)?;
        pool.with_transaction(|conn| {
            schema::migrate(conn)?;
            Ok(())
        })?;
        Ok(Storage { pool })
    }

    /// 获取底层连接池引用（仅 crate 内部使用）。
    pub(crate) fn pool(&self) -> &Pool {
        &self.pool
    }
}

/// 分页查询每页最大条数。
pub(crate) const MAX_PAGE_SIZE: i32 = 1000;

/// 转义 LIKE 模式通配符。
///
/// 将 `%`、`_` 和 `\` 转义后包裹在 `%...%` 中，
/// 配合 `ESCAPE '\'` 子句使用，防止用户输入被解释为通配符。
pub(crate) fn escape_like_keyword(keyword: &str) -> String {
    let escaped = keyword
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// 获取当前 Unix 时间戳（秒）。
pub(crate) fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
