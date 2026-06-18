//! 会话 token 管理。
//!
//! 内存中维护 `HashMap<token, SessionInfo>`，不落盘。
//! token 使用 OsRng 生成 32 字节随机数，base64url 编码。

use std::collections::HashMap;
use std::sync::Mutex;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::AuthError;

/// 会话超时时长（秒）：5 分钟。
const SESSION_TIMEOUT_SECS: i64 = 300;

/// 会话信息。
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// 用户 ID（T08.id）。
    pub user_id: i64,
    /// 用户名。
    pub username: String,
    /// 角色（0=admin, 1=operator, 2=auditor）。
    pub role: i32,
    /// 签发时间（Unix 秒）。
    pub issue_time: i64,
    /// 最后活跃时间（Unix 秒）。
    pub last_active_time: i64,
    /// 管理端来源 IP。
    pub source_ip: String,
}

/// 会话管理器。
pub struct SessionManager {
    sessions: Mutex<HashMap<String, SessionInfo>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// 创建会话管理器。
    pub fn new() -> Self {
        SessionManager {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// 创建新会话，返回 token。
    pub fn create_session(
        &self,
        user_id: i64,
        username: &str,
        role: i32,
        source_ip: &str,
    ) -> String {
        let token = generate_token();
        let now = now_unix();
        let info = SessionInfo {
            user_id,
            username: username.to_string(),
            role,
            issue_time: now,
            last_active_time: now,
            source_ip: source_ip.to_string(),
        };
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(token.clone(), info);
        token
    }

    /// 验证 token 有效性。返回 SessionInfo 的克隆。不更新 last_active_time。
    pub fn validate_token(&self, token: &str) -> Result<SessionInfo, AuthError> {
        let sessions = self.sessions.lock().unwrap();
        match sessions.get(token) {
            Some(info) => {
                let now = now_unix();
                if now - info.last_active_time > SESSION_TIMEOUT_SECS {
                    drop(sessions);
                    self.remove_token(token);
                    return Err(AuthError::Unauthenticated);
                }
                Ok(info.clone())
            }
            None => Err(AuthError::Unauthenticated),
        }
    }

    /// 刷新 token 的最后活跃时间。由业务请求（非心跳）触发。
    pub fn refresh_token(&self, token: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(info) = sessions.get_mut(token) {
            info.last_active_time = now_unix();
        }
    }

    /// 销毁指定 token。
    pub fn remove_token(&self, token: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(token);
    }

    /// 使指定用户的所有 token 失效。用于改密码、删除用户等场景。
    pub fn invalidate_user_sessions(&self, user_id: i64) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|_, info| info.user_id != user_id);
    }

    /// 清理所有过期 session（可定期调用）。
    pub fn cleanup_expired(&self) {
        let now = now_unix();
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|_, info| now - info.last_active_time <= SESSION_TIMEOUT_SECS);
    }

    /// 当前活跃 session 数（测试用）。
    #[cfg(test)]
    pub fn active_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }
}

/// 使用 OsRng 生成 32 字节随机数，base64url 编码。
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// 获取当前 Unix 时间戳（秒）。
fn now_unix() -> i64 {
    common::time::now_unix()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_validate_session() {
        let mgr = SessionManager::new();
        let token = mgr.create_session(1, "admin", 0, "192.168.1.1");
        assert!(!token.is_empty());

        let info = mgr.validate_token(&token).unwrap();
        assert_eq!(info.user_id, 1);
        assert_eq!(info.username, "admin");
        assert_eq!(info.role, 0);
        assert_eq!(info.source_ip, "192.168.1.1");
    }

    #[test]
    fn validate_invalid_token_returns_unauthenticated() {
        let mgr = SessionManager::new();
        assert!(mgr.validate_token("invalid_token").is_err());
    }

    #[test]
    fn remove_token_invalidates_session() {
        let mgr = SessionManager::new();
        let token = mgr.create_session(1, "admin", 0, "127.0.0.1");
        mgr.remove_token(&token);
        assert!(mgr.validate_token(&token).is_err());
    }

    #[test]
    fn invalidate_user_sessions_removes_all_tokens() {
        let mgr = SessionManager::new();
        let _t1 = mgr.create_session(1, "admin", 0, "127.0.0.1");
        let _t2 = mgr.create_session(1, "admin", 0, "192.168.1.1");
        let t3 = mgr.create_session(2, "operator", 1, "127.0.0.1");
        assert_eq!(mgr.active_count(), 3);

        mgr.invalidate_user_sessions(1);
        assert_eq!(mgr.active_count(), 1);
        assert!(mgr.validate_token(&t3).is_ok());
    }

    #[test]
    fn refresh_token_updates_last_active() {
        let mgr = SessionManager::new();
        let token = mgr.create_session(1, "admin", 0, "127.0.0.1");
        let info_before = mgr.validate_token(&token).unwrap();
        mgr.refresh_token(&token);
        let info_after = mgr.validate_token(&token).unwrap();
        assert!(info_after.last_active_time >= info_before.last_active_time);
    }

    #[test]
    fn generate_token_is_unique() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
        assert_eq!(t1.len(), 43); // 32 bytes → base64url no-pad = 43 chars
    }

    #[test]
    fn cleanup_expired_removes_stale_sessions() {
        let mgr = SessionManager::new();
        let token = mgr.create_session(1, "admin", 0, "127.0.0.1");
        {
            let mut sessions = mgr.sessions.lock().unwrap();
            sessions.get_mut(&token).unwrap().last_active_time -= SESSION_TIMEOUT_SECS + 1;
        }
        mgr.cleanup_expired();
        assert_eq!(mgr.active_count(), 0);
    }
}
