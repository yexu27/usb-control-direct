//! S02 键鼠准入错误类型。

/// S02 键鼠准入控制错误。
#[derive(Debug, thiserror::Error)]
pub enum HidAccessError {
    /// HID 设备接管失败。
    #[error("HID 接管失败: {0}")]
    GrabFailed(String),

    /// 设备已物理拔出，操作无效。
    #[error("设备已移除")]
    DeviceRemoved,

    /// 状态机收到当前状态下不允许的事件。
    #[error("状态转换非法: {from} -> {event}")]
    InvalidTransition { from: String, event: String },

    /// 内部逻辑错误。
    #[error("内部错误: {0}")]
    Internal(String),
}
