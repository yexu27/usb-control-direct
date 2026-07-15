//! 响应成功发送后的系统升级编排。

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use common::audit_const::{action_type, log_type};
use log_audit::AuditService;
use protocol_gateway::post_send::{PostSendAction, PostSendActionExecutor};
use protocol_gateway::GatewayError;
use storage::model::OperationLogInsert;
use system_upgrade::{UpgradeCoordinator, UpgradeTask};
use tracing::warn;

pub use crate::upgrade_result::UpgradeResultObserver;

/// “系统升级开始”业务日志端口。
pub trait UpgradeStartAudit: Send + Sync {
    fn record_start(&self, task: &UpgradeTask) -> Result<(), String>;
}

/// 生产审计适配器。
pub struct AuditUpgradeStart {
    audit: Arc<AuditService>,
}

impl AuditUpgradeStart {
    pub fn new(audit: Arc<AuditService>) -> Self {
        Self { audit }
    }
}

impl UpgradeStartAudit for AuditUpgradeStart {
    fn record_start(&self, task: &UpgradeTask) -> Result<(), String> {
        let mut item = OperationLogInsert {
            op_time: 0,
            username: task.username.clone(),
            role: task.role,
            log_type: log_type::PROGRAM_UPGRADE.into(),
            action_type: Some(action_type::SYSTEM_UPGRADE.into()),
            target: Some(task.target_version.to_string()),
            before_value: Some(task.source_version.to_string()),
            after_value: None,
            related_file: None,
            related_version: Some(task.target_version.to_string()),
            result: 0,
            fail_reason: None,
            source_ip: Some(task.source_ip.clone()),
            app_version: None,
            session_id: None,
            request_id: Some(format!("system-upgrade:{}:start", task.upgrade_id)),
            detail: Some("系统升级开始".into()),
        };
        self.audit
            .log_operation(&mut item)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

/// 串行执行 accept、尽力审计和 updater 调度，并抑制重复动作。
pub struct UpgradeDispatch {
    coordinator: Arc<UpgradeCoordinator>,
    audit: Arc<dyn UpgradeStartAudit>,
    observer: Arc<dyn UpgradeResultObserver>,
    completed: Mutex<HashSet<String>>,
}

impl UpgradeDispatch {
    pub fn new(
        coordinator: Arc<UpgradeCoordinator>,
        audit: Arc<dyn UpgradeStartAudit>,
        observer: Arc<dyn UpgradeResultObserver>,
    ) -> Self {
        Self {
            coordinator,
            audit,
            observer,
            completed: Mutex::new(HashSet::new()),
        }
    }
}

impl PostSendActionExecutor for UpgradeDispatch {
    fn execute(&self, action: PostSendAction) -> Result<(), GatewayError> {
        let PostSendAction::StartSystemUpgrade { upgrade_id } = action;
        let mut completed = self
            .completed
            .lock()
            .map_err(|_| GatewayError::PostSendAction("升级动作幂等锁已损坏".into()))?;
        if completed.contains(&upgrade_id) {
            return Ok(());
        }

        let task = self
            .coordinator
            .accept_after_response(&upgrade_id)
            .map_err(|error| GatewayError::PostSendAction(error.to_string()))?;
        if let Err(error) = self.audit.record_start(&task) {
            warn!(upgrade_id, reason = %error, "系统升级开始日志写入失败，继续调度 updater");
        }
        self.observer.observe(task);
        let result = self
            .coordinator
            .schedule(&upgrade_id)
            .map_err(|error| GatewayError::PostSendAction(error.to_string()));
        completed.insert(upgrade_id);
        result
    }

    fn cancel(&self, action: &PostSendAction) {
        let PostSendAction::StartSystemUpgrade { upgrade_id } = action;
        if let Err(error) = self.coordinator.response_failed(upgrade_id) {
            warn!(upgrade_id, reason = %error, "取消未发送的系统升级任务失败");
        }
    }
}
