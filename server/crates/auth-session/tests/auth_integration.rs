//! S08 鉴权服务集成测试。

use auth_session::password;
use auth_session::service::AuthService;
use auth_session::session::SessionManager;
use storage::Storage;
use tempfile::NamedTempFile;

fn setup() -> (AuthService, tempfile::TempPath) {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.into_temp_path();
    let storage = Storage::open(&path).unwrap();

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

// ===== 密码工具 =====

#[test]
fn hash_password_produces_valid_hash() {
    let hash = password::hash_password("test@123").unwrap();
    assert!(hash.starts_with("$2b$12$"));
    assert!(password::verify_password("test@123", &hash).unwrap());
}
