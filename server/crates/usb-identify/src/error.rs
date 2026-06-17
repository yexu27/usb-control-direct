//! S01 USB 识别错误类型。

/// S01 USB 设备识别操作错误。
#[derive(Debug, thiserror::Error)]
pub enum UsbIdentifyError {
    #[error("udev 错误: {0}")]
    Udev(String),

    #[error("挂载失败: {0}")]
    MountFailed(String),

    #[error("卸载失败: {0}")]
    UmountFailed(String),

    #[error("描述符解析失败: {0}")]
    DescriptorParseFailed(String),

    #[error("状态转换非法: {from:?} -> {event}")]
    InvalidTransition { from: String, event: String },

    #[error("扫描错误: {0}")]
    ScanError(String),

    #[error("映射错误: {0}")]
    MapError(String),

    #[error("存储层错误: {0}")]
    Storage(#[from] storage::error::StorageError),

    #[error("内部错误: {0}")]
    Internal(String),
}
