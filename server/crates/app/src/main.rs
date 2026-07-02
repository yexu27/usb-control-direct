//! USB 安全管理装置端服务入口。

mod config;
mod logging;
mod shutdown;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, RwLock};

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use auth_session::{AuthService, SessionManager};
use config::AppConfig;
use file_access::engine::FileAccessEngine;
use file_access::gadget::GadgetRuntime;
use file_access::nbd::{
    disconnect_nbd_pool, read_nbd_partition_scan_status, NbdPartitionScanStatus,
};
use hid_access::hid_gadget::{discover_hidg_nodes_for_functions, HidFunctionNames, HidgNodes};
use license_upgrade::{
    LicenseValidator, ProductionLicenseValidator, SystemUpgradeManager, VirusdbUpgradeManager,
};
use log_audit::AuditService;
use malware_scan::clam_scanner::ClamScanner;
use malware_scan::scan_service::ScanService;
use policy_import_export::{FileKeyProvider, PolicyService};
use protocol_gateway::connection::{handle_connection, ConnectionManager};
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
use usb_identify::mount::RealMountOps;
use usb_identify::orchestrator::{DeviceEvent, DeviceOrchestrator};
use usb_identify::recovery::{clear_lun_backing_file, recover_raw_mounts};
use whitelist::WhitelistManager;

const NBD_POOL_SIZE: u32 = 4;

fn prepare_usb_gadget_startup(config: &AppConfig) -> Result<(GadgetRuntime, HidgNodes), String> {
    let bootstrap_config = file_access::gadget_bootstrap::GadgetBootstrapConfig {
        configfs_root: std::path::PathBuf::from("/sys/kernel/config/usb_gadget"),
        udc_root: std::path::PathBuf::from("/sys/class/udc"),
        gadget_name: config.gadget.name.clone(),
        config_name: config.gadget.config.clone(),
        udc: config.gadget.udc.clone(),
        keep_adb: config.gadget.keep_adb,
        storage_function: config.gadget.storage.function.clone(),
        storage_lun: config.gadget.storage.lun,
        keyboard_function: config.gadget.keyboard.function.clone(),
        mouse_function: config.gadget.mouse.function.clone(),
    };

    let gadget_runtime = file_access::gadget_bootstrap::GadgetBootstrap::prepare(bootstrap_config)
        .map_err(|e| format!("USB gadget bootstrap 失败: {e}"))?;

    let names = HidFunctionNames {
        keyboard: config.gadget.keyboard.function.clone(),
        mouse: config.gadget.mouse.function.clone(),
    };
    let hidg_nodes = discover_hidg_nodes_for_functions(gadget_runtime.gadget_dir(), &names)
        .map_err(|e| format!("hidg 节点发现失败: {e}"))?;

    Ok((gadget_runtime, hidg_nodes))
}

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

    let storage = Arc::new(Storage::open_with_pool_size(&db_path, 8).expect("数据库未就绪"));

    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&storage),
        SessionManager::new(),
    ));
    let audit_service = Arc::new(AuditService::new(Arc::clone(&storage), &db_path));
    let whitelist_manager =
        Arc::new(WhitelistManager::new(Arc::clone(&storage)).expect("白名单管理器初始化失败"));
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
    let (gadget_runtime, hidg_nodes) =
        prepare_usb_gadget_startup(&config).expect("USB gadget 启动准备失败");
    info!(
        gadget = %gadget_runtime.gadget_name(),
        function = %gadget_runtime.function_name(),
        lun = %gadget_runtime.lun_dir().display(),
        keyboard = %hidg_nodes.keyboard.display(),
        mouse = %hidg_nodes.mouse.display(),
        "USB gadget 启动准备完成"
    );

    let lun_file = gadget_runtime.lun_dir().join("file");
    if let Err(e) = clear_lun_backing_file(&lun_file) {
        warn!(error = %e, lun_file = %lun_file.display(), "启动恢复: 清空 LUN backing 失败");
    }
    disconnect_nbd_pool(NBD_POOL_SIZE);
    if let Err(e) = recover_raw_mounts(&RealMountOps, Path::new("/mnt/usb_raw")) {
        warn!(error = %e, "启动恢复: 清理原始 U 盘挂载失败");
    }
    match read_nbd_partition_scan_status() {
        Ok(NbdPartitionScanStatus::Disabled) => info!("NBD 本机分区扫描已关闭"),
        Ok(NbdPartitionScanStatus::Enabled(max_part)) => warn!(
            max_part,
            "NBD 本机分区扫描已开启，生产镜像应配置 nbd.max_part=0，避免 nbdXpY 事件风暴"
        ),
        Err(e) => warn!(error = %e, "读取 NBD 分区扫描配置失败"),
    }

    // ===== 实例化下游服务 =====
    let scan_service = Arc::new(ScanService::new(
        ClamScanner::new(&config.clamdscan_path),
        Arc::clone(&audit_service),
        &config.scan_log_dir,
    ));

    let file_access_engine = Arc::new(FileAccessEngine::new(
        Arc::clone(&storage),
        gadget_runtime.clone(),
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
    let orchestrator_cleanup = orchestrator.cleanup_handle();

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
    let shutdown_signal = shutdown::wait_for_shutdown_signal();
    tokio::pin!(shutdown_signal);

    loop {
        let (tcp_stream, peer_addr) = tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok(s) => s,
                    Err(e) => {
                        error!("接受连接失败: {}", e);
                        continue;
                    }
                }
            }
            _ = &mut shutdown_signal => {
                info!("收到服务退出信号，开始清理 USB 会话");
                let _ = shutdown_tx.send(());
                orchestrator_cleanup.shutdown_cleanup("service_shutdown").await;
                break;
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

        if let Err(e) = handle_connection(tls_stream, &router, guard, &state, source_ip).await {
            info!("连接结束: {}", e);
        }
    }
}
