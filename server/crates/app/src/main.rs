//! USB 安全管理装置端服务入口。

mod config;
mod logging;

use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{error, info};

use auth_session::{AuthService, SessionManager};
use config::AppConfig;
use license_upgrade::{LicenseValidator, ProductionLicenseValidator, SystemUpgradeManager, VirusdbUpgradeManager};
use log_audit::AuditService;
use policy_import_export::{FileKeyProvider, PolicyService};
use protocol_gateway::connection::{ConnectionManager, handle_connection};
use protocol_gateway::context::AppState;
use protocol_gateway::handlers::register::{
    register_auth_handlers, register_file_access_handlers, register_license_handlers,
    register_log_handlers, register_policy_handlers, register_system_handlers,
    register_user_handlers, register_whitelist_handlers,
};
use protocol_gateway::router::Router;
use protocol_gateway::tls::create_tls_acceptor;
use storage::Storage;
use file_access::engine::FileAccessEngine;
use file_access::gadget::GadgetRuntime;
use hid_access::hid_gadget::discover_hidg_nodes;
use malware_scan::clam_scanner::ClamScanner;
use malware_scan::scan_service::ScanService;
use usb_identify::monitor::DeviceManager;
use usb_identify::orchestrator::{DeviceEvent, DeviceOrchestrator};
use whitelist::WhitelistManager;

#[tokio::main]
async fn main() {
    let config = AppConfig::load_from_args(std::env::args()).expect("启动配置加载失败");
    let _log_guards = logging::init_logging(&config.log_dir, &config.log_level_conf);

    info!(
        version = AppConfig::package_version(),
        config = %config.config_path.display(),
        "USB 安全管理装置端服务启动"
    );

    let db_path = config.database_path.clone();

    let storage = Arc::new(
        Storage::open_with_pool_size(&db_path, 8).expect("数据库未就绪"),
    );

    let auth_service = Arc::new(AuthService::new(Arc::clone(&storage), SessionManager::new()));
    let audit_service = Arc::new(AuditService::new(Arc::clone(&storage), &db_path));
    let whitelist_manager = Arc::new(
        WhitelistManager::new(Arc::clone(&storage)).expect("白名单管理器初始化失败"),
    );
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));

    let key_provider = Arc::new(FileKeyProvider::new(&config.policy_key_dir));
    let policy_service = Arc::new(PolicyService::new(
        Arc::clone(&storage),
        key_provider,
        Arc::clone(&whitelist_manager),
    ));

    let system_upgrade_mgr = Arc::new(SystemUpgradeManager::new(
        config.install_dir.clone(),
        config.service_name.clone(),
    ));
    let virusdb_upgrade_mgr = Arc::new(VirusdbUpgradeManager::with_default_path());

    let license_validator: Arc<dyn LicenseValidator> = Arc::new(
        ProductionLicenseValidator::from_key_file(&config.license_pubkey_path)
            .expect("授权公钥加载失败"),
    );

    // ===== USB Gadget 运行时检查 =====
    let gadget_runtime = GadgetRuntime::discover().expect("RK mass_storage LUN 发现失败");
    gadget_runtime
        .prepare_empty_lun()
        .expect("RK mass_storage LUN 初始化失败");
    info!(
        gadget = %gadget_runtime.gadget_name(),
        function = %gadget_runtime.function_name(),
        lun = %gadget_runtime.lun_dir().display(),
        "RK mass_storage LUN 已准备为空介质状态"
    );

    let hidg_nodes = discover_hidg_nodes().expect("hidg 节点发现失败");
    info!("HID gadget: keyboard={}, mouse={}", hidg_nodes.keyboard.display(), hidg_nodes.mouse.display());

    // ===== 实例化下游服务 =====
    let scan_service = Arc::new(ScanService::new(
        ClamScanner::new(&config.clamdscan_path),
        Arc::clone(&audit_service),
        &config.scan_log_dir,
    ));

    let file_access_engine = Arc::new(FileAccessEngine::new(Arc::clone(&storage)));

    let state = Arc::new(AppState {
        auth_service,
        audit_service,
        whitelist_manager,
        device_manager,
        storage,
        policy_service,
        license_validator,
        system_upgrade_mgr,
        virusdb_upgrade_mgr,
    });

    // 启动 udev 监听与主编排器
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
    let (event_tx, event_rx) = mpsc::unbounded_channel::<DeviceEvent>();

    let orchestrator = DeviceOrchestrator::new(
        event_rx,
        Arc::clone(&state.whitelist_manager),
        Arc::clone(&state.audit_service),
        Arc::clone(&state.device_manager),
        scan_service,
        file_access_engine,
        hidg_nodes,
    );

    // 启动编排器
    tokio::spawn(async move {
        orchestrator.run().await;
    });

    // 启动 udev 实时监听
    let udev_tx = event_tx.clone();
    let udev_shutdown = shutdown_tx.subscribe();
    tokio::spawn(async move {
        usb_identify::udev_monitor::start_udev_monitor(udev_tx, udev_shutdown).await;
    });

    // 启动时枚举存量设备
    let enum_tx = event_tx;
    tokio::task::spawn_blocking(move || {
        usb_identify::udev_monitor::enumerate_and_send(enum_tx);
    });

    // 启动 session 过期清理任务
    {
        let auth_for_cleanup = Arc::clone(&state.auth_service);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                auth_for_cleanup.session_manager().cleanup_expired();
            }
        });
    }

    let mut router = Router::new();
    register_auth_handlers(&mut router);
    register_whitelist_handlers(&mut router);
    register_file_access_handlers(&mut router);
    register_policy_handlers(&mut router);
    register_license_handlers(&mut router);
    register_system_handlers(&mut router);
    register_log_handlers(&mut router);
    register_user_handlers(&mut router);

    let cert_path = config.tls_cert_path.clone();
    let key_path = config.tls_key_path.clone();
    let (tls_acceptor, cert_fingerprint) =
        create_tls_acceptor(&cert_path, &key_path).expect("TLS 配置初始化失败");
    info!(fingerprint = %cert_fingerprint, "TLS 证书 SHA-256 指纹");

    let conn_mgr = ConnectionManager::new();

    let addr: SocketAddr = config.listen_addr.parse().expect("监听地址解析失败");
    let listener = TcpListener::bind(addr).await.expect("端口绑定失败");
    info!("TLS 监听: {}", addr);

    loop {
        let (tcp_stream, peer_addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                error!("接受连接失败: {}", e);
                continue;
            }
        };

        let guard = match conn_mgr.try_acquire() {
            Ok(g) => g,
            Err(_) => {
                info!("拒绝连接（已有管理端连接）: {}", peer_addr);
                continue;
            }
        };

        let tls_stream = match tls_acceptor.accept(tcp_stream).await {
            Ok(s) => s,
            Err(e) => {
                error!("TLS 握手失败: {}", e);
                continue;
            }
        };

        let source_ip = peer_addr.ip().to_string();

        if let Err(e) =
            handle_connection(tls_stream, &router, guard, &state, source_ip).await
        {
            info!("连接结束: {}", e);
        }
    }
}
