//! Storage session 发布器。
//!
//! 本模块负责把已构建的虚拟介质发布成 `/dev/nbdX` 并暴露到业务
//! mass storage LUN。它不构建文件树、不加载策略、不扫描病毒。

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, error, info};

use crate::exfat::fs::VirtualExfatFs;
use crate::gadget::{GadgetError, GadgetRuntime};
use crate::nbd::{ensure_partition_scan_disabled, run_request_loop, NbdServer};

/// 已发布 storage 资源的运行时句柄。
pub(crate) trait PublishedStorageRuntime: Send {
    /// 清理发布资源。
    fn stop(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

/// Storage 发布能力。
pub(crate) trait StorageRuntimePublisher: Send + Sync {
    /// 发布虚拟介质并绑定 gadget LUN。
    fn publish(
        &self,
        fs: VirtualExfatFs,
        nbd_index: u32,
        readonly: bool,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Box<dyn PublishedStorageRuntime>, PublishError>> + Send + '_,
        >,
    >;
}

/// 一次已发布的 storage runtime 资源。
pub struct PublishedStorage {
    nbd_server: NbdServer,
    gadget: GadgetRuntime,
    fs: Arc<VirtualExfatFs>,
    nbd_device: PathBuf,
}

impl PublishedStorageRuntime for PublishedStorage {
    fn stop(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            let mut this = *self;
            if let Err(e) = this.gadget.detach_mass_storage() {
                error!(error = %e, "清理 mass storage LUN 失败");
            }
            if let Err(e) = this.fs.shutdown() {
                error!(error = %e, "虚拟介质 flush/shutdown 失败");
            }
            this.nbd_server.stop_async().await;
            info!(nbd = %this.nbd_device.display(), "storage 发布资源已清理");
        })
    }
}

/// Storage publisher。
#[derive(Clone)]
pub struct StoragePublisher {
    gadget: GadgetRuntime,
}

impl StoragePublisher {
    /// 创建 publisher。
    pub fn new(gadget: GadgetRuntime) -> Self {
        Self { gadget }
    }
}

impl StorageRuntimePublisher for StoragePublisher {
    fn publish(
        &self,
        fs: VirtualExfatFs,
        nbd_index: u32,
        readonly: bool,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Box<dyn PublishedStorageRuntime>, PublishError>> + Send + '_,
        >,
    > {
        Box::pin(async move {
            let nbd_device = nbd_device_path(nbd_index);
            let total_sectors = fs.total_sectors();
            let fs = Arc::new(fs);
            let mut nbd_server = NbdServer::new(&nbd_device);

            ensure_partition_scan_disabled()
                .map_err(|e| PublishError::Nbd(format!("NBD 本机分区扫描未关闭，拒绝发布: {e}")))?;

            debug!(nbd = %nbd_device.display(), total_sectors, "启动 NBD 服务");
            let user_fd = nbd_server
                .start(total_sectors, readonly)
                .map_err(|e| PublishError::Nbd(e.to_string()))?;

            let fs_for_loop = Arc::clone(&fs);
            let request_loop_handle = tokio::task::spawn_blocking(move || {
                run_request_loop(user_fd, fs_for_loop);
            });
            nbd_server.set_request_loop_handle(request_loop_handle);

            if let Err(e) = nbd_server.wait_ready(total_sectors, Duration::from_millis(500)) {
                nbd_server.stop_async().await;
                return Err(PublishError::Nbd(format!("NBD backing 未就绪: {e}")));
            }

            if let Err(e) = self.gadget.attach_mass_storage(&nbd_device, readonly) {
                nbd_server.stop_async().await;
                return Err(PublishError::Gadget(e));
            }

            info!(nbd = %nbd_device.display(), readonly, "storage 已发布到 gadget");
            let published: Box<dyn PublishedStorageRuntime> = Box::new(PublishedStorage {
                nbd_server,
                gadget: self.gadget.clone(),
                fs,
                nbd_device,
            });
            Ok(published)
        })
    }
}

/// 构造整盘 NBD 设备路径。
pub fn nbd_device_path(nbd_index: u32) -> PathBuf {
    PathBuf::from(format!("/dev/nbd{nbd_index}"))
}

/// 发布错误。
#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("NBD 发布失败: {0}")]
    Nbd(String),
    #[error("gadget 绑定失败: {0}")]
    Gadget(#[from] GadgetError),
}
