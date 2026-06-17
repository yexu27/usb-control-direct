//! S01 下游模块 trait 定义。
//!
//! Scanner (S03, P04 实现) 和 DeviceMapper (S04, P05 实现) 的接口契约。
//! 本阶段仅定义 trait，mock 实现在 `#[cfg(test)]` 中。

use std::path::Path;

/// 病毒扫描结果。
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// 是否干净（无病毒）。
    pub is_clean: bool,
    /// 病毒文件路径列表（相对于 mount_path）。
    pub infected_files: Vec<String>,
}

/// 病毒扫描错误。
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("扫描失败: {0}")]
    Failed(String),
    #[error("扫描被取消")]
    Cancelled,
    #[error("clamd 不可用")]
    ServiceUnavailable,
}

/// S03 病毒扫描（P04 实现）。
///
/// S01 在 U 盘状态 SCANNING 时调用 scan，拔出时调用 cancel。
pub trait Scanner: Send + Sync {
    /// 扫描指定挂载路径下的文件。
    fn scan(
        &self,
        mount_path: &Path,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ScanResult, ScanError>> + Send + '_>,
    >;

    /// 取消正在进行的扫描任务。
    fn cancel(&self, mount_path: &Path);
}

/// 映射上下文。
#[derive(Debug, Clone)]
pub struct MapContext {
    /// U 盘挂载路径。
    pub mount_path: String,
    /// 扫描结果（含病毒文件列表）。
    pub scan_result: ScanResult,
    /// 权限：0=只读 / 1=读写。
    pub permission: i32,
}

/// 映射会话句柄。
#[derive(Debug)]
pub struct MappedSession {
    /// 标识（用于清理时定位）。
    pub id: String,
    /// 挂载路径。
    pub mount_path: String,
}

/// 映射错误。
#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error("NBD 启动失败: {0}")]
    NbdFailed(String),
    #[error("OTG Gadget 绑定失败: {0}")]
    GadgetFailed(String),
    #[error("虚拟文件树构建失败: {0}")]
    BuildFailed(String),
}

/// 取消映射错误。
#[derive(Debug, thiserror::Error)]
pub enum UnmapError {
    #[error("取消映射失败: {0}")]
    Failed(String),
}

/// S04 文件访问控制映射：构建虚拟文件树 + NBD + OTG（P05 实现）。
///
/// 注意：mount/umount 由 S01 自己完成，DeviceMapper 只负责虚拟映射。
pub trait DeviceMapper: Send + Sync {
    /// 构建受控文件树 + 启动 NBD + 绑定 OTG Gadget。
    fn map_device(
        &self,
        ctx: MapContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<MappedSession, MapError>> + Send + '_>,
    >;

    /// 解绑 OTG + 停止 NBD + 清理虚拟文件树。
    fn unmap_device(
        &self,
        session: MappedSession,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), UnmapError>> + Send + '_>,
    >;
}
