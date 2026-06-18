//! 请求上下文。
//!
//! 每个请求帧经过 token 中间件后，生成 RequestContext 传递给 handler。

use std::sync::{Arc, RwLock};

use auth_session::session::SessionInfo;
use auth_session::AuthService;
use license_upgrade::{LicenseValidator, SystemUpgradeManager, VirusdbUpgradeManager};
use log_audit::AuditService;
use policy_import_export::PolicyService;
use storage::Storage;
use usb_identify::monitor::DeviceManager;
use whitelist::WhitelistManager;

/// 应用全局共享状态。
///
/// 聚合全部服务实例，在 main 中初始化后传入 handle_connection，
/// 用于构建每个请求的 RequestContext。
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub audit_service: Arc<AuditService>,
    pub whitelist_manager: Arc<WhitelistManager>,
    pub device_manager: Arc<RwLock<DeviceManager>>,
    pub storage: Arc<Storage>,
    pub policy_service: Arc<PolicyService>,
    pub license_validator: Arc<dyn LicenseValidator>,
    pub system_upgrade_mgr: Arc<SystemUpgradeManager>,
    pub virusdb_upgrade_mgr: Arc<VirusdbUpgradeManager>,
}

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
    /// 白名单管理器（共享）。
    pub whitelist_manager: Option<Arc<WhitelistManager>>,
    /// 设备管理器（共享）。
    pub device_manager: Option<Arc<RwLock<DeviceManager>>>,
    /// 数据库存储（共享）。
    pub storage: Option<Arc<Storage>>,
    /// 策略导入导出服务。
    pub policy_service: Option<Arc<PolicyService>>,
    /// 授权校验器。
    pub license_validator: Option<Arc<dyn LicenseValidator>>,
    /// 系统升级管理器。
    pub system_upgrade_mgr: Option<Arc<SystemUpgradeManager>>,
    /// 病毒库升级管理器。
    pub virusdb_upgrade_mgr: Option<Arc<VirusdbUpgradeManager>>,
}

impl RequestContext {
    /// 获取 Storage 引用。
    pub fn storage(&self) -> Option<&Storage> {
        self.storage.as_ref().map(|s| s.as_ref())
    }

    /// 获取已验证的会话信息。
    ///
    /// 中间件已保证 role-restricted handler 进入时 session 一定存在，
    /// 此方法用于统一获取 session 而不必在每个 handler 重复检查。
    pub fn session_required(&self) -> &SessionInfo {
        self.session
            .as_ref()
            .expect("middleware guarantees session exists for role-restricted handlers")
    }
}
