//! 时间工具。

use std::time::{SystemTime, UNIX_EPOCH};

/// 获取当前 Unix 时间戳（秒）。
pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
