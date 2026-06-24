//! 装置端服务运行日志初始化。
//!
//! 按模块分组输出到 4 个滚动日志文件 + stdout。
//! 日志文件按大小滚动（10MB），每组保留 5 个历史文件。
//! 支持通过 /etc/usb-control/log.conf 动态切换日志级别，无需重启进程。

use std::path::Path;
use std::time::SystemTime;

use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// 单个日志文件最大字节数（10 MB）。
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// 每组保留的历史文件数。
const MAX_FILE_COUNT: usize = 5;

/// 动态日志级别配置文件路径。
const LOG_LEVEL_CONF: &str = "/etc/usb-control/log.conf";

/// 配置文件轮询间隔（秒）。
const POLL_INTERVAL_SECS: u64 = 30;

/// 默认日志级别。
const DEFAULT_LOG_FILTER: &str = "info";

/// 日志文件分组定义。
struct LogGroup {
    file_prefix: &'static str,
    targets: &'static [&'static str],
}

const LOG_GROUPS: &[LogGroup] = &[
    LogGroup {
        file_prefix: "usb",
        targets: &["usb_identify", "hid_access", "malware_scan", "file_access"],
    },
    LogGroup {
        file_prefix: "protocol",
        targets: &["protocol_gateway", "auth_session"],
    },
    LogGroup {
        file_prefix: "system",
        targets: &["license_upgrade", "storage", "usb_control"],
    },
    LogGroup {
        file_prefix: "audit",
        targets: &["log_audit", "whitelist", "policy_import_export"],
    },
];

/// 从配置文件读取日志级别，文件不存在则返回默认级别。
fn load_log_filter() -> String {
    match std::fs::read_to_string(LOG_LEVEL_CONF) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                DEFAULT_LOG_FILTER.to_string()
            } else {
                trimmed.to_string()
            }
        }
        Err(_) => DEFAULT_LOG_FILTER.to_string(),
    }
}

/// 获取配置文件的修改时间，文件不存在返回 None。
fn conf_mtime() -> Option<SystemTime> {
    std::fs::metadata(LOG_LEVEL_CONF).ok().and_then(|m| m.modified().ok())
}

/// 启动配置文件轮询任务，检测到变化时热切换日志级别。
fn spawn_log_level_watcher(reload_handle: reload::Handle<EnvFilter, impl tracing::Subscriber + Send + Sync + 'static>) {
    tokio::spawn(async move {
        let mut last_mtime = conf_mtime();

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;

            let current_mtime = conf_mtime();
            if current_mtime == last_mtime {
                continue;
            }
            last_mtime = current_mtime;

            let filter_str = load_log_filter();
            match filter_str.parse::<EnvFilter>() {
                Ok(new_filter) => {
                    if let Err(e) = reload_handle.reload(new_filter) {
                        tracing::warn!("日志级别热切换失败: {}", e);
                    } else {
                        tracing::info!("日志级别已切换: {}", filter_str);
                    }
                }
                Err(e) => {
                    tracing::warn!("日志级别配置解析失败: {} (内容: {})", e, filter_str);
                }
            }
        }
    });
}

/// 初始化日志系统。
///
/// 参数:
/// - `log_dir`: 日志文件输出目录，目录不存在会自动创建。
///
/// 返回:
/// - `Vec<WorkerGuard>`，调用方必须持有到进程退出，
///   否则 non_blocking writer 会在 guard drop 时停止写入。
#[must_use = "guard 必须持有到进程退出，否则日志文件写入会停止"]
pub fn init_logging(log_dir: &Path) -> Vec<WorkerGuard> {
    std::fs::create_dir_all(log_dir).expect("日志目录创建失败");

    let mut guards = Vec::new();

    let initial_filter_str = load_log_filter();
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| initial_filter_str.parse().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER)));

    let (reload_filter, reload_handle) = reload::Layer::new(env_filter);

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_ansi(true);

    let mut file_layers: Vec<Box<dyn tracing_subscriber::Layer<_> + Send + Sync>> = Vec::new();

    for group in LOG_GROUPS {
        let file_name = format!("{}.log", group.file_prefix);
        let file_path = log_dir.join(&file_name);

        let appender = BasicRollingFileAppender::new(
            file_path,
            RollingConditionBasic::new().max_size(MAX_FILE_SIZE),
            MAX_FILE_COUNT,
        )
        .unwrap_or_else(|e| panic!("日志文件 {} 创建失败: {}", file_name, e));

        let (non_blocking, guard) = tracing_appender::non_blocking(appender);
        guards.push(guard);

        let mut target_filter = Targets::new();
        for target in group.targets {
            target_filter = target_filter.with_target(*target, Level::TRACE);
        }

        let layer = fmt::layer()
            .with_target(true)
            .with_ansi(false)
            .with_writer(non_blocking)
            .with_filter(target_filter);

        file_layers.push(Box::new(layer));
    }

    tracing_subscriber::registry()
        .with(reload_filter)
        .with(stdout_layer)
        .with(file_layers)
        .init();

    // RUST_LOG 环境变量优先级高于配置文件，有 RUST_LOG 时不启动轮询
    if std::env::var("RUST_LOG").is_err() {
        spawn_log_level_watcher(reload_handle);
    }

    guards
}
