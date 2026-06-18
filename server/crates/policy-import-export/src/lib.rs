//! S06 策略包导入导出服务。
//!
//! 使用 SM4-CBC 加密策略内容，SM3 计算摘要，SM2 签名。
//! 密钥通过 [`PolicyKeyProvider`] trait 注入。

pub mod crypto;
pub mod error;
pub mod format;
pub mod key_provider;
pub mod service;

pub use error::PolicyError;
pub use key_provider::{FileKeyProvider, PolicyKeyProvider, PolicyKeys};
pub use service::PolicyService;
