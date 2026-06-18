//! 帧流编解码器。
//!
//! 基于 common::frame::FrameHeader，处理 TCP 流上的不完整 buffer。
//! 读取时累积字节直到帧头 + payload 完整，写出时拼接帧头 + payload。

use common::frame::{self, FrameHeader, FRAME_HEADER_LEN};
use tracing::error;

use crate::error::GatewayError;

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

    let header = FrameHeader::decode(buf)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_decode_insufficient_header() {
        let buf = [0u8; 10];
        assert!(try_decode_frame(&buf).unwrap().is_none());
    }

    #[test]
    fn try_decode_insufficient_payload() {
        let payload = b"hello";
        let crc = frame::payload_crc32(payload);
        let header = FrameHeader::new(0x0001, 1, payload.len() as u32, crc).unwrap();
        let header_bytes = header.encode();
        let mut buf = Vec::from(header_bytes.as_slice());
        buf.extend_from_slice(&payload[..3]);
        assert!(try_decode_frame(&buf).unwrap().is_none());
    }

    #[test]
    fn try_decode_complete_frame() {
        let payload = b"hello";
        let encoded = encode_frame(0x0001, 42, payload).unwrap();
        let (header, decoded_payload, consumed) = try_decode_frame(&encoded).unwrap().unwrap();
        assert_eq!(header.msg_type, 0x0001);
        assert_eq!(header.seq_id, 42);
        assert_eq!(decoded_payload, payload);
        assert_eq!(consumed, encoded.len());
    }

    #[test]
    fn try_decode_bad_magic() {
        let mut buf = [0u8; FRAME_HEADER_LEN];
        buf[0..4].copy_from_slice(&0xDEADBEEFu32.to_be_bytes());
        assert!(try_decode_frame(&buf).is_err());
    }

    #[test]
    fn verify_crc_correct() {
        let payload = b"test payload";
        let crc = frame::payload_crc32(payload);
        let header = FrameHeader::new(0x0001, 1, payload.len() as u32, crc).unwrap();
        assert!(verify_crc(&header, payload));
    }

    #[test]
    fn verify_crc_tampered() {
        let payload = b"test payload";
        let header = FrameHeader::new(0x0001, 1, payload.len() as u32, 0xDEADBEEF).unwrap();
        assert!(!verify_crc(&header, payload));
    }

    #[test]
    fn encode_and_decode_round_trip() {
        let payload = b"round trip test";
        let encoded = encode_frame(0xFF01, 99, payload).unwrap();
        let (header, decoded, consumed) = try_decode_frame(&encoded).unwrap().unwrap();
        assert_eq!(header.msg_type, 0xFF01);
        assert_eq!(header.seq_id, 99);
        assert_eq!(decoded, payload);
        assert_eq!(consumed, encoded.len());
        assert!(verify_crc(&header, &decoded));
    }

    #[test]
    fn encode_empty_payload() {
        let encoded = encode_frame(0xFF02, 0, &[]).unwrap();
        let (header, decoded, _) = try_decode_frame(&encoded).unwrap().unwrap();
        assert_eq!(header.payload_len, 0);
        assert_eq!(header.crc32, 0);
        assert!(decoded.is_empty());
    }
}
