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

/// CMD 常量（文件访问控制策略）。
const CMD_GET_FILE_POLICY: u32 = 0x0200;
const CMD_UPDATE_FILE_POLICY_SWITCH: u32 = 0x0202;
const CMD_ADD_BLACKLIST_EXTENSION: u32 = 0x0203;
const CMD_REMOVE_BLACKLIST_EXTENSION: u32 = 0x0204;

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
