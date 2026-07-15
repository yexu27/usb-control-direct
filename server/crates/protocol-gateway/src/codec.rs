//! 帧流编解码器。
//!
//! 基于 common::frame::FrameHeader，处理 TCP 流上的不完整 buffer。
//! 读取时累积字节直到帧头 + payload 完整，写出时拼接帧头 + payload。

use common::frame::{self, FrameHeader, FRAME_HEADER_LEN};
use tracing::{debug, error};

use crate::error::GatewayError;

/// 协议 payload 统一上限：128 MiB。
pub const MAX_PAYLOAD_SIZE: usize = 128 * 1024 * 1024;

/// 从 buffer 中尝试解码一帧。
///
/// 返回:
///   - `Ok(Some((header, payload, consumed)))`: 成功解码，consumed 为消耗的字节数。
///   - `Ok(None)`: 数据不足，需要更多字节。
///   - `Err(...)`: 帧格式错误（magic/长度等）。
pub fn try_decode_frame(buf: &[u8]) -> Result<Option<(FrameHeader, Vec<u8>, usize)>, GatewayError> {
    if buf.len() < FRAME_HEADER_LEN {
        return Ok(None);
    }

    let declared_payload_len = u32::from_be_bytes(
        buf[12..16]
            .try_into()
            .expect("frame header length was checked"),
    );
    if declared_payload_len as usize > MAX_PAYLOAD_SIZE {
        return Err(GatewayError::PayloadTooLarge {
            declared: declared_payload_len,
        });
    }

    let header = match FrameHeader::decode(buf) {
        Ok(h) => h,
        Err(e) => {
            debug!(reason = %e, "帧解码失败");
            return Err(e.into());
        }
    };
    let total_len = FRAME_HEADER_LEN + header.payload_len as usize;

    if buf.len() < total_len {
        return Ok(None);
    }

    let payload = buf[FRAME_HEADER_LEN..total_len].to_vec();
    Ok(Some((header, payload, total_len)))
}

/// 校验 payload 的 CRC32。
pub fn verify_crc(header: &FrameHeader, payload: &[u8]) -> bool {
    let expected = frame::payload_crc32(payload);
    header.crc32 == expected
}

/// 编码帧为字节流（帧头 + payload）。
pub fn encode_frame(msg_type: u32, seq_id: u32, payload: &[u8]) -> Result<Vec<u8>, GatewayError> {
    if payload.len() > MAX_PAYLOAD_SIZE {
        return Err(GatewayError::PayloadTooLarge {
            declared: u32::try_from(payload.len()).unwrap_or(u32::MAX),
        });
    }
    let crc = frame::payload_crc32(payload);
    let header = FrameHeader::new(msg_type, seq_id, payload.len() as u32, crc).map_err(|e| {
        error!(
            "encode_frame 失败: msg_type=0x{:04X}, seq_id={}, payload_len={}, error={}",
            msg_type,
            seq_id,
            payload.len(),
            e
        );
        e
    })?;
    let header_bytes = header.encode();
    let mut out = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
    out.extend_from_slice(&header_bytes);
    out.extend_from_slice(payload);
    Ok(out)
}
