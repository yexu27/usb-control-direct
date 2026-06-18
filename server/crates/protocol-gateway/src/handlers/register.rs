//! Handler 统一注册。

use crate::router::Router;

/// CMD 常量（鉴权）。
const CMD_LOGIN: u32 = 0x0001;
const CMD_AUTH_STATUS_QUERY: u32 = 0x0003;
const CMD_LOGOUT: u32 = 0x0009;
const CMD_CHANGE_PASSWORD: u32 = 0x0605;

/// CMD 常量（白名单）。
const CMD_LIST_WHITELIST: u32 = 0x0100;
const CMD_ADD_WHITELIST: u32 = 0x0104;
const CMD_REMOVE_WHITELIST: u32 = 0x0105;
const CMD_UPDATE_WHITELIST: u32 = 0x0106;
const CMD_GET_CONNECTED_DEVICES: u32 = 0x0102;

/// CMD 常量（文件访问控制策略）。
const CMD_GET_FILE_POLICY: u32 = 0x0200;
const CMD_UPDATE_FILE_POLICY_SWITCH: u32 = 0x0202;
const CMD_ADD_BLACKLIST_EXTENSION: u32 = 0x0203;
const CMD_REMOVE_BLACKLIST_EXTENSION: u32 = 0x0204;

/// CMD 常量（策略导入导出）。
const CMD_EXPORT_POLICY: u32 = 0x0300;
const CMD_IMPORT_POLICY: u32 = 0x0302;

/// CMD 常量（授权管理）。
const CMD_GET_MACHINE_CODE: u32 = 0x0005;
const CMD_UPLOAD_LICENSE: u32 = 0x0007;

/// CMD 常量（系统管理）。
const CMD_GET_SYSTEM_INFO: u32 = 0x0500;
const CMD_UPLOAD_SYSTEM_UPGRADE: u32 = 0x0502;
const CMD_UPLOAD_VIRUSDB_UPGRADE: u32 = 0x0503;
const CMD_UPDATE_DEVICE_DESC: u32 = 0x0504;

/// CMD 常量（日志管理）。
const CMD_QUERY_LOGS: u32 = 0x0400;
const CMD_EXPORT_LOGS: u32 = 0x0402;
const CMD_DELETE_LOGS: u32 = 0x0404;

/// CMD 常量（用户管理）。
const CMD_LIST_USERS: u32 = 0x0600;
const CMD_CREATE_USER: u32 = 0x0602;
const CMD_DELETE_USER: u32 = 0x0603;
const CMD_RESET_PASSWORD: u32 = 0x0604;

/// 注册所有 P02 handler。
pub fn register_auth_handlers(router: &mut Router) {
    // 登录（白名单 cmd，无角色限制）
    router.register(CMD_LOGIN, Box::new(super::login::handle_login));

    // 授权状态查询（白名单 cmd，handler 自行校验 token）
    router.register(
        CMD_AUTH_STATUS_QUERY,
        Box::new(super::auth_status::handle_auth_status),
    );

    // 登出（所有已登录角色可调用）
    router.register_with_roles(
        CMD_LOGOUT,
        Box::new(super::logout::handle_logout),
        vec![0, 1, 2],
    );

    // 改密码（所有已登录角色可调用）
    router.register_with_roles(
        CMD_CHANGE_PASSWORD,
        Box::new(super::change_password::handle_change_password),
        vec![0, 1, 2],
    );
}

/// 注册所有 P03 白名单 handler。
pub fn register_whitelist_handlers(router: &mut Router) {
    // 白名单列表（管理员和审计员可查看）
    router.register_with_roles(
        CMD_LIST_WHITELIST,
        Box::new(super::whitelist::handle_list_whitelist),
        vec![0, 1],
    );

    // 添加白名单（管理员和审计员可操作）
    router.register_with_roles(
        CMD_ADD_WHITELIST,
        Box::new(super::whitelist::handle_add_whitelist),
        vec![0, 1],
    );

    // 删除白名单（仅管理员）
    router.register_with_roles(
        CMD_REMOVE_WHITELIST,
        Box::new(super::whitelist::handle_remove_whitelist),
        vec![0],
    );

    // 更新白名单（仅管理员）
    router.register_with_roles(
        CMD_UPDATE_WHITELIST,
        Box::new(super::whitelist::handle_update_whitelist),
        vec![0],
    );

    // 当前连接设备列表（管理员和审计员）
    router.register_with_roles(
        CMD_GET_CONNECTED_DEVICES,
        Box::new(super::connected_devices::handle_get_connected_devices),
        vec![0, 1],
    );
}

/// 注册所有 P05 文件访问控制 handler。
pub fn register_file_access_handlers(router: &mut Router) {
    router.register_with_roles(
        CMD_GET_FILE_POLICY,
        Box::new(super::file_access::handle_get_file_policy),
        vec![1],
    );

    router.register_with_roles(
        CMD_UPDATE_FILE_POLICY_SWITCH,
        Box::new(super::file_access::handle_update_file_policy_switch),
        vec![1],
    );

    router.register_with_roles(
        CMD_ADD_BLACKLIST_EXTENSION,
        Box::new(super::file_access::handle_add_blacklist_extension),
        vec![1],
    );

    router.register_with_roles(
        CMD_REMOVE_BLACKLIST_EXTENSION,
        Box::new(super::file_access::handle_remove_blacklist_extension),
        vec![1],
    );
}

/// 注册策略导入导出 handler。
pub fn register_policy_handlers(router: &mut Router) {
    // 导出策略（操作员可操作）
    router.register_with_roles(
        CMD_EXPORT_POLICY,
        Box::new(super::policy::handle_export_policy),
        vec![1],
    );

    // 导入策略（操作员可操作）
    router.register_with_roles(
        CMD_IMPORT_POLICY,
        Box::new(super::policy::handle_import_policy),
        vec![1],
    );
}

/// 注册授权管理 handler。
pub fn register_license_handlers(router: &mut Router) {
    // 获取机器码（所有角色可调用，handler 内部按授权状态分支判权）
    router.register_with_roles(
        CMD_GET_MACHINE_CODE,
        Box::new(super::license::handle_get_machine_code),
        vec![0, 1, 2],
    );

    // 上传授权文件（所有角色可调用，handler 内部按授权状态分支判权）
    router.register_with_roles(
        CMD_UPLOAD_LICENSE,
        Box::new(super::license::handle_upload_license),
        vec![0, 1, 2],
    );
}

/// 注册系统管理 handler。
pub fn register_system_handlers(router: &mut Router) {
    // 获取系统信息（所有角色可查看）
    router.register_with_roles(
        CMD_GET_SYSTEM_INFO,
        Box::new(super::system::handle_get_system_info),
        vec![0, 1, 2],
    );

    // 系统升级（仅管理员）
    router.register_with_roles(
        CMD_UPLOAD_SYSTEM_UPGRADE,
        Box::new(super::system::handle_upload_system_upgrade),
        vec![0],
    );

    // 病毒库升级（仅管理员）
    router.register_with_roles(
        CMD_UPLOAD_VIRUSDB_UPGRADE,
        Box::new(super::system::handle_upload_virusdb_upgrade),
        vec![0],
    );

    // 修改设备描述（仅管理员）
    router.register_with_roles(
        CMD_UPDATE_DEVICE_DESC,
        Box::new(super::system::handle_update_device_desc),
        vec![0],
    );
}

/// 注册日志管理 handler。
pub fn register_log_handlers(router: &mut Router) {
    // 查询日志（审计员可操作）
    router.register_with_roles(
        CMD_QUERY_LOGS,
        Box::new(super::logs::handle_query_logs),
        vec![2],
    );

    // 导出日志（审计员可操作）
    router.register_with_roles(
        CMD_EXPORT_LOGS,
        Box::new(super::logs::handle_export_logs),
        vec![2],
    );

    // 清理日志（审计员可操作）
    router.register_with_roles(
        CMD_DELETE_LOGS,
        Box::new(super::logs::handle_delete_logs),
        vec![2],
    );
}

/// 注册用户管理 handler。
pub fn register_user_handlers(router: &mut Router) {
    // 用户列表（仅管理员）
    router.register_with_roles(
        CMD_LIST_USERS,
        Box::new(super::users::handle_list_users),
        vec![0],
    );

    // 创建用户（仅管理员）
    router.register_with_roles(
        CMD_CREATE_USER,
        Box::new(super::users::handle_create_user),
        vec![0],
    );

    // 删除用户（仅管理员）
    router.register_with_roles(
        CMD_DELETE_USER,
        Box::new(super::users::handle_delete_user),
        vec![0],
    );

    // 重置密码（仅管理员）
    router.register_with_roles(
        CMD_RESET_PASSWORD,
        Box::new(super::users::handle_reset_password),
        vec![0],
    );
}
