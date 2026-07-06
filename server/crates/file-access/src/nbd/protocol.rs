//! NBD 协议结构和编解码。
//!
//! 本模块只处理 NBD 请求、响应和命令类型，不访问 fd、sysfs、ioctl 或业务后端。

/// NBD 请求魔数。
pub const NBD_REQUEST_MAGIC: u32 = 0x2560_9513;

/// NBD 响应魔数。
pub const NBD_REPLY_MAGIC: u32 = 0x6744_6698;

/// NBD 请求大小（字节）。
pub const NBD_REQUEST_SIZE: usize = 28;

/// NBD I/O 错误码（EIO = 5）。
pub const NBD_EIO: u32 = 5;

/// NBD 命令类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbdCommand {
    Read,
    Write,
    Disconnect,
    Flush,
    Unknown(u32),
}

/// NBD 请求。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NbdRequest {
    pub command: NbdCommand,
    pub handle: u64,
    pub from: u64,
    pub len: u32,
}

impl NbdRequest {
    /// 解析 28 字节 NBD 请求。
    pub fn parse(buf: &[u8; NBD_REQUEST_SIZE]) -> Option<Self> {
        let magic = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        if magic != NBD_REQUEST_MAGIC {
            return None;
        }

        let type_val = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let command = match type_val & 0xffff {
            0 => NbdCommand::Read,
            1 => NbdCommand::Write,
            2 => NbdCommand::Disconnect,
            3 => NbdCommand::Flush,
            other => NbdCommand::Unknown(other),
        };

        Some(Self {
            command,
            handle: u64::from_be_bytes(buf[8..16].try_into().unwrap()),
            from: u64::from_be_bytes(buf[16..24].try_into().unwrap()),
            len: u32::from_be_bytes(buf[24..28].try_into().unwrap()),
        })
    }
}

/// 构建 16 字节 NBD 响应头。
pub fn build_reply(handle: u64, error: u32) -> Vec<u8> {
    let mut reply = Vec::with_capacity(16);
    reply.extend_from_slice(&NBD_REPLY_MAGIC.to_be_bytes());
    reply.extend_from_slice(&error.to_be_bytes());
    reply.extend_from_slice(&handle.to_be_bytes());
    reply
}
