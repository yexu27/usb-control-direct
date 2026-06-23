//! Token 中间件和权限中间件。
//!
//! token 中间件从 payload 提取 session_token，调用 AuthService 校验。
//! 权限中间件根据 Router 注册的角色声明校验当前会话角色。

use prost::Message;
use tracing::warn;

use auth_session::session::SessionInfo;
use auth_session::AuthService;
use common::code::ResultCode;

/// 从 protobuf payload 中提取 session_token（field 1, string）。
///
/// 所有需要 token 的消息的第一个字段都是 session_token。
/// 使用一个只包含 field 1 的 wrapper 结构体进行反序列化。
pub fn extract_token(payload: &[u8]) -> Option<String> {
    #[derive(Message)]
    struct TokenWrapper {
        #[prost(string, tag = "1")]
        session_token: String,
    }

    TokenWrapper::decode(payload)
        .ok()
        .map(|w| w.session_token)
        .filter(|t| !t.is_empty())
}

/// Token 校验结果。
pub enum TokenResult {
    /// 白名单 cmd，跳过 token 校验。
    Whitelist,
    /// token 校验成功。
    Authenticated(SessionInfo),
    /// token 校验失败。
    Failed(ResultCode),
}

/// CMD 白名单（不需要 token）。
const TOKEN_WHITELIST: &[u32] = &[
    0x0001, // CMD_LOGIN
];

/// 需要 token 但由 handler 自行校验的 cmd。
const HANDLER_VALIDATES_TOKEN: &[u32] = &[
    0x0003, // CMD_AUTH_STATUS_QUERY
];

/// Token 中间件：校验 token 有效性。
///
/// 返回 TokenResult 指示后续处理方式。
pub fn check_token(msg_type: u32, payload: &[u8], auth_service: &AuthService) -> TokenResult {
    if TOKEN_WHITELIST.contains(&msg_type) {
        return TokenResult::Whitelist;
    }

    // handler 自行校验的 cmd，传 session=None 给 handler
    if HANDLER_VALIDATES_TOKEN.contains(&msg_type) {
        return TokenResult::Whitelist;
    }

    let token = match extract_token(payload) {
        Some(t) => t,
        None => return TokenResult::Failed(ResultCode::Unauthenticated),
    };

    match auth_service.validate_token(&token) {
        Ok(info) => {
            // 业务请求刷新活跃时间
            auth_service.refresh_token(&token);
            TokenResult::Authenticated(info)
        }
        Err(_) => TokenResult::Failed(ResultCode::Unauthenticated),
    }
}

/// 权限中间件：校验当前角色是否在允许列表中。
///
/// `allowed_roles` 为空表示无角色限制。
pub fn check_permission(session: &SessionInfo, allowed_roles: &[i32]) -> Result<(), ResultCode> {
    if allowed_roles.is_empty() {
        return Ok(());
    }
    if allowed_roles.contains(&session.role) {
        Ok(())
    } else {
        warn!(user = %session.username, role = session.role, allowed = ?allowed_roles, "权限拒绝");
        Err(ResultCode::PermissionDenied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn extract_token_from_valid_payload() {
        #[derive(prost::Message)]
        struct FakeCmd {
            #[prost(string, tag = "1")]
            session_token: String,
            #[prost(string, tag = "2")]
            other_field: String,
        }
        let cmd = FakeCmd {
            session_token: "my_token_123".into(),
            other_field: "data".into(),
        };
        let payload = cmd.encode_to_vec();
        assert_eq!(extract_token(&payload), Some("my_token_123".into()));
    }

    #[test]
    fn extract_token_empty_returns_none() {
        #[derive(prost::Message)]
        struct FakeCmd {
            #[prost(string, tag = "1")]
            session_token: String,
        }
        let cmd = FakeCmd {
            session_token: "".into(),
        };
        let payload = cmd.encode_to_vec();
        assert_eq!(extract_token(&payload), None);
    }

    #[test]
    fn check_permission_allowed() {
        let info = SessionInfo {
            user_id: 1,
            username: "admin".into(),
            role: 0,
            issue_time: 0,
            last_active_time: 0,
            source_ip: "".into(),
        };
        assert!(check_permission(&info, &[0, 1]).is_ok());
    }

    #[test]
    fn check_permission_denied() {
        let info = SessionInfo {
            user_id: 3,
            username: "audit".into(),
            role: 2,
            issue_time: 0,
            last_active_time: 0,
            source_ip: "".into(),
        };
        assert_eq!(
            check_permission(&info, &[0]).unwrap_err(),
            ResultCode::PermissionDenied
        );
    }

    #[test]
    fn check_permission_empty_roles_always_ok() {
        let info = SessionInfo {
            user_id: 1,
            username: "admin".into(),
            role: 0,
            issue_time: 0,
            last_active_time: 0,
            source_ip: "".into(),
        };
        assert!(check_permission(&info, &[]).is_ok());
    }
}
