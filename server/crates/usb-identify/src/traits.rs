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

/// 已授权的大容量存储设备。
///
/// 该结构是 S01 传给 storage session owner 的唯一业务输入。
/// S01 已完成设备类型识别、白名单命中和权限判定。
#[derive(Debug, Clone)]
pub struct AuthorizedStorageDevice {
    pub parent_path: String,
    pub sys_path: String,
    pub dev_path: String,
    pub serial_number: String,
    pub vid: String,
    pub pid: String,
    pub device_name: String,
    pub capacity_bytes: Option<i64>,
    pub permission: i32,
}

/// Storage session 启动后返回给 S01 的只读句柄。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSessionHandle {
    pub session_id: String,
    pub parent_path: String,
}

/// Storage session 生命周期错误。
#[derive(Debug, thiserror::Error)]
pub enum StorageSessionError {
    #[error("storage session rejected: {0}")]
    Rejected(String),
    #[error("storage session failed: {0}")]
    Failed(String),
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
    ///
    /// 参数:
    ///   - mount_path: U 盘挂载路径。
    ///   - device_sn: 设备序列号（用于 T06 日志）。
    ///   - device_name: 设备名称（用于 T06 日志）。
    fn scan(
        &self,
        mount_path: &Path,
        device_sn: &str,
        device_name: &str,
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
    /// 真实源分区大小（字节），用于生成受控主机可见的虚拟 U 盘容量。
    /// Source partition size in bytes, used as the virtual media capacity target.
    pub source_size_bytes: u64,
    /// 已分配的 NBD 设备路径，例如 /dev/nbd3。
    /// Allocated NBD device path, for example /dev/nbd3.
    pub nbd_device: String,
}

/// 映射会话句柄。
#[derive(Debug)]
pub struct MappedSession {
    /// 标识（用于清理时定位）。
    pub id: String,
    /// 挂载路径。
    pub mount_path: String,
    /// 本次映射使用的 NBD 设备路径。
    /// NBD device path used by this mapping.
    pub nbd_device: String,
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), UnmapError>> + Send + '_>>;
}

/// 大容量存储受控会话控制器。
///
/// `DeviceOrchestrator` 只依赖该抽象，不直接 mount、scan、启动 NBD 或操作 gadget。
pub trait StorageSessionController: Send + Sync {
    /// 启动一个已授权 storage 设备的受控会话。
    fn start_authorized_storage(
        &self,
        device: AuthorizedStorageDevice,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<StorageSessionHandle, StorageSessionError>>
                + Send
                + '_,
        >,
    >;

    /// 按 USB parent path 停止会话，用于设备拔出。
    fn stop_by_parent(
        &self,
        parent_path: String,
        reason: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), StorageSessionError>> + Send + '_>,
    >;

    /// 停止所有会话，用于服务停止。
    fn stop_all(
        &self,
        reason: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), StorageSessionError>> + Send + '_>,
    >;

    /// 当前是否存在 active storage session。
    fn has_active_storage(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>>;
}
