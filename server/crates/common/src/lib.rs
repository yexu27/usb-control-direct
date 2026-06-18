//! 装置端公共类型库。
//!
//! 提供 ResultCode、设备/角色/权限枚举、协议帧头编解码、
//! 协议↔DB 枚举映射、thiserror 错误类型与 protobuf 生成类型。
//!
//! **约束**：本 crate 不包含 I/O、异步运行时、`std::thread::spawn`
//! 和全局可变状态。

pub mod code;
pub mod error;
pub mod frame;
pub mod mapping;
pub mod proto;
pub mod time;
pub mod types;
