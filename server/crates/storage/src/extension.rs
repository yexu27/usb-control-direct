//! 文件后缀规范化与校验。

use crate::StorageError;

/// 规范化并校验文件后缀。
///
/// 参数:
/// - `input`: 以单个点号开头的文件后缀。
///
/// 返回:
/// - 成功时返回去除首尾空白、转换为 ASCII 小写的后缀。
pub fn normalize_extension(input: &str) -> Result<String, StorageError> {
    let extension = input.trim();
    let body = extension
        .strip_prefix('.')
        .ok_or_else(invalid_extension)?;
    let mut chars = body.chars();
    let first = chars.next().ok_or_else(invalid_extension)?;
    let valid_first = first.is_ascii_alphanumeric();
    let valid_remaining = chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if !valid_first || !valid_remaining {
        return Err(invalid_extension());
    }

    Ok(extension.to_ascii_lowercase())
}

fn invalid_extension() -> StorageError {
    StorageError::Validation("文件后缀格式错误".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_extension_trims_and_lowercases_ascii() {
        assert_eq!(normalize_extension("  .PS1  ").unwrap(), ".ps1");
    }

    #[test]
    fn normalize_extension_accepts_ascii_letters_digits_underscore_and_hyphen() {
        assert_eq!(normalize_extension(".A0_b-C").unwrap(), ".a0_b-c");
    }

    #[test]
    fn normalize_extension_rejects_invalid_formats() {
        for input in [
            "", "   ", "ps1", ".", "..ps1", ".ps.1", "._ps1", ".-ps1", ".脚本",
            ".ps 1",
        ] {
            assert!(normalize_extension(input).is_err(), "应拒绝后缀: {input:?}");
        }
    }
}
