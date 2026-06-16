//! S08 鉴权业务服务。
//!
//! 组合 password 模块和 SessionManager，提供登录、锁定、改密码等完整业务接口。

use storage::Storage;

use crate::error::AuthError;
use crate::password;
use crate::session::{SessionInfo, SessionManager};

/// 连续登录失败锁定阈值。
const MAX_FAIL_COUNT: i32 = 5;

/// 锁定时长（秒）：5 分钟。
const LOCK_DURATION_SECS: i64 = 300;

/// 登录结果。
#[derive(Debug)]
pub struct LoginResult {
    /// session token。
    pub token: String,
    /// 用户名。
    pub username: String,
    /// 角色（0/1/2）。
    pub role: i32,
}

/// 鉴权业务服务。
pub struct AuthService {
    storage: Storage,
    session_mgr: SessionManager,
}

impl AuthService {
    /// 创建鉴权服务。
    pub fn new(storage: Storage, session_mgr: SessionManager) -> Self {
        AuthService {
            storage,
            session_mgr,
        }
    }

    /// 获取 SessionManager 引用。
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_mgr
    }

    /// 获取 Storage 引用。
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// 用户登录。
    ///
    /// 流程：
    /// 1. 查用户 → 不存在返回 UserOrPasswordError
    /// 2. 检查锁定状态（lock_until）
    /// 3. 验证密码 → 失败递增计数，达到阈值则锁定
    /// 4. 成功 → 清零计数，创建 session
    pub fn login(
        &self,
        username: &str,
        password: &str,
        source_ip: &str,
    ) -> Result<LoginResult, AuthError> {
        let user = self
            .storage
            .user_query_by_username(username)?
            .ok_or(AuthError::UserOrPasswordError)?;

        // 已删除用户不可登录
        if user.status == 2 {
            return Err(AuthError::UserOrPasswordError);
        }

        // 检查锁定
        let fail_count_base = if let Some(lock_until) = user.lock_until {
            let now = now_unix();
            if now < lock_until {
                return Err(AuthError::AccountLocked);
            }
            // 锁定超时自动解除，清零计数
            self.storage
                .user_update_login_fail(user.id, 0, None)?;
            0
        } else {
            user.login_fail_count
        };

        // 验证密码
        let matched = password::verify_password(password, &user.password_hash)?;
        if !matched {
            let new_count = fail_count_base + 1;
            let lock_until = if new_count >= MAX_FAIL_COUNT {
                Some(now_unix() + LOCK_DURATION_SECS)
            } else {
                None
            };
            self.storage
                .user_update_login_fail(user.id, new_count, lock_until)?;
            return Err(AuthError::UserOrPasswordError);
        }

        // 登录成功：清零失败计数
        if fail_count_base > 0 {
            self.storage
                .user_update_login_fail(user.id, 0, None)?;
        }

        // 创建 session
        let token = self
            .session_mgr
            .create_session(user.id, &user.username, user.role, source_ip);

        Ok(LoginResult {
            token,
            username: user.username,
            role: user.role,
        })
    }

    /// 用户登出。
    pub fn logout(&self, token: &str) -> Result<SessionInfo, AuthError> {
        let info = self.session_mgr.validate_token(token)?;
        self.session_mgr.remove_token(token);
        Ok(info)
    }

    /// 修改密码。
    ///
    /// 流程：
    /// 1. 从 token 获取当前用户
    /// 2. 验证旧密码
    /// 3. 校验新密码复杂度
    /// 4. 校验确认密码一致
    /// 5. 更新密码 hash
    /// 6. 使该用户所有 token 失效
    pub fn change_password(
        &self,
        token: &str,
        old_password: &str,
        new_password: &str,
        confirm_password: &str,
    ) -> Result<SessionInfo, AuthError> {
        let info = self.session_mgr.validate_token(token)?;

        // 校验确认密码
        if new_password != confirm_password {
            return Err(AuthError::PasswordConfirmMismatch);
        }

        // 校验新密码复杂度
        if !password::validate_password_complexity(new_password) {
            return Err(AuthError::PasswordComplexity);
        }

        // 查用户并验证旧密码
        let user = self
            .storage
            .user_query_by_username(&info.username)?
            .ok_or(AuthError::UserNotFound)?;

        let old_matched = password::verify_password(old_password, &user.password_hash)?;
        if !old_matched {
            return Err(AuthError::OldPasswordError);
        }

        // 更新密码
        let new_hash = password::hash_password(new_password)?;
        self.storage.user_update_password(user.id, &new_hash)?;

        // 使该用户所有 token 失效
        self.session_mgr.invalidate_user_sessions(user.id);

        Ok(info)
    }

    /// 验证 token（代理到 SessionManager）。
    pub fn validate_token(&self, token: &str) -> Result<SessionInfo, AuthError> {
        self.session_mgr.validate_token(token)
    }

    /// 刷新 token 活跃时间（代理到 SessionManager）。
    pub fn refresh_token(&self, token: &str) {
        self.session_mgr.refresh_token(token);
    }
}

/// 获取当前 Unix 时间戳（秒）。
fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
