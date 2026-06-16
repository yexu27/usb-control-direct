//! 请求上下文。
//!
//! 每个请求帧经过 token 中间件后，生成 RequestContext 传递给 handler。

use std::sync::Arc;

use auth_session::session::SessionInfo;
use auth_session::AuthService;
use log_audit::AuditService;

/// 请求上下文。
pub struct RequestContext {
    /// 请求序列号。
    pub seq_id: u32,
    /// 已验证的会话信息（白名单 cmd 时为 None）。
    pub session: Option<SessionInfo>,
    /// 管理端来源 IP。
    pub source_ip: String,
    /// 鉴权服务（共享）。
    pub auth_service: Arc<AuthService>,
    /// 审计服务（共享）。
    pub audit_service: Arc<AuditService>,
}
