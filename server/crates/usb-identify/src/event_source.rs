//! USB 事件入口层。
//!
//! 本模块负责启动时枚举和运行期 udev 订阅，将系统 USB 状态转换成
//! `DeviceEvent` 发送给 `DeviceOrchestrator`。本模块不做白名单、准入、
//! 存储资源或下游业务处理。

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, warn};

mod convert;
mod enumerator;
mod subscriber;

use crate::orchestrator::DeviceEvent;

pub(crate) use convert::parse_device_info_from_syspath;
pub use convert::{device_event_from_info, is_usb_interface_devtype, should_forward_usb_event};
pub use enumerator::UsbEnumerator;
pub use subscriber::{SubscriberStopToken, UdevSubscriber};

/// USB 事件源 owner。
///
/// 负责启动时枚举和运行期 udev 订阅，并在服务停止时关闭订阅任务。
pub struct UsbEventSource {
    stop_token: SubscriberStopToken,
    subscriber_handle: Option<JoinHandle<()>>,
    enumerator_handle: Option<JoinHandle<()>>,
}

impl UsbEventSource {
    /// 创建未启动的 USB 事件源。
    pub fn new() -> Self {
        Self {
            stop_token: SubscriberStopToken::new(),
            subscriber_handle: None,
            enumerator_handle: None,
        }
    }

    /// 启动存量枚举和运行期 udev 订阅。
    pub fn start(&mut self, tx: mpsc::UnboundedSender<DeviceEvent>) {
        let enum_tx = tx.clone();
        self.enumerator_handle = Some(tokio::task::spawn_blocking(move || {
            UsbEnumerator::enumerate_and_send(&enum_tx);
        }));

        let stop_token = self.stop_token.clone();
        self.subscriber_handle = Some(tokio::task::spawn_blocking(move || {
            UdevSubscriber::new(stop_token).run(tx);
        }));
    }

    /// 请求事件源停止。
    pub fn stop(&self) {
        self.stop_token.stop();
    }

    /// 判断事件源是否已经收到停止请求。
    pub fn is_stopped(&self) -> bool {
        self.stop_token.is_stopped()
    }

    /// 等待事件源后台任务退出。
    pub async fn join(mut self) {
        if let Some(handle) = self.enumerator_handle.take() {
            if let Err(e) = handle.await {
                warn!(error = %e, "USB 存量枚举任务异常退出");
            }
        }
        if let Some(handle) = self.subscriber_handle.take() {
            match tokio::time::timeout(std::time::Duration::from_secs(3), handle).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => error!(error = %e, "udev 订阅任务异常退出"),
                Err(_) => warn!("等待 udev 订阅任务退出超时"),
            }
        }
    }
}

impl Default for UsbEventSource {
    fn default() -> Self {
        Self::new()
    }
}
