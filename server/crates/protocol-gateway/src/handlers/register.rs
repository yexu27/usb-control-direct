//! Handler 统一注册。

use crate::router::Router;

/// CMD 常量。
const CMD_LOGIN: u32 = 0x0001;
const CMD_AUTH_STATUS_QUERY: u32 = 0x0003;
const CMD_LOGOUT: u32 = 0x0009;
const CMD_CHANGE_PASSWORD: u32 = 0x0605;

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
