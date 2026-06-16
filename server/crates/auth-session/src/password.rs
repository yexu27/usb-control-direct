//! 密码哈希与验证。
//!
//! 使用 bcrypt (cost=12)。

use crate::error::AuthError;

/// bcrypt cost 参数。
const BCRYPT_COST: u32 = 12;

/// 对明文密码进行 bcrypt 哈希。
pub fn hash_password(plain: &str) -> Result<String, AuthError> {
    let hash = bcrypt::hash(plain, BCRYPT_COST)?;
    Ok(hash)
}

/// 验证明文密码与 bcrypt hash 是否匹配。
pub fn verify_password(plain: &str, hash: &str) -> Result<bool, AuthError> {
    let matched = bcrypt::verify(plain, hash)?;
    Ok(matched)
}

/// 校验密码复杂度。
///
/// 规则（架构 07 §6）：
///   - 长度 >= 8
///   - 至少包含以下 3 类中的 2 类：字母、数字、特殊字符
pub fn validate_password_complexity(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }
    let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());

    let categories = [has_letter, has_digit, has_special]
        .iter()
        .filter(|&&v| v)
        .count();
    categories >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_correct_password() {
        let hash = hash_password("admin@123").unwrap();
        assert!(verify_password("admin@123", &hash).unwrap());
    }

    #[test]
    fn verify_wrong_password_returns_false() {
        let hash = hash_password("admin@123").unwrap();
        assert!(!verify_password("wrong_pass", &hash).unwrap());
    }

    #[test]
    fn complexity_valid_letter_digit_special() {
        assert!(validate_password_complexity("admin@123"));
    }

    #[test]
    fn complexity_valid_letter_digit() {
        assert!(validate_password_complexity("abcdef12"));
    }

    #[test]
    fn complexity_valid_letter_special() {
        assert!(validate_password_complexity("abcdef@!"));
    }

    #[test]
    fn complexity_valid_digit_special() {
        assert!(validate_password_complexity("123456@!"));
    }

    #[test]
    fn complexity_too_short() {
        assert!(!validate_password_complexity("ab@1"));
    }

    #[test]
    fn complexity_only_letters() {
        assert!(!validate_password_complexity("abcdefgh"));
    }

    #[test]
    fn complexity_only_digits() {
        assert!(!validate_password_complexity("12345678"));
    }
}
