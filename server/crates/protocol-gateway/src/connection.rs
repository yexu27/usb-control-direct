//! 单连接管理 + CRC 失败计数 + 心跳超时。
//!
//! 装置端同一时刻只接受一个管理端连接。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

use crate::codec;
use crate::error::GatewayError;
use crate::router::Router;

/// CRC 连续失败断连阈值。
const MAX_CRC_FAILURES: u32 = 3;

/// 心跳间隔（秒）。
const HEARTBEAT_INTERVAL_SECS: u64 = 10;

/// 心跳超时倍数：心跳间隔 × 3 + 5s 抖动窗口。
const HEARTBEAT_TIMEOUT_SECS: u64 = HEARTBEAT_INTERVAL_SECS * 3 + 5;

/// 心跳请求消息类型。
const CMD_HEARTBEAT: u32 = 0xFF01;

/// 心跳响应消息类型。
const CMD_HEARTBEAT_RSP: u32 = 0xFF02;

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

/// 处理单个 TLS 连接的帧循环。
///
/// 参数:
///   - `stream`: TLS 连接流。
///   - `router`: 消息路由器。
///   - `_guard`: 连接守卫（持有期间占用连接槽位）。
pub async fn handle_connection(
    mut stream: TlsStream<TcpStream>,
    router: &Router,
    _guard: ConnectionGuard,
) -> Result<(), GatewayError> {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    let mut crc_fail_count: u32 = 0;
    let mut last_activity = Instant::now();

    loop {
        let timeout = Duration::from_secs(HEARTBEAT_TIMEOUT_SECS);
        let remaining = timeout.saturating_sub(last_activity.elapsed());

        tokio::select! {
            result = stream.read(&mut tmp) => {
                let n = result?;
                if n == 0 {
                    return Ok(());
                }
                buf.extend_from_slice(&tmp[..n]);
                last_activity = Instant::now();

                while let Some((header, payload, consumed)) = codec::try_decode_frame(&buf)? {
                    buf.drain(..consumed);

                    if !codec::verify_crc(&header, &payload) {
                        crc_fail_count += 1;
                        if crc_fail_count >= MAX_CRC_FAILURES {
                            return Err(GatewayError::CrcExceeded);
                        }
                        continue;
                    }
                    crc_fail_count = 0;

                    if header.msg_type == CMD_HEARTBEAT {
                        let rsp = codec::encode_frame(CMD_HEARTBEAT_RSP, header.seq_id, &[])?;
                        stream.write_all(&rsp).await?;
                        continue;
                    }

                    let response = router.dispatch(header.msg_type, header.seq_id, &payload);
                    stream.write_all(&response).await?;
                }
            }
            _ = tokio::time::sleep(remaining) => {
                return Err(GatewayError::HeartbeatTimeout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_manager_single_connection() {
        let mgr = ConnectionManager::new();
        let guard = mgr.try_acquire().unwrap();
        assert!(mgr.try_acquire().is_err());
        drop(guard);
        assert!(mgr.try_acquire().is_ok());
    }

    #[test]
    fn connection_guard_releases_on_drop() {
        let mgr = ConnectionManager::new();
        {
            let _guard = mgr.try_acquire().unwrap();
        }
        let _guard2 = mgr.try_acquire().unwrap();
    }
}
