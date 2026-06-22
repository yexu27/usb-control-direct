//! USB 安全管理装置端服务入口。

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{error, info};

use auth_session::{AuthService, SessionManager};
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
use usb_identify::monitor::DeviceManager;
use usb_identify::orchestrator::{DeviceEvent, DeviceOrchestrator};
use whitelist::WhitelistManager;

/// 数据库路径。
const DB_PATH: &str = "/var/lib/usb-control/data.db";
/// TLS 证书路径。
const CERT_PATH: &str = "/etc/usb-control/certs/server.crt";
/// TLS 私钥路径。
const KEY_PATH: &str = "/etc/usb-control/certs/server.key";
/// 监听地址。
const LISTEN_ADDR: &str = "0.0.0.0:9600";
/// 安装目录。
const INSTALL_DIR: &str = "/opt/usb-control";
/// systemd 服务名。
const SERVICE_NAME: &str = "usb-control";
/// 策略密钥目录。
const POLICY_KEY_DIR: &str = "/etc/usb-control/keys";
/// SM2 授权验签公钥路径。
const LICENSE_PUBKEY_PATH: &str = "/etc/usb-control/keys/license_verify.pub";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("USB 安全管理装置端服务启动");

    let db_path = PathBuf::from(DB_PATH);

    let storage_main = Storage::open(&db_path).expect("数据库初始化失败（main）");
    let storage_auth = Storage::open(&db_path).expect("数据库初始化失败（auth）");
    let storage_audit = Storage::open(&db_path).expect("数据库初始化失败（audit）");
    let storage_whitelist = Storage::open(&db_path).expect("数据库初始化失败（whitelist）");
    let storage_policy = Arc::new(Storage::open(&db_path).expect("数据库初始化失败（policy）"));

    let auth_service = Arc::new(AuthService::new(storage_auth, SessionManager::new()));
    let audit_service = Arc::new(AuditService::new(storage_audit, &db_path));
    let whitelist_manager = Arc::new(
        WhitelistManager::new(storage_whitelist).expect("白名单管理器初始化失败"),
    );
    let device_manager = Arc::new(RwLock::new(DeviceManager::new()));
    let storage = Arc::new(storage_main);

    let key_provider = Arc::new(FileKeyProvider::new(POLICY_KEY_DIR));
    let policy_service = Arc::new(PolicyService::new(
        Arc::clone(&storage_policy),
        key_provider,
    ));

    let system_upgrade_mgr = Arc::new(SystemUpgradeManager::new(INSTALL_DIR, SERVICE_NAME));
    let virusdb_upgrade_mgr = Arc::new(VirusdbUpgradeManager::with_default_path());

    let license_validator: Arc<dyn LicenseValidator> = Arc::new(
        ProductionLicenseValidator::from_key_file(&PathBuf::from(LICENSE_PUBKEY_PATH))
            .expect("授权公钥加载失败"),
    );

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

    let mut router = Router::new();
    register_auth_handlers(&mut router);
    register_whitelist_handlers(&mut router);
    register_file_access_handlers(&mut router);
    register_policy_handlers(&mut router);
    register_license_handlers(&mut router);
    register_system_handlers(&mut router);
    register_log_handlers(&mut router);
    register_user_handlers(&mut router);

    let cert_path = PathBuf::from(CERT_PATH);
    let key_path = PathBuf::from(KEY_PATH);
    let tls_acceptor =
        create_tls_acceptor(&cert_path, &key_path).expect("TLS 配置初始化失败");

    let conn_mgr = ConnectionManager::new();

    let addr: SocketAddr = LISTEN_ADDR.parse().expect("监听地址解析失败");
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
