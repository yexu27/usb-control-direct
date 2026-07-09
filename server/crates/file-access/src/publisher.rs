//! Storage session 发布器。
//!
//! 本模块负责把已构建的虚拟介质发布成 `/dev/nbdX` 并暴露到业务
//! mass storage LUN。它不构建文件树、不加载策略、不扫描病毒。

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::block_backend::BlockBackend;
use crate::exfat::fs::VirtualExfatFs;
use crate::gadget::{GadgetError, GadgetRuntime};
use crate::nbd::{device::NbdDevice, NbdDeviceManager};

const UDC_ENUMERATION_TIMEOUT: Duration = Duration::from_secs(3);
const UDC_ENUMERATION_POLL_INTERVAL: Duration = Duration::from_millis(100);

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
    nbd_device_runtime: NbdDevice,
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
            this.nbd_device_runtime.stop().await;
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

fn udc_state_needs_rebind(state: Option<&str>) -> bool {
    matches!(
        state,
        None | Some("not attached") | Some("powered") | Some("default")
    )
}

fn udc_state_is_configured(state: Option<&str>) -> bool {
    matches!(state, Some("configured"))
}

async fn ensure_host_enumerated_after_lun_update(
    gadget: &GadgetRuntime,
) -> Result<(), GadgetError> {
    let before_state = gadget.current_udc_state()?;
    if udc_state_is_configured(before_state.as_deref()) {
        debug!(
            state = ?before_state,
            "UDC already configured after mass storage LUN update"
        );
        return Ok(());
    }

    let before_udc = gadget.current_udc_name()?;
    if udc_state_needs_rebind(before_state.as_deref()) {
        info!(
            udc = ?before_udc,
            state = ?before_state,
            "UDC is not configured after LUN update; rebind once to trigger host enumeration"
        );
        gadget.rebind_current_udc()?;
    } else {
        warn!(
            udc = ?before_udc,
            state = ?before_state,
            "UDC is not configured after LUN update; waiting without rebind"
        );
    }

    let deadline = Instant::now() + UDC_ENUMERATION_TIMEOUT;
    loop {
        let after_state = gadget.current_udc_state()?;
        if udc_state_is_configured(after_state.as_deref()) {
            info!(
                udc = ?gadget.current_udc_name()?,
                before_state = ?before_state,
                after_state = ?after_state,
                "UDC host enumeration completed after mass storage LUN update"
            );
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(GadgetError::UdcEnumerationFailed {
                udc: gadget.current_udc_name()?,
                before: before_state,
                after: after_state,
            });
        }

        sleep(UDC_ENUMERATION_POLL_INTERVAL).await;
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
            let backend: Arc<dyn BlockBackend> = fs.clone();

            debug!(nbd = %nbd_device.display(), total_sectors, "启动 NBD 服务");
            let mut nbd_device_runtime = NbdDeviceManager::default()
                .start(nbd_index, total_sectors, readonly, backend)
                .await
                .map_err(|e| PublishError::Nbd(e.to_string()))?;

            if let Err(e) = self.gadget.attach_mass_storage(&nbd_device, readonly) {
                nbd_device_runtime.stop().await;
                return Err(PublishError::Gadget(e));
            }

            if let Err(e) = ensure_host_enumerated_after_lun_update(&self.gadget).await {
                let _ = self.gadget.detach_mass_storage();
                nbd_device_runtime.stop().await;
                return Err(PublishError::Gadget(e));
            }

            info!(nbd = %nbd_device.display(), readonly, "storage 已发布到 gadget");
            let published: Box<dyn PublishedStorageRuntime> = Box::new(PublishedStorage {
                nbd_device_runtime,
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
