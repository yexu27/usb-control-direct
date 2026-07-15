//! 系统升级受理错误到现有协议结果码的唯一映射。

use common::code::ResultCode;
use system_upgrade::{UpgradeError, UpgradePreflightFailure};

/// 可直接写入 `RspCommon` 的稳定错误信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpgradeProtocolError {
    pub result_code: ResultCode,
    pub message: &'static str,
}

pub fn map_upgrade_error(error: &UpgradeError) -> UpgradeProtocolError {
    let (result_code, message) = match error {
        UpgradeError::InvalidVersion(_) => (ResultCode::UpgradeFormatError, "系统版本格式无效"),
        UpgradeError::VersionNotGreater => (ResultCode::VersionTooLow, "目标版本必须高于当前版本"),
        UpgradeError::DigestMismatch => (ResultCode::UpgradeChecksumError, "升级包摘要不匹配"),
        UpgradeError::SignatureInvalid => (ResultCode::UpgradeFormatError, "升级包签名无效"),
        UpgradeError::ProductMismatch => (ResultCode::UpgradeFormatError, "升级包产品不匹配"),
        UpgradeError::ArchitectureMismatch => (ResultCode::UpgradeFormatError, "升级包架构不匹配"),
        UpgradeError::SchemaIncompatible => {
            (ResultCode::UpgradeFormatError, "升级包数据库 Schema 不兼容")
        }
        UpgradeError::Format(_) | UpgradeError::DebInspection(_) => {
            (ResultCode::UpgradeFormatError, "升级包格式错误")
        }
        UpgradeError::Busy => (ResultCode::DeviceBusy, "已有系统升级任务正在处理"),
        UpgradeError::Preflight(failure) => match failure {
            UpgradePreflightFailure::InsufficientSpace { .. } => {
                (ResultCode::DeviceBusy, "装置升级空间不足")
            }
            UpgradePreflightFailure::DpkgBusy => (ResultCode::DeviceBusy, "装置正忙，请稍后重试"),
            UpgradePreflightFailure::ServiceUnavailable => {
                (ResultCode::DeviceBusy, "装置当前不可升级")
            }
            UpgradePreflightFailure::DpkgDamaged
            | UpgradePreflightFailure::ClamAvUnavailable
            | UpgradePreflightFailure::RollbackUnavailable
            | UpgradePreflightFailure::PlatformIncompatible
            | UpgradePreflightFailure::ProbeFailed(_) => {
                (ResultCode::InternalError, "装置升级环境不可用")
            }
        },
        UpgradeError::Io(_) | UpgradeError::State(_) => {
            (ResultCode::InternalError, "系统升级受理失败")
        }
    };
    UpgradeProtocolError {
        result_code,
        message,
    }
}
