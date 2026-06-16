//! S11 装置侧数据存储适配。
//!
//! 本 crate 封装所有对 SQLite 数据库的访问，对外仅暴露高层操作接口；
//! 底层 SQL 与连接池细节不对外泄露。

pub mod error;
pub mod model;
pub(crate) mod pool;
pub(crate) mod schema;

pub use error::StorageError;
