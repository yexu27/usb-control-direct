//! S08 鉴权服务集成测试。

use std::sync::Arc;

use auth_session::password;
use auth_session::service::AuthService;
use auth_session::session::SessionManager;
use storage::Storage;
use tempfile::NamedTempFile;

fn setup() -> (AuthService, tempfile::TempPath) {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.into_temp_path();
    let storage = Arc::new(Storage::open(&path).unwrap());

    // 内置用户由 schema 自动插入
    let session_mgr = SessionManager::new();
    let service = AuthService::new(storage, session_mgr);
    (service, path)
}

// ===== 登录 =====

#[test]
fn login_success_returns_token_and_role() {
    let (service, _path) = setup();
    let result = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    assert!(!result.token.is_empty());
    assert_eq!(result.username, "admin");
    assert_eq!(result.role, 0);
}

#[test]
fn login_operator_returns_role_1() {
    let (service, _path) = setup();
    let result = service.login("operator", "operator@123", "127.0.0.1").unwrap();
    assert_eq!(result.role, 1);
}

#[test]
fn login_audit_returns_role_2() {
    let (service, _path) = setup();
    let result = service.login("audit", "audit@123", "127.0.0.1").unwrap();
    assert_eq!(result.role, 2);
}

#[test]
fn login_wrong_password_returns_error() {
    let (service, _path) = setup();
    let err = service
        .login("admin", "wrong_password", "127.0.0.1")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::UserOrPasswordError
    );
}

#[test]
fn login_nonexistent_user_returns_error() {
    let (service, _path) = setup();
    let err = service
        .login("nobody", "password", "127.0.0.1")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::UserOrPasswordError
    );
}

// ===== 锁定 =====

#[test]
fn five_failures_lock_account() {
    let (service, _path) = setup();
    for _ in 0..5 {
        let _ = service.login("admin", "wrong", "127.0.0.1");
    }
    let err = service.login("admin", "admin@123", "127.0.0.1").unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::AccountLocked
    );
}

#[test]
fn successful_login_resets_fail_count() {
    let (service, _path) = setup();
    for _ in 0..4 {
        let _ = service.login("admin", "wrong", "127.0.0.1");
    }
    // 第 5 次用正确密码
    let result = service.login("admin", "admin@123", "127.0.0.1");
    assert!(result.is_ok());
    // 再来 4 次错误应该不会锁定
    for _ in 0..4 {
        let _ = service.login("admin", "wrong", "127.0.0.1");
    }
    let result = service.login("admin", "admin@123", "127.0.0.1");
    assert!(result.is_ok());
}

// ===== Token =====

#[test]
fn validate_token_after_login() {
    let (service, _path) = setup();
    let result = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    let info = service.validate_token(&result.token).unwrap();
    assert_eq!(info.username, "admin");
}

#[test]
fn logout_invalidates_token() {
    let (service, _path) = setup();
    let result = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    service.logout(&result.token).unwrap();
    assert!(service.validate_token(&result.token).is_err());
}

// ===== 改密码 =====

#[test]
fn change_password_success() {
    let (service, _path) = setup();
    let result = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    service
        .change_password(&result.token, "admin@123", "newPass@1", "newPass@1")
        .unwrap();

    // 旧密码登录失败
    assert!(service.login("admin", "admin@123", "127.0.0.1").is_err());
    // 新密码登录成功
    assert!(service.login("admin", "newPass@1", "127.0.0.1").is_ok());
}

#[test]
fn change_password_invalidates_all_tokens() {
    let (service, _path) = setup();
    let r1 = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    service
        .change_password(&r1.token, "admin@123", "newPass@1", "newPass@1")
        .unwrap();
    // 旧 token 失效
    assert!(service.validate_token(&r1.token).is_err());
}

#[test]
fn change_password_wrong_old_password() {
    let (service, _path) = setup();
    let result = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    let err = service
        .change_password(&result.token, "wrong_old", "newPass@1", "newPass@1")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::OldPasswordError
    );
}

#[test]
fn change_password_weak_new_password() {
    let (service, _path) = setup();
    let result = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    let err = service
        .change_password(&result.token, "admin@123", "weak", "weak")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::PasswordComplexityError
    );
}

#[test]
fn change_password_confirm_mismatch() {
    let (service, _path) = setup();
    let result = service.login("admin", "admin@123", "127.0.0.1").unwrap();
    let err = service
        .change_password(&result.token, "admin@123", "newPass@1", "different@1")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::PasswordConfirmMismatch
    );
}

// ===== 锁定过期后重试 =====

#[test]
fn lock_expired_then_wrong_password_does_not_relock_immediately() {
    let (service, _path) = setup();

    // 连续 5 次错误密码，触发锁定
    for _ in 0..5 {
        let _ = service.login("admin", "wrong", "127.0.0.1");
    }
    let err = service.login("admin", "admin@123", "127.0.0.1").unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::AccountLocked
    );

    // 手动将 lock_until 设为过去（模拟锁定超时）
    let user = service
        .storage()
        .user_query_by_username("admin")
        .unwrap()
        .unwrap();
    service
        .storage()
        .user_update_login_fail(user.id, 5, Some(1))
        .unwrap();

    // 锁定过期后，输入 1 次错误密码不应立即重新锁定
    let err = service.login("admin", "wrong", "127.0.0.1").unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::UserOrPasswordError
    );

    // 此时 fail_count 应为 1（不是 6），再输 3 次错仍不锁定
    for _ in 0..3 {
        let err = service.login("admin", "wrong", "127.0.0.1").unwrap_err();
        assert_eq!(
            err.to_result_code(),
            common::code::ResultCode::UserOrPasswordError
        );
    }

    // 第 5 次错误触发锁定（本次仍返回 UserOrPasswordError）
    let err = service.login("admin", "wrong", "127.0.0.1").unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::UserOrPasswordError
    );

    // 第 6 次尝试才返回 AccountLocked
    let err = service.login("admin", "admin@123", "127.0.0.1").unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::AccountLocked
    );
}

#[test]
fn lock_expired_then_correct_password_succeeds() {
    let (service, _path) = setup();

    // 锁定账号
    for _ in 0..5 {
        let _ = service.login("admin", "wrong", "127.0.0.1");
    }

    // 手动将 lock_until 设为过去
    let user = service
        .storage()
        .user_query_by_username("admin")
        .unwrap()
        .unwrap();
    service
        .storage()
        .user_update_login_fail(user.id, 5, Some(1))
        .unwrap();

    // 锁定过期后正确密码应成功登录
    let result = service.login("admin", "admin@123", "127.0.0.1");
    assert!(result.is_ok());
}

// ===== 密码工具 =====

#[test]
fn hash_password_produces_valid_hash() {
    let hash = password::hash_password("test@123").unwrap();
    assert!(hash.starts_with("$2b$12$"));
    assert!(password::verify_password("test@123", &hash).unwrap());
}

// ===== 用户管理 =====

#[test]
fn list_users_returns_builtin_users() {
    let (service, _path) = setup();
    let users = service.list_users().unwrap();
    // schema 自动插入了 admin、operator、audit 三个内置用户
    assert!(users.len() >= 3);
    let names: Vec<&str> = users.iter().map(|u| u.username.as_str()).collect();
    assert!(names.contains(&"admin"));
    assert!(names.contains(&"operator"));
    assert!(names.contains(&"audit"));
}

#[test]
fn create_user_success() {
    let (service, _path) = setup();
    let id = service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();
    assert!(id > 0);

    // 新用户可以登录
    let result = service.login("testuser", "Test@12345", "127.0.0.1").unwrap();
    assert_eq!(result.username, "testuser");
    assert_eq!(result.role, 1);
}

#[test]
fn create_user_duplicate_returns_error() {
    let (service, _path) = setup();
    service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();
    let err = service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::UsernameExists
    );
}

#[test]
fn create_user_password_mismatch() {
    let (service, _path) = setup();
    let err = service
        .create_user("testuser", 1, "Test@12345", "Different@1")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::PasswordConfirmMismatch
    );
}

#[test]
fn create_user_weak_password() {
    let (service, _path) = setup();
    let err = service
        .create_user("testuser", 1, "weak", "weak")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::PasswordComplexityError
    );
}

#[test]
fn create_user_invalid_role() {
    let (service, _path) = setup();
    let err = service
        .create_user("testuser", 5, "Test@12345", "Test@12345")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::ValidationFailed
    );
}

#[test]
fn delete_user_success() {
    let (service, _path) = setup();
    let id = service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();

    // 删除用户（current_user_id 不等于 target user id）
    service.delete_user("testuser", id + 999).unwrap();

    // 删除后不能登录
    assert!(service.login("testuser", "Test@12345", "127.0.0.1").is_err());
}

#[test]
fn delete_user_self_returns_forbidden() {
    let (service, _path) = setup();
    let id = service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();
    let err = service.delete_user("testuser", id).unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::SelfDeleteForbidden
    );
}

#[test]
fn delete_builtin_user_returns_error() {
    let (service, _path) = setup();
    let err = service.delete_user("admin", 9999).unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::BuiltinUserNoDelete
    );
}

#[test]
fn delete_user_invalidates_sessions() {
    let (service, _path) = setup();
    service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();
    let login_result = service
        .login("testuser", "Test@12345", "127.0.0.1")
        .unwrap();

    // 查出 user id 用于 current_user_id（用不同值）
    let user = service
        .storage()
        .user_query_by_username("testuser")
        .unwrap()
        .unwrap();
    service.delete_user("testuser", user.id + 999).unwrap();

    // token 应失效
    assert!(service.validate_token(&login_result.token).is_err());
}

#[test]
fn reset_password_success() {
    let (service, _path) = setup();
    service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();

    service
        .reset_password("testuser", "NewPass@99", "NewPass@99")
        .unwrap();

    // 旧密码登录失败
    assert!(service.login("testuser", "Test@12345", "127.0.0.1").is_err());
    // 新密码登录成功
    assert!(service.login("testuser", "NewPass@99", "127.0.0.1").is_ok());
}

#[test]
fn reset_password_clears_lock() {
    let (service, _path) = setup();
    service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();

    // 锁定账号
    for _ in 0..5 {
        let _ = service.login("testuser", "wrong", "127.0.0.1");
    }
    let err = service
        .login("testuser", "Test@12345", "127.0.0.1")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::AccountLocked
    );

    // 管理员重置密码
    service
        .reset_password("testuser", "NewPass@99", "NewPass@99")
        .unwrap();

    // 锁定解除，可以登录
    assert!(service.login("testuser", "NewPass@99", "127.0.0.1").is_ok());
}

#[test]
fn reset_password_confirm_mismatch() {
    let (service, _path) = setup();
    let err = service
        .reset_password("admin", "NewPass@99", "Different@1")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::PasswordConfirmMismatch
    );
}

#[test]
fn reset_password_invalidates_sessions() {
    let (service, _path) = setup();
    let login_result = service.login("admin", "admin@123", "127.0.0.1").unwrap();

    service
        .reset_password("admin", "NewPass@99", "NewPass@99")
        .unwrap();

    // 旧 token 失效
    assert!(service.validate_token(&login_result.token).is_err());
}

#[test]
fn create_user_deleted_username_cannot_reuse() {
    let (service, _path) = setup();
    let id = service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap();

    // 删除用户
    service.delete_user("testuser", id + 999).unwrap();

    // 同名不可复用
    let err = service
        .create_user("testuser", 1, "Test@12345", "Test@12345")
        .unwrap_err();
    assert_eq!(
        err.to_result_code(),
        common::code::ResultCode::UsernameDeletedReuse
    );
}
