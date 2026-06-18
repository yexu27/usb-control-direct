//! 操作审计日志公共辅助函数。

use auth_session::session::SessionInfo;
use storage::model::OperationLogInsert;

use crate::context::RequestContext;

/// 记录操作日志。
///
/// 参数:
///   - ctx: 请求上下文。
///   - session: 当前会话信息。
///   - log_type: 日志分类（如 "user_management"、"system_management"）。
///   - action_type: 操作类型（如 "create"、"delete"）。
///   - target: 操作目标。
///   - result: 0=成功，1=失败。
///   - fail_reason: 失败原因（成功时为 None）。
pub fn log_operation(
    ctx: &RequestContext,
    session: &SessionInfo,
    log_type: &str,
    action_type: &str,
    target: &str,
    result: i32,
    fail_reason: Option<&str>,
) {
    let mut log = OperationLogInsert {
        op_time: 0,
        username: session.username.clone(),
        role: session.role,
        log_type: log_type.into(),
        action_type: Some(action_type.into()),
        target: Some(target.into()),
        before_value: None,
        after_value: None,
        related_file: None,
        related_version: None,
        result,
        fail_reason: fail_reason.map(|s| s.to_string()),
        source_ip: Some(ctx.source_ip.clone()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: None,
    };
    let _ = ctx.audit_service.log_operation(&mut log);
}
