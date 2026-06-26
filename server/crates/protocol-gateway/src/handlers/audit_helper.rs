//! 操作审计日志公共辅助函数。

use tracing::warn;

use auth_session::session::SessionInfo;
use storage::model::OperationLogInsert;

use crate::context::RequestContext;

/// 操作日志扩展字段。
#[derive(Default)]
pub struct OperationDetail {
    pub before_value: Option<String>,
    pub after_value: Option<String>,
    pub related_file: Option<String>,
    pub related_version: Option<String>,
    pub detail: Option<String>,
}

/// 记录操作日志（完整版，支持扩展字段）。
///
/// 参数:
///   - ctx: 请求上下文。
///   - session: 当前会话信息。
///   - log_type: 日志分类（如 "user_management"、"system_management"）。
///   - action_type: 操作类型（如 "login"、"whitelist_add"）。
///   - target: 操作目标。
///   - result: 0=成功，1=失败。
///   - fail_reason: 失败原因（成功时为 None）。
///   - ext: 扩展字段（before_value、after_value、related_file、related_version、detail）。
pub fn log_operation_full(
    ctx: &RequestContext,
    session: &SessionInfo,
    log_type: &str,
    action_type: &str,
    target: &str,
    result: i32,
    fail_reason: Option<&str>,
    ext: &OperationDetail,
) {
    let mut log = OperationLogInsert {
        op_time: 0,
        username: session.username.clone(),
        role: session.role,
        log_type: log_type.into(),
        action_type: Some(action_type.into()),
        target: Some(target.into()),
        before_value: ext.before_value.clone(),
        after_value: ext.after_value.clone(),
        related_file: ext.related_file.clone(),
        related_version: ext.related_version.clone(),
        result,
        fail_reason: fail_reason.map(|s| s.to_string()),
        source_ip: Some(ctx.source_ip.clone()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: ext.detail.clone(),
    };
    if let Err(e) = ctx.audit_service.log_operation(&mut log) {
        warn!("审计日志写入失败: {e}");
    }
}

/// 记录操作日志（简化版，无扩展字段）。
///
/// 参数:
///   - ctx: 请求上下文。
///   - session: 当前会话信息。
///   - log_type: 日志分类。
///   - action_type: 操作类型。
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
    log_operation_full(
        ctx,
        session,
        log_type,
        action_type,
        target,
        result,
        fail_reason,
        &OperationDetail::default(),
    );
}

/// 记录操作日志（从 ctx.session 获取会话，用于无法直接获取 session 引用的场景）。
///
/// 参数:
///   - ctx: 请求上下文（session 从 ctx.session 读取）。
///   - log_type: 日志分类。
///   - action_type: 操作类型。
///   - target: 操作目标（None 时存为空字符串）。
///   - result: 0=成功，1=失败。
///   - fail_reason: 失败原因（成功时为 None）。
///   - ext: 扩展字段。
pub fn log_operation_from_ctx(
    ctx: &RequestContext,
    log_type: &str,
    action_type: &str,
    target: Option<&str>,
    result: i32,
    fail_reason: Option<&str>,
    ext: &OperationDetail,
) {
    let mut log = OperationLogInsert {
        op_time: 0,
        username: ctx
            .session
            .as_ref()
            .map(|s| s.username.clone())
            .unwrap_or_default(),
        role: ctx.session.as_ref().map(|s| s.role).unwrap_or(-1),
        log_type: log_type.into(),
        action_type: Some(action_type.into()),
        target: target.map(|t| t.to_string()),
        before_value: ext.before_value.clone(),
        after_value: ext.after_value.clone(),
        related_file: ext.related_file.clone(),
        related_version: ext.related_version.clone(),
        result,
        fail_reason: fail_reason.map(|r| r.to_string()),
        source_ip: Some(ctx.source_ip.clone()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: ext.detail.clone(),
    };
    if let Err(e) = ctx.audit_service.log_operation(&mut log) {
        warn!("审计日志写入失败: {e}");
    }
}
