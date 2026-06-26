//! 业务日志常量模块，与架构 08 枚举严格一致。

/// T05 USB 审计日志 event_type（共 3 个）。
pub mod event_type {
    pub const INSERT_SUCCESS: &str = "insert_success";
    pub const INSERT_FAILED: &str = "insert_failed";
    pub const DEVICE_REMOVE: &str = "device_remove";
}

/// T10 操作日志 log_type（架构 08 枚举，共 7 个）。
pub mod log_type {
    pub const LOGIN_AUTH: &str = "login_auth";
    pub const USER_MANAGEMENT: &str = "user_management";
    pub const SECURITY_CONFIG: &str = "security_config";
    pub const AUTH_MANAGEMENT: &str = "auth_management";
    pub const SYSTEM_MANAGEMENT: &str = "system_management";
    pub const PROGRAM_UPGRADE: &str = "program_upgrade";
    pub const LOG_MANAGEMENT: &str = "log_management";
}

/// T10 操作日志 action_type（架构 08 示例值）。
pub mod action_type {
    pub const LOGIN: &str = "login";
    pub const LOGOUT: &str = "logout";
    pub const LOGIN_FAILED: &str = "login_failed";
    pub const PASSWORD_CHANGE: &str = "password_change";
    pub const PASSWORD_RESET: &str = "password_reset";
    pub const USER_CREATE: &str = "user_create";
    pub const USER_DELETE: &str = "user_delete";
    pub const WHITELIST_ADD: &str = "whitelist_add";
    pub const WHITELIST_REMOVE: &str = "whitelist_remove";
    pub const WHITELIST_UPDATE: &str = "whitelist_update";
    pub const FILE_POLICY_UPDATE: &str = "file_policy_update";
    pub const BLACKLIST_ADD: &str = "blacklist_add";
    pub const BLACKLIST_REMOVE: &str = "blacklist_remove";
    pub const POLICY_IMPORT: &str = "policy_import";
    pub const POLICY_EXPORT: &str = "policy_export";
    pub const SYSTEM_UPGRADE: &str = "system_upgrade";
    pub const VIRUSDB_UPGRADE: &str = "virusdb_upgrade";
    pub const AUTH_UPLOAD: &str = "auth_upload";
    pub const DEVICE_DESC_UPDATE: &str = "device_desc_update";
    pub const LOG_CLEAN: &str = "log_clean";
    pub const LOG_EXPORT: &str = "log_export";
    pub const MACHINE_CODE_DOWNLOAD: &str = "machine_code_download";
}

/// T06 恶意代码检测日志 scan_result。
pub mod scan_result {
    pub const CLEAN: i32 = 0;
    pub const INFECTED: i32 = 1;
    pub const FAILED: i32 = 2;
    pub const CANCELLED: i32 = 3;
}

/// T06 恶意代码检测日志 process_result。
pub mod process_result {
    pub const MARKED: i32 = 0;
    pub const BLOCKED: i32 = 1;
    pub const FAILED: i32 = 2;
    pub const NO_ACTION: i32 = 3;
}
