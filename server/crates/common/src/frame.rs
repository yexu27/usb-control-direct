//! 私有协议帧头编解码。
//!
//! 协议帧 = 20 字节头 + Payload。CRC32 字段在本阶段仅做占位编解码，实际
//! 计算由 P01 协议层在写入/读取流时执行（写时算 payload CRC，读时校验）。

use crate::error::CommonError;

/// 协议帧魔数 "USBC" = 0x55534243。
pub const FRAME_MAGIC: u32 = 0x55534243;

/// Payload 长度上限：128 MiB。
pub const MAX_PAYLOAD_LEN: u32 = 128 * 1024 * 1024;

/// 帧头固定长度（字节）。
pub const FRAME_HEADER_LEN: usize = 20;

/// 协议帧头。所有整数字段使用大端编码。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    /// 魔数，固定为 `FRAME_MAGIC`。
    pub magic: u32,
    /// 消息类型码（对应架构 09 §3）。
    pub msg_type: u32,
    /// 请求序列号，响应需回填同一值。
    pub seq_id: u32,
    /// Payload 字节长度，上限 `MAX_PAYLOAD_LEN`。
    pub payload_len: u32,
    /// Payload 的 CRC32（IEEE 802.3 多项式，0xEDB88320）。Payload 为空时为 0。
    pub crc32: u32,
}

impl FrameHeader {
    /// 构造一个魔数为 `FRAME_MAGIC` 的帧头。
    ///
    /// 参数:
    ///   - `msg_type`: 消息类型码。
    ///   - `seq_id`: 序列号。
    ///   - `payload_len`: Payload 字节长度，不得超过 `MAX_PAYLOAD_LEN`。
    ///   - `crc32`: 已计算好的 Payload CRC32（Payload 为空时传 0）。
    ///
    /// 错误:
    ///   - `payload_len` 超过 `MAX_PAYLOAD_LEN` 时返回 `CommonError::InvalidFrame`。
    pub fn new(msg_type: u32, seq_id: u32, payload_len: u32, crc32: u32) -> Result<Self, CommonError> {
        if payload_len > MAX_PAYLOAD_LEN {
            return Err(CommonError::InvalidFrame(format!(
                "payload too large: {} bytes",
                payload_len
            )));
        }
        Ok(FrameHeader {
            magic: FRAME_MAGIC,
            msg_type,
            seq_id,
            payload_len,
            crc32,
        })
    }

    /// 将帧头编码为 20 字节（大端）。
    pub fn encode(&self) -> [u8; FRAME_HEADER_LEN] {
        let mut buf = [0u8; FRAME_HEADER_LEN];
        buf[0..4].copy_from_slice(&self.magic.to_be_bytes());
        buf[4..8].copy_from_slice(&self.msg_type.to_be_bytes());
        buf[8..12].copy_from_slice(&self.seq_id.to_be_bytes());
        buf[12..16].copy_from_slice(&self.payload_len.to_be_bytes());
        buf[16..20].copy_from_slice(&self.crc32.to_be_bytes());
        buf
    }

    /// 从 20 字节切片解码帧头。
    ///
    /// 失败条件：
    /// - 输入长度不足 20 字节
    /// - 魔数不等于 `FRAME_MAGIC`
    /// - payload_len 超过 `MAX_PAYLOAD_LEN`
    pub fn decode(input: &[u8]) -> Result<Self, CommonError> {
        if input.len() < FRAME_HEADER_LEN {
            return Err(CommonError::InvalidFrame(format!(
                "header too short: {} bytes",
                input.len()
            )));
        }
        let magic = u32::from_be_bytes(input[0..4].try_into().unwrap());
        if magic != FRAME_MAGIC {
            return Err(CommonError::InvalidFrame(format!(
                "magic mismatch: 0x{:08X}",
                magic
            )));
        }
        let msg_type = u32::from_be_bytes(input[4..8].try_into().unwrap());
        let seq_id = u32::from_be_bytes(input[8..12].try_into().unwrap());
        let payload_len = u32::from_be_bytes(input[12..16].try_into().unwrap());
        if payload_len > MAX_PAYLOAD_LEN {
            return Err(CommonError::InvalidFrame(format!(
                "payload too large: {} bytes",
                payload_len
            )));
        }
        let crc32 = u32::from_be_bytes(input[16..20].try_into().unwrap());
        Ok(FrameHeader {
            magic,
            msg_type,
            seq_id,
            payload_len,
            crc32,
        })
    }
}

/// 计算 Payload 的 CRC32（IEEE 802.3 / 0xEDB88320）。Payload 为空时返回 0。
pub fn payload_crc32(payload: &[u8]) -> u32 {
    if payload.is_empty() {
        0
    } else {
        crc32fast::hash(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_header_encode_decode_round_trip() {
        let h = FrameHeader::new(0x0001, 42, 123, 0xDEAD_BEEF).unwrap();
        let bytes = h.encode();
        assert_eq!(bytes.len(), FRAME_HEADER_LEN);
        let decoded = FrameHeader::decode(&bytes).unwrap();
        assert_eq!(decoded, h);
    }

    #[test]
    fn frame_header_magic_is_usbc() {
        let h = FrameHeader::new(0, 0, 0, 0).unwrap();
        let bytes = h.encode();
        assert_eq!(&bytes[0..4], b"USBC");
    }

    #[test]
    fn frame_header_decode_too_short_fails() {
        let bytes = [0u8; 10];
        assert!(FrameHeader::decode(&bytes).is_err());
    }

    #[test]
    fn frame_header_decode_bad_magic_fails() {
        let mut bytes = [0u8; FRAME_HEADER_LEN];
        bytes[0..4].copy_from_slice(&0xDEAD_BEEFu32.to_be_bytes());
        assert!(FrameHeader::decode(&bytes).is_err());
    }

    #[test]
    fn frame_header_new_payload_too_large_fails() {
        assert!(FrameHeader::new(0, 0, MAX_PAYLOAD_LEN + 1, 0).is_err());
    }

    #[test]
    fn frame_header_new_max_payload_ok() {
        assert!(FrameHeader::new(0, 0, MAX_PAYLOAD_LEN, 0).is_ok());
    }

    #[test]
    fn frame_header_decode_max_payload_ok() {
        let h = FrameHeader::new(0, 0, MAX_PAYLOAD_LEN, 0).unwrap();
        let bytes = h.encode();
        assert!(FrameHeader::decode(&bytes).is_ok());
    }

    #[test]
    fn payload_crc32_empty_is_zero() {
        assert_eq!(payload_crc32(&[]), 0);
    }

    #[test]
    fn payload_crc32_known_value() {
        // CRC32 of b"123456789" = 0xCBF43926（IEEE 802.3 标准向量）
        assert_eq!(payload_crc32(b"123456789"), 0xCBF4_3926);
    }
}
