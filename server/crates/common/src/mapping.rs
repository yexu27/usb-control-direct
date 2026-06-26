//! 协议层 ↔ 数据库层枚举字段映射函数。
//!
//! 所有协议消息解析与响应组装中涉及枚举字符串与整数互转的位置，必须复用
//! 本模块的函数，不允许各业务模块自行实现，避免出现不一致。
//!
//! 时间字段（Unix 秒，int64）协议层与数据库层同型，不在本模块。

use crate::error::CommonError;
use crate::types::{Permission, Role};

// =====================================================================
// Role 映射
// =====================================================================

/// 角色协议字符串转数据库整数。
///
/// 参数:
///   - `value`: `admin` / `operator` / `auditor`。
///
/// 返回:
///   - 0 / 1 / 2；输入未识别时返回 `CommonError::UnknownEnum`。
pub fn role_str_to_int(value: &str) -> Result<i32, CommonError> {
    match value {
        "admin" => Ok(0),
        "operator" => Ok(1),
        "auditor" => Ok(2),
        other => Err(CommonError::UnknownEnum(format!("role={}", other))),
    }
}

/// 角色数据库整数转协议字符串。
///
/// 参数:
///   - `value`: T08.role 的整数值（0/1/2）。
///
/// 返回:
///   - `admin` / `operator` / `auditor`；输入越界时返回错误。
pub fn role_int_to_str(value: i32) -> Result<&'static str, CommonError> {
    match value {
        0 => Ok("admin"),
        1 => Ok("operator"),
        2 => Ok("auditor"),
        other => Err(CommonError::UnknownEnum(format!("role_int={}", other))),
    }
}

/// 协议字符串转 Role 枚举。
pub fn role_str_to_enum(value: &str) -> Result<Role, CommonError> {
    match value {
        "admin" => Ok(Role::Admin),
        "operator" => Ok(Role::Operator),
        "auditor" => Ok(Role::Auditor),
        other => Err(CommonError::UnknownEnum(format!("role={}", other))),
    }
}

// =====================================================================
// Permission 映射
// =====================================================================

/// 权限协议字符串转数据库整数。
///
/// 参数:
///   - `value`: `readonly` / `readwrite`。
pub fn permission_str_to_int(value: &str) -> Result<i32, CommonError> {
    match value {
        "readonly" => Ok(0),
        "readwrite" => Ok(1),
        other => Err(CommonError::UnknownEnum(format!("permission={}", other))),
    }
}

/// 权限数据库整数转协议字符串。
pub fn permission_int_to_str(value: i32) -> Result<&'static str, CommonError> {
    match value {
        0 => Ok("readonly"),
        1 => Ok("readwrite"),
        other => Err(CommonError::UnknownEnum(format!("permission_int={}", other))),
    }
}

/// 协议字符串转 Permission 枚举。
pub fn permission_str_to_enum(value: &str) -> Result<Permission, CommonError> {
    match value {
        "readonly" => Ok(Permission::ReadOnly),
        "readwrite" => Ok(Permission::ReadWrite),
        other => Err(CommonError::UnknownEnum(format!("permission={}", other))),
    }
}

// =====================================================================
// ScanResult 映射（本阶段约定 clean/infected/failed/cancelled，详见 P00 计划）
// =====================================================================

/// 扫描结果协议字符串转数据库整数。
///
/// 参数:
///   - `value`: `clean` / `infected` / `failed` / `cancelled`。
pub fn scan_result_str_to_int(value: &str) -> Result<i32, CommonError> {
    match value {
        "clean" => Ok(0),
        "infected" => Ok(1),
        "failed" => Ok(2),
        "cancelled" => Ok(3),
        other => Err(CommonError::UnknownEnum(format!("scan_result={}", other))),
    }
}

/// 扫描结果数据库整数转协议字符串。
pub fn scan_result_int_to_str(value: i32) -> Result<&'static str, CommonError> {
    match value {
        0 => Ok("clean"),
        1 => Ok("infected"),
        2 => Ok("failed"),
        3 => Ok("cancelled"),
        other => Err(CommonError::UnknownEnum(format!("scan_result_int={}", other))),
    }
}

// =====================================================================
// ProcessResult 映射
// =====================================================================

/// 处理结果协议字符串转数据库整数。
///
/// 参数:
///   - `value`: `marked` / `blocked` / `failed` / `no_action`。
pub fn process_result_str_to_int(value: &str) -> Result<i32, CommonError> {
    match value {
        "marked" => Ok(0),
        "blocked" => Ok(1),
        "failed" => Ok(2),
        "no_action" => Ok(3),
        other => Err(CommonError::UnknownEnum(format!("process_result={}", other))),
    }
}

/// 处理结果数据库整数转协议字符串。
pub fn process_result_int_to_str(value: i32) -> Result<&'static str, CommonError> {
    match value {
        0 => Ok("marked"),
        1 => Ok("blocked"),
        2 => Ok("failed"),
        3 => Ok("no_action"),
        other => Err(CommonError::UnknownEnum(format!("process_result_int={}", other))),
    }
}

// =====================================================================
// AddMethod 映射
// =====================================================================

/// 添加方式协议字符串转数据库整数。
///
/// 参数:
///   - `value`: `device` / `management`。
///
/// 返回:
///   - 0 / 1；输入未识别时返回 `CommonError::UnknownEnum`。
pub fn add_method_str_to_int(value: &str) -> Result<i32, CommonError> {
    match value {
        "device" => Ok(0),
        "management" => Ok(1),
        other => Err(CommonError::UnknownEnum(format!("add_method={}", other))),
    }
}

/// 添加方式数据库整数转协议字符串。
///
/// 参数:
///   - `value`: 数据库中存储的整数值（0/1）。
///
/// 返回:
///   - `device` / `management`；输入越界时返回错误。
pub fn add_method_int_to_str(value: i32) -> Result<&'static str, CommonError> {
    match value {
        0 => Ok("device"),
        1 => Ok("management"),
        other => Err(CommonError::UnknownEnum(format!("add_method_int={}", other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_round_trip() {
        for (s, i) in [("admin", 0), ("operator", 1), ("auditor", 2)] {
            assert_eq!(role_str_to_int(s).unwrap(), i);
            assert_eq!(role_int_to_str(i).unwrap(), s);
        }
    }

    #[test]
    fn role_unknown_string_returns_error() {
        assert!(role_str_to_int("super_admin").is_err());
    }

    #[test]
    fn role_unknown_int_returns_error() {
        assert!(role_int_to_str(9).is_err());
    }

    #[test]
    fn permission_round_trip() {
        for (s, i) in [("readonly", 0), ("readwrite", 1)] {
            assert_eq!(permission_str_to_int(s).unwrap(), i);
            assert_eq!(permission_int_to_str(i).unwrap(), s);
        }
    }

    #[test]
    fn scan_result_round_trip() {
        for (s, i) in [("clean", 0), ("infected", 1), ("failed", 2), ("cancelled", 3)] {
            assert_eq!(scan_result_str_to_int(s).unwrap(), i);
            assert_eq!(scan_result_int_to_str(i).unwrap(), s);
        }
    }

    #[test]
    fn add_method_round_trip() {
        for (s, i) in [("device", 0), ("management", 1)] {
            assert_eq!(add_method_str_to_int(s).unwrap(), i);
            assert_eq!(add_method_int_to_str(i).unwrap(), s);
        }
    }

    #[test]
    fn add_method_unknown_returns_error() {
        assert!(add_method_str_to_int("unknown").is_err());
        assert!(add_method_int_to_str(9).is_err());
    }

    #[test]
    fn process_result_round_trip() {
        for (s, i) in [("marked", 0), ("blocked", 1), ("failed", 2), ("no_action", 3)] {
            assert_eq!(process_result_str_to_int(s).unwrap(), i);
            assert_eq!(process_result_int_to_str(i).unwrap(), s);
        }
    }

    #[test]
    fn process_result_unknown_returns_error() {
        assert!(process_result_str_to_int("unknown").is_err());
        assert!(process_result_int_to_str(9).is_err());
    }
}
