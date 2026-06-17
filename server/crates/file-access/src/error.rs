//! S04 错误类型。

use thiserror::Error;

/// 文件访问控制引擎错误。
#[derive(Debug, Error)]
pub enum FileAccessError {
    /// 文件树构建失败。
    #[error("文件树构建失败: {0}")]
    TreeBuildFailed(String),

    /// exFAT 卷生成失败。
    #[error("exFAT 卷生成失败: {0}")]
    VolumeBuildFailed(String),

    /// NBD 启动失败。
    #[error("NBD 启动失败: {0}")]
    NbdFailed(String),

    /// OTG Gadget 绑定失败。
    #[error("OTG Gadget 绑定失败: {0}")]
    GadgetFailed(String),

    /// I/O 错误。
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 写回失败。
    #[error("写回失败: {0}")]
    WriteBackFailed(String),
}
