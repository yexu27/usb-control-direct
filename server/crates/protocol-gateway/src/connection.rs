//! 单连接管理 + CRC 失败计数 + 心跳超时。
//!
//! 装置端同一时刻只接受一个管理端连接。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use tracing::{debug, error, info, warn};

use common::code::ResultCode;

use crate::codec;
use crate::context::{AppState, RequestContext};
use crate::error::GatewayError;
use crate::middleware;
use crate::post_send::{HandlerOutcome, PostSendActionExecutor};
use crate::router::Router;

/// CRC 连续失败断连阈值。
const MAX_CRC_FAILURES: u32 = 3;

/// 心跳间隔（秒）。
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// 心跳超时倍数：心跳间隔 × 3 + 5s 抖动窗口。
const HEARTBEAT_TIMEOUT_SECS: u64 = HEARTBEAT_INTERVAL_SECS * 3 + 5;

/// 心跳请求消息类型。
const CMD_HEARTBEAT: u32 = 0xFF01;

/// 心跳响应消息类型。
const CMD_HEARTBEAT_RSP: u32 = 0xFF02;

/// 通用响应消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// 单连接守卫。Drop 时释放连接槽位。
pub struct ConnectionGuard {
    connected: Arc<AtomicBool>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.connected.store(false, Ordering::SeqCst);
    }
}

/// 连接管理器。
pub struct ConnectionManager {
    connected: Arc<AtomicBool>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    /// 创建连接管理器。
    pub fn new() -> Self {
        ConnectionManager {
            connected: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 尝试获取连接槽位。
    ///
    /// 已有连接时返回 `GatewayError::DeviceBusy`。
    pub fn try_acquire(&self) -> Result<ConnectionGuard, GatewayError> {
        let was_connected = self.connected.swap(true, Ordering::SeqCst);
        if was_connected {
            return Err(GatewayError::DeviceBusy);
        }
        Ok(ConnectionGuard {
            connected: Arc::clone(&self.connected),
        })
    }
}

/// 构造错误响应帧。
fn make_error_response(seq_id: u32, code: ResultCode) -> Vec<u8> {
    let rsp = common::proto::RspCommon {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: format!("{}", code),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

/// 完整发送 handler 响应，并按发送结果恰好处理一次发送后动作。
///
/// `write_all` 和 `flush` 都成功后才消费动作并执行；任一步骤失败时仅取消动作。
/// 正式连接循环和集成测试共用该接口。
pub async fn send_handler_outcome<W>(
    stream: &mut W,
    executor: &dyn PostSendActionExecutor,
    outcome: HandlerOutcome,
) -> Result<(), GatewayError>
where
    W: AsyncWrite + Unpin,
{
    let HandlerOutcome {
        response,
        post_send_action,
    } = outcome;

    if let Err(error) = stream.write_all(&response).await {
        if let Some(action) = post_send_action.as_ref() {
            executor.cancel(action);
        }
        return Err(error.into());
    }
    if let Err(error) = stream.flush().await {
        if let Some(action) = post_send_action.as_ref() {
            executor.cancel(action);
        }
        return Err(error.into());
    }
    if let Some(action) = post_send_action {
        executor.execute(action)?;
    }
    Ok(())
}

/// 处理单个 TLS 连接的帧循环。
///
/// 参数:
///   - `stream`: TLS 连接流。
///   - `router`: 消息路由器。
///   - `_guard`: 连接守卫（持有期间占用连接槽位）。
///   - `state`: 应用全局共享状态（包含全部服务实例）。
///   - `source_ip`: 管理端来源 IP。
pub async fn handle_connection(
    mut stream: TlsStream<TcpStream>,
    router: &Router,
    _guard: ConnectionGuard,
    state: &AppState,
    source_ip: String,
) -> Result<(), GatewayError> {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    let mut crc_fail_count: u32 = 0;
    let mut last_activity = Instant::now();

    info!(source_ip = %source_ip, "管理端 TLS 连接已建立");

    loop {
        let timeout = Duration::from_secs(HEARTBEAT_TIMEOUT_SECS);
        let remaining = timeout.saturating_sub(last_activity.elapsed());

        tokio::select! {
            result = stream.read(&mut tmp) => {
                let n = result?;
                if n == 0 {
                    info!(source_ip = %source_ip, "管理端主动断开连接");
                    return Ok(());
                }
                buf.extend_from_slice(&tmp[..n]);
                last_activity = Instant::now();

                while let Some((header, payload, consumed)) = codec::try_decode_frame(&buf)? {
                    buf.drain(..consumed);

                    if !codec::verify_crc(&header, &payload) {
                        crc_fail_count += 1;
                        warn!(seq_id = header.seq_id, count = crc_fail_count, "CRC 校验失败");
                        if crc_fail_count >= MAX_CRC_FAILURES {
                            error!(source_ip = %source_ip, "CRC 连续失败 {} 次，断开连接", MAX_CRC_FAILURES);
                            return Err(GatewayError::CrcExceeded);
                        }
                        continue;
                    }
                    crc_fail_count = 0;

                    // 心跳处理（不刷新 session 活跃时间）
                    if header.msg_type == CMD_HEARTBEAT {
                        debug!("收到心跳请求");
                        let rsp = codec::encode_frame(CMD_HEARTBEAT_RSP, header.seq_id, &[])?;
                        stream.write_all(&rsp).await?;
                        continue;
                    }

                    debug!(msg_type = format_args!("0x{:04X}", header.msg_type), seq_id = header.seq_id, "收到业务请求");

                    // Token 中间件
                    let token_result = middleware::check_token(
                        header.msg_type,
                        &payload,
                        &state.auth_service,
                    );

                    let session = match token_result {
                        middleware::TokenResult::Whitelist => None,
                        middleware::TokenResult::Authenticated(info) => Some(info),
                        middleware::TokenResult::Failed(code) => {
                            let rsp = make_error_response(header.seq_id, code);
                            stream.write_all(&rsp).await?;
                            continue;
                        }
                    };

                    // 权限中间件
                    if let Some(ref info) = session {
                        let allowed = router.allowed_roles(header.msg_type);
                        if let Err(code) = middleware::check_permission(info, allowed) {
                            let rsp = make_error_response(header.seq_id, code);
                            stream.write_all(&rsp).await?;
                            continue;
                        }
                    }

                    // 构建 RequestContext 并分发
                    let ctx = RequestContext {
                        seq_id: header.seq_id,
                        session,
                        source_ip: source_ip.clone(),
                        auth_service: Arc::clone(&state.auth_service),
                        audit_service: Arc::clone(&state.audit_service),
                        whitelist_manager: Some(Arc::clone(&state.whitelist_manager)),
                        device_manager: Some(Arc::clone(&state.device_manager)),
                        device_runtime_registry: Some(Arc::clone(&state.device_runtime_registry)),
                        storage: Some(Arc::clone(&state.storage)),
                        policy_service: Some(Arc::clone(&state.policy_service)),
                        license_validator: Some(Arc::clone(&state.license_validator)),
                        system_upgrade_coordinator: Arc::clone(
                            &state.system_upgrade_coordinator,
                        ),
                        virusdb_upgrade_mgr: Some(Arc::clone(&state.virusdb_upgrade_mgr)),
                    };

                    let outcome = router.dispatch(&ctx, header.msg_type, &payload);
                    send_handler_outcome(
                        &mut stream,
                        state.post_send_action_executor.as_ref(),
                        outcome,
                    )
                    .await?;
                }
            }
            _ = tokio::time::sleep(remaining) => {
                warn!(source_ip = %source_ip, seconds = remaining.as_secs(), "管理端心跳超时，断开连接");
                return Err(GatewayError::HeartbeatTimeout);
            }
        }
    }
}
