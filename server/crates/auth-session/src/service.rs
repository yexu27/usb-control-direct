//! S08 鉴权业务服务。
//!
//! 组合 password 模块和 SessionManager，提供登录、锁定、改密码等完整业务接口。

use tracing::{debug, info, warn};

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
        debug!(user = %username, source_ip = %source_ip, "处理登录请求");

        let user = self
            .storage
            .user_query_by_username(username)?
            .ok_or(AuthError::UserOrPasswordError)?;

        // 已删除用户不可登录
        if user.status == 2 {
            warn!(user = %username, "已删除用户尝试登录");
            return Err(AuthError::UserOrPasswordError);
        }

        // 检查锁定
        let fail_count_base = if let Some(lock_until) = user.lock_until {
            let now = now_unix();
            if now < lock_until {
                warn!(user = %username, remaining = lock_until - now, "账户锁定中");
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
            if new_count >= MAX_FAIL_COUNT {
                warn!(user = %username, count = new_count, "登录失败次数达到阈值，账户锁定");
            } else {
                warn!(user = %username, count = new_count, "密码验证失败");
            }
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

        info!(user = %username, role = user.role, "登录验证成功");

        Ok(LoginResult {
            token,
            username: user.username,
            role: user.role,
        })
    }

    /// 用户登出。
    pub fn logout(&self, token: &str) -> Result<SessionInfo, AuthError> {
        debug!("处理登出请求");
        let info = self.session_mgr.validate_token(token)?;
        self.session_mgr.remove_token(token);
        info!(user = %info.username, "登出成功");
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

        debug!(user = %info.username, "处理密码修改请求");

        // 校验确认密码
        if new_password != confirm_password {
            debug!(user = %info.username, "确认密码不一致");
            return Err(AuthError::PasswordConfirmMismatch);
        }

        // 校验新密码复杂度
        if !password::validate_password_complexity(new_password) {
            debug!(user = %info.username, "新密码复杂度不满足要求");
            return Err(AuthError::PasswordComplexity);
        }

        // 查用户并验证旧密码
        let user = self
            .storage
            .user_query_by_username(&info.username)?
            .ok_or(AuthError::UserNotFound)?;

        let old_matched = password::verify_password(old_password, &user.password_hash)?;
        if !old_matched {
            warn!(user = %info.username, "旧密码验证失败");
            return Err(AuthError::OldPasswordError);
        }

        // 更新密码
        let new_hash = password::hash_password(new_password)?;
        self.storage.user_update_password(user.id, &new_hash)?;

        // 使该用户所有 token 失效
        self.session_mgr.invalidate_user_sessions(user.id);

        info!(user = %info.username, "密码修改成功");

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

    /// 查询全部活跃用户列表。
    pub fn list_users(&self) -> Result<Vec<storage::model::User>, AuthError> {
        debug!("查询活跃用户列表");
        let users = self.storage.user_query_active()?;
        debug!(count = users.len(), "活跃用户列表查询成功");
        Ok(users)
    }

    /// 新建用户。
    ///
    /// 校验:
    /// 1. 用户名唯一（包括已删除的不可复用）
    /// 2. 角色合法（0/1/2）
    /// 3. 密码复杂度
    /// 4. 确认密码一致
    pub fn create_user(
        &self,
        username: &str,
        role: i32,
        password_str: &str,
        confirm_password: &str,
    ) -> Result<i64, AuthError> {
        debug!(user = %username, role = role, "处理创建用户请求");

        if !validate_username(username) {
            return Err(AuthError::InvalidUsername);
        }

        if password_str != confirm_password {
            return Err(AuthError::PasswordConfirmMismatch);
        }

        if !password::validate_password_complexity(password_str) {
            return Err(AuthError::PasswordComplexity);
        }

        if !(0..=2).contains(&role) {
            return Err(AuthError::InvalidRole);
        }

        if let Some(existing) = self.storage.user_query_by_username(username)? {
            if existing.status == 2 {
                return Err(AuthError::UsernameDeletedReuse);
            }
            return Err(AuthError::UsernameExists);
        }

        let hash = password::hash_password(password_str)?;
        info!(user = %username, role = role, "用户创建成功");
        let insert = storage::model::UserInsert {
            username: username.to_string(),
            password_hash: hash,
            role,
        };
        let id = self.storage.user_insert(&insert)?;
        Ok(id)
    }

    /// 删除用户（软删除）。
    ///
    /// 校验:
    /// 1. 目标用户存在
    /// 2. 非内置用户
    /// 3. 不能删除自己（当前登录用户）
    pub fn delete_user(
        &self,
        target_username: &str,
        current_user_id: i64,
    ) -> Result<(), AuthError> {
        debug!(target_user = %target_username, "处理删除用户请求");

        let user = self
            .storage
            .user_query_by_username(target_username)?
            .ok_or(AuthError::UserNotFound)?;

        if user.status == 2 {
            return Err(AuthError::UserNotFound);
        }

        if user.is_builtin != 0 {
            return Err(AuthError::BuiltinUserNoDelete);
        }

        if user.id == current_user_id {
            return Err(AuthError::SelfDeleteForbidden);
        }

        self.storage.user_soft_delete(user.id)?;
        self.session_mgr.invalidate_user_sessions(user.id);

        info!(target_user = %target_username, "用户删除成功");

        Ok(())
    }

    /// 重置用户密码（管理员操作）。
    ///
    /// 校验:
    /// 1. 密码复杂度
    /// 2. 确认密码一致
    /// 3. 重置成功后清零失败计数并解除锁定
    pub fn reset_password(
        &self,
        target_username: &str,
        new_password: &str,
        confirm_password: &str,
    ) -> Result<(), AuthError> {
        debug!(target_user = %target_username, "处理重置密码请求");

        if new_password != confirm_password {
            return Err(AuthError::PasswordConfirmMismatch);
        }

        if !password::validate_password_complexity(new_password) {
            return Err(AuthError::PasswordComplexity);
        }

        let user = self
            .storage
            .user_query_by_username(target_username)?
            .ok_or(AuthError::UserNotFound)?;

        if user.status == 2 {
            return Err(AuthError::UserNotFound);
        }

        let new_hash = password::hash_password(new_password)?;
        self.storage.user_update_password(user.id, &new_hash)?;
        self.storage.user_update_login_fail(user.id, 0, None)?;
        self.session_mgr.invalidate_user_sessions(user.id);

        info!(target_user = %target_username, "密码重置成功");

        Ok(())
    }
}

/// 获取当前 Unix 时间戳（秒）。
fn now_unix() -> i64 {
    common::time::now_unix()
}

/// 校验用户名格式。
///
/// 规则：1-32 字符，仅允许字母、数字和下划线。
fn validate_username(username: &str) -> bool {
    !username.is_empty()
        && username.len() <= 32
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}
