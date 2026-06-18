//! 日志多条件分页查询。

use crate::error::AuditError;

/// 半年时间常量（秒）：180 天。
const SIX_MONTHS_SECS: i64 = 180 * 24 * 3600;

/// 日志类型枚举（从协议 log_type 字符串解析）。
pub enum LogType {
    /// USB 审计日志。
    UsbAudit,
    /// 恶意代码检测日志。
    Malware,
    /// 操作日志。
    Operation,
}

impl LogType {
    /// 从字符串解析日志类型。
    ///
    /// 参数:
    /// - `s`: 日志类型字符串。
    ///
    /// 返回:
    /// - 匹配时返回对应枚举值，否则返回 None。
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "usb_audit" => Some(LogType::UsbAudit),
            "malware" => Some(LogType::Malware),
            "operation" => Some(LogType::Operation),
            _ => None,
        }
    }
}

/// 校验日志清理时间范围：end_time 必须 <= now - 6个月。
///
/// 参数:
/// - `end_time`: 清理截止时间戳（Unix 秒）。
///
/// 返回:
/// - 校验通过返回 Ok，违反半年保留期返回 RetentionViolation 错误。
pub fn validate_delete_time(end_time: i64) -> Result<(), AuditError> {
    let now = crate::audit::now_unix();
    let six_months_ago = now - SIX_MONTHS_SECS;
    if end_time > six_months_ago {
        return Err(AuditError::RetentionViolation);
    }
    Ok(())
}
