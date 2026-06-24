//! USB 安全管理装置端服务入口。

mod logging;

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
use file_access::engine::FileAccessEngine;
use file_access::gadget::GadgetManager;
use hid_access::hid_gadget::{configure_hid_function, discover_hidg_nodes, KEYBOARD_FUNCTION, MOUSE_FUNCTION};
use hid_access::hid_report::{KEYBOARD_REPORT_DESC, KEYBOARD_REPORT_LEN, MOUSE_REPORT_DESC, MOUSE_REPORT_LEN};
use malware_scan::clam_scanner::ClamScanner;
use malware_scan::scan_service::ScanService;
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
/// Rust 服务运行日志目录。
const LOG_DIR: &str = "/var/log/usb-control";
/// SM2 授权验签公钥路径。
const LICENSE_PUBKEY_PATH: &str = "/etc/usb-control/keys/license_verify.pub";

#[tokio::main]
async fn main() {
    let _log_guards = logging::init_logging(std::path::Path::new(LOG_DIR));

    info!("USB 安全管理装置端服务启动");

    let db_path = PathBuf::from(DB_PATH);

    let storage_main = Storage::open(&db_path).expect("数据库初始化失败（main）");
    let storage_auth = Storage::open(&db_path).expect("数据库初始化失败（auth）");
    let storage_audit = Storage::open(&db_path).expect("数据库初始化失败（audit）");
    let storage_whitelist = Storage::open(&db_path).expect("数据库初始化失败（whitelist）");
    let storage_policy = Arc::new(Storage::open(&db_path).expect("数据库初始化失败（policy）"));
    let storage_file_access = Storage::open(&db_path).expect("数据库初始化失败（file_access）");

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
        Arc::clone(&whitelist_manager),
    ));

    let system_upgrade_mgr = Arc::new(SystemUpgradeManager::new(INSTALL_DIR, SERVICE_NAME));
    let virusdb_upgrade_mgr = Arc::new(VirusdbUpgradeManager::with_default_path());

    let license_validator: Arc<dyn LicenseValidator> = Arc::new(
        ProductionLicenseValidator::from_key_file(&PathBuf::from(LICENSE_PUBKEY_PATH))
            .expect("授权公钥加载失败"),
    );

    // ===== USB Gadget 统一初始化 =====
    let gadget_mgr = {
        let mgr = GadgetManager::new();
        let _ = mgr.unbind_udc();
        let _ = mgr.remove_config_links();

        // 先创建 functions（必须在写 config 属性之前）
        mgr.configure_mass_storage(&PathBuf::from("/dev/nbd0"), true)
            .expect("mass_storage function 配置失败");

        let functions_base = PathBuf::from("/sys/kernel/config/usb_gadget/rockchip/functions");
        configure_hid_function(&functions_base.join(KEYBOARD_FUNCTION), 1, KEYBOARD_REPORT_DESC, KEYBOARD_REPORT_LEN)
            .expect("HID keyboard function 配置失败");
        configure_hid_function(&functions_base.join(MOUSE_FUNCTION), 2, MOUSE_REPORT_DESC, MOUSE_REPORT_LEN)
            .expect("HID mouse function 配置失败");

        // 再写 gadget 属性（idVendor/strings/MaxPower）
        mgr.setup_gadget("USB Security Control Device")
            .expect("gadget 基础结构初始化失败");
        mgr
    };

    gadget_mgr.link_function("mass_storage.usb0").expect("链接 mass_storage 失败");
    gadget_mgr.link_function(KEYBOARD_FUNCTION).expect("链接 keyboard 失败");
    gadget_mgr.link_function(MOUSE_FUNCTION).expect("链接 mouse 失败");
    gadget_mgr.bind_udc().expect("UDC 绑定失败");

    let hidg_nodes = discover_hidg_nodes().expect("hidg 节点发现失败");
    info!("HID gadget: keyboard={}, mouse={}", hidg_nodes.keyboard.display(), hidg_nodes.mouse.display());

    // ===== 实例化下游服务 =====
    let scan_service = Arc::new(ScanService::new(
        ClamScanner::new("/usr/bin/clamdscan"),
        Arc::clone(&audit_service),
        &PathBuf::from("/var/log/usb-control/scan"),
    ));

    let file_access_engine = Arc::new(FileAccessEngine::new(
        storage_file_access,
        Arc::clone(&audit_service),
        "/dev/nbd0",
    ));

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
