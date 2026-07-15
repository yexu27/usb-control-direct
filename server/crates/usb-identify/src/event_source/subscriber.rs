use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::event_source::{
    device_event_from_info, parse_device_info_from_syspath, should_forward_usb_event,
};
use crate::orchestrator::DeviceEvent;

const UDEV_POLL_TIMEOUT: Duration = Duration::from_millis(200);

/// udev 订阅停止令牌。
#[derive(Clone, Debug)]
pub struct SubscriberStopToken {
    stopped: Arc<AtomicBool>,
}

impl SubscriberStopToken {
    /// 创建未停止状态的令牌。
    pub fn new() -> Self {
        Self {
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 请求订阅循环停止。
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    /// 判断是否已经收到停止请求。
    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    /// 返回内部原子标记，供测试验证共享状态。
    pub fn inner(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stopped)
    }
}

impl Default for SubscriberStopToken {
    fn default() -> Self {
        Self::new()
    }
}

/// 运行期 USB udev 订阅器。
///
/// 该订阅器只负责监听 udev usb_interface add/remove 事件，并转换成
/// `DeviceEvent`。业务准入由 `DeviceOrchestrator` 处理。
pub struct UdevSubscriber {
    stop_token: SubscriberStopToken,
}

impl UdevSubscriber {
    /// 创建 udev 订阅器。
    pub fn new(stop_token: SubscriberStopToken) -> Self {
        Self { stop_token }
    }

    /// 运行可停止的 udev monitor 订阅循环。
    pub fn run(self, tx: mpsc::UnboundedSender<DeviceEvent>) {
        let socket = match udev::MonitorBuilder::new()
            .and_then(|builder| builder.match_subsystem("usb"))
            .and_then(|builder| builder.listen())
        {
            Ok(socket) => socket,
            Err(e) => {
                error!("udev 监听器创建失败: {}", e);
                return;
            }
        };

        info!("udev USB 设备监听已启动");
        while !self.stop_token.is_stopped() {
            match poll_udev_fd(socket.as_raw_fd(), UDEV_POLL_TIMEOUT) {
                PollResult::Readable => drain_events(&socket, &tx),
                PollResult::Timeout | PollResult::Interrupted => {}
                PollResult::Error(e) => {
                    warn!(error = %e, "udev poll 失败");
                    break;
                }
            }
        }
        info!("udev USB 设备监听已停止");
    }
}

enum PollResult {
    Readable,
    Timeout,
    Interrupted,
    Error(std::io::Error),
}

fn poll_udev_fd(fd: i32, timeout: Duration) -> PollResult {
    let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    let mut poll_fd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };

    // 安全性: poll_fd 指向当前栈上的有效 pollfd；nfds=1；timeout 为有限毫秒。
    let result = unsafe { libc::poll(&mut poll_fd, 1, timeout_ms) };
    if result == 0 {
        return PollResult::Timeout;
    }
    if result < 0 {
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::Interrupted {
            return PollResult::Interrupted;
        }
        return PollResult::Error(error);
    }
    if poll_fd.revents & libc::POLLIN != 0 {
        PollResult::Readable
    } else {
        PollResult::Timeout
    }
}

fn drain_events(socket: &udev::MonitorSocket, tx: &mpsc::UnboundedSender<DeviceEvent>) {
    for event in socket.iter() {
        let action = match event.action() {
            Some(action) => action.to_string_lossy().to_string(),
            None => continue,
        };
        let sys_path = event.syspath().to_string_lossy().to_string();
        let devtype = event
            .property_value("DEVTYPE")
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();

        if !should_forward_usb_event(&action, &devtype) {
            continue;
        }

        match action.as_str() {
            "add" => {
                info!(sys_path = %sys_path, devtype = %devtype, "USB 设备插入事件");
                if let Some(info) = parse_device_info_from_syspath(event.syspath()) {
                    let _ = tx.send(device_event_from_info(info));
                }
            }
            "remove" => {
                info!(sys_path = %sys_path, devtype = %devtype, "USB 设备拔出事件");
                let _ = tx.send(DeviceEvent::DeviceRemoved(sys_path));
            }
            _ => {}
        }
    }
}
