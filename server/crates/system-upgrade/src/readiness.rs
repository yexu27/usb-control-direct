//! 主服务与 updater 共享的健康就绪文件格式。

use serde::{Deserialize, Serialize};

use crate::SystemVersion;

/// 主服务完成全部启动依赖后发布的稳定就绪信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ServiceReady {
    pub format_version: u32,
    pub version: SystemVersion,
    pub schema_version: u32,
    pub pid: u32,
    pub started_at: i64,
}
