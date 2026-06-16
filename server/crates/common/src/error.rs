//! 装置端公共错误类型。
//!
//! `CommonError` 用作各业务 crate 错误的通用上游；业务错误若可归类到这里，
//! 应直接使用本枚举或通过 `From` 转换上抛。

use thiserror::Error;

/// 装置端公共错误。
#[derive(Debug, Error)]
pub enum CommonError {
    /// 协议帧头校验失败（魔数错误、长度越界、CRC32 不匹配等）。
    #[error("协议帧头校验失败: {0}")]
    InvalidFrame(String),

    /// 字段映射失败（如不支持的角色字符串、非法时间戳格式）。
    #[error("字段映射失败: {0}")]
    MappingError(String),

    /// 协议字段值不在已知枚举范围内。
    #[error("未知枚举值: {0}")]
    UnknownEnum(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_error_display_includes_message() {
        let err = CommonError::InvalidFrame("magic mismatch".to_string());
        assert_eq!(format!("{}", err), "协议帧头校验失败: magic mismatch");
    }

    #[test]
    fn common_error_mapping_error_display() {
        let err = CommonError::MappingError("invalid role".to_string());
        assert_eq!(format!("{}", err), "字段映射失败: invalid role");
    }

    #[test]
    fn common_error_unknown_enum_display() {
        let err = CommonError::UnknownEnum("x".to_string());
        assert_eq!(format!("{}", err), "未知枚举值: x");
    }
}
