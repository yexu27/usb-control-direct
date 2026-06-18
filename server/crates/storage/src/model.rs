//! T01-T11 行映射结构体。
//!
//! 每个结构体与数据库表一一对应，用于 CRUD 函数的输入输出。
//! 禁止暴露 `rusqlite::Row` 给上层。

use serde::{Deserialize, Serialize};

/// T01 白名单条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbWhitelist {
    pub id: i64,
    pub serial_number: String,
    pub vid: Option<String>,
    pub pid: Option<String>,
    pub device_name: Option<String>,
    pub capacity_bytes: Option<i64>,
    pub device_type: String,
    pub description: Option<String>,
    pub permission: i32,
    pub add_method: i32,
    pub created_at: i64,
}

/// T01 插入参数。
#[derive(Debug, Clone)]
pub struct UsbWhitelistInsert {
    pub serial_number: String,
    pub vid: Option<String>,
    pub pid: Option<String>,
    pub device_name: Option<String>,
    pub capacity_bytes: Option<i64>,
    pub device_type: String,
    pub description: Option<String>,
    pub permission: i32,
    pub add_method: i32,
}

/// T02 文件类型黑名单条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeBlacklist {
    pub id: i64,
    pub extension: String,
    pub description: Option<String>,
    pub is_default: i32,
    pub created_at: i64,
}

/// T03 文件访问控制开关。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessPolicy {
    pub policy_key: String,
    pub enabled: i32,
    pub updated_at: i64,
}

/// T04 可执行程序类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecType {
    pub id: i64,
    pub type_name: String,
    pub description: Option<String>,
}

/// T05 USB 审计日志。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbAuditLog {
    pub id: i64,
    pub event_time: i64,
    pub device_type: Option<String>,
    pub interface_type: Option<String>,
    pub interface_class: Option<i32>,
    pub interface_subclass: Option<i32>,
    pub interface_protocol: Option<i32>,
    pub device_name: Option<String>,
    pub device_sn: Option<String>,
    pub vid: Option<String>,
    pub pid: Option<String>,
    pub event_type: String,
    pub permission: Option<i32>,
    pub capacity_bytes: Option<i64>,
    pub file_path: Option<String>,
    pub matched_policy: Option<String>,
    pub result: String,
    pub fail_reason: Option<String>,
    pub detail: Option<String>,
}

/// T05 插入参数。
#[derive(Debug, Clone)]
pub struct UsbAuditLogInsert {
    pub event_time: i64,
    pub device_type: Option<String>,
    pub interface_type: Option<String>,
    pub interface_class: Option<i32>,
    pub interface_subclass: Option<i32>,
    pub interface_protocol: Option<i32>,
    pub device_name: Option<String>,
    pub device_sn: Option<String>,
    pub vid: Option<String>,
    pub pid: Option<String>,
    pub event_type: String,
    pub permission: Option<i32>,
    pub capacity_bytes: Option<i64>,
    pub file_path: Option<String>,
    pub matched_policy: Option<String>,
    pub result: String,
    pub fail_reason: Option<String>,
    pub detail: Option<String>,
}

/// T06 恶意代码检测日志。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalwareLog {
    pub id: i64,
    pub scan_time: i64,
    pub device_sn: Option<String>,
    pub device_name: Option<String>,
    pub file_path: Option<String>,
    pub scan_result: i32,
    pub virus_name: Option<String>,
    pub virus_db_version: Option<String>,
    pub process_result: Option<i32>,
    pub fail_reason: Option<String>,
    pub detail: Option<String>,
}

/// T06 插入参数。
#[derive(Debug, Clone)]
pub struct MalwareLogInsert {
    pub scan_time: i64,
    pub device_sn: Option<String>,
    pub device_name: Option<String>,
    pub file_path: Option<String>,
    pub scan_result: i32,
    pub virus_name: Option<String>,
    pub virus_db_version: Option<String>,
    pub process_result: Option<i32>,
    pub fail_reason: Option<String>,
    pub detail: Option<String>,
}

/// T07 系统配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub config_key: String,
    pub config_value: Option<String>,
    pub updated_at: i64,
}

/// T08 用户。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: i32,
    pub status: i32,
    pub is_builtin: i32,
    pub login_fail_count: i32,
    pub lock_until: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub password_changed_at: Option<i64>,
}

/// T08 插入参数。
#[derive(Debug, Clone)]
pub struct UserInsert {
    pub username: String,
    pub password_hash: String,
    pub role: i32,
}

/// T09 角色权限。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission {
    pub id: i64,
    pub role: i32,
    pub page_key: String,
}

/// T10 操作日志。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: i64,
    pub op_time: i64,
    pub username: String,
    pub role: i32,
    pub log_type: String,
    pub action_type: Option<String>,
    pub target: Option<String>,
    pub before_value: Option<String>,
    pub after_value: Option<String>,
    pub related_file: Option<String>,
    pub related_version: Option<String>,
    pub result: i32,
    pub fail_reason: Option<String>,
    pub source_ip: Option<String>,
    pub app_version: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub detail: Option<String>,
}

/// T10 插入参数。
#[derive(Debug, Clone)]
pub struct OperationLogInsert {
    pub op_time: i64,
    pub username: String,
    pub role: i32,
    pub log_type: String,
    pub action_type: Option<String>,
    pub target: Option<String>,
    pub before_value: Option<String>,
    pub after_value: Option<String>,
    pub related_file: Option<String>,
    pub related_version: Option<String>,
    pub result: i32,
    pub fail_reason: Option<String>,
    pub source_ip: Option<String>,
    pub app_version: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub detail: Option<String>,
}

/// T11 日志覆盖追溯。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionEvent {
    pub id: i64,
    pub trigger_time: i64,
    pub log_category: String,
    pub storage_usage_percent: i32,
    pub covered_from_time: Option<i64>,
    pub covered_to_time: Option<i64>,
    pub covered_count: i32,
    pub result: i32,
    pub fail_reason: Option<String>,
}

/// T11 插入参数。
#[derive(Debug, Clone)]
pub struct LogRetentionEventInsert {
    pub trigger_time: i64,
    pub log_category: String,
    pub storage_usage_percent: i32,
    pub covered_from_time: Option<i64>,
    pub covered_to_time: Option<i64>,
    pub covered_count: i32,
    pub result: i32,
    pub fail_reason: Option<String>,
}

/// 日志分页查询参数。
#[derive(Debug, Clone, Default)]
pub struct LogQueryParams {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub keyword: Option<String>,
    pub event_type: Option<String>,
    pub log_category: Option<String>,
    pub action_type: Option<String>,
    pub page: i32,
    pub page_size: i32,
}
