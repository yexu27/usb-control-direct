//! S02 鼠标 evdev 转发器。
//!
//! 使用 evdev crate 打开 Linux mouse input 设备，将相对位移和按键事件
//! 转为 HID mouse report 写入 /dev/hidgX。无验证步骤，插入即转发。

use std::fs::OpenOptions;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use device_runtime::{DeviceRuntimeRegistry, DeviceRuntimeUpdate};
use evdev::{Device, InputEventKind, Key, RelativeAxisType};
use tokio::sync::watch;
use tracing::{info, trace, warn};

use crate::error::HidAccessError;
use crate::evdev_poll::{poll_fd_readable, PollResult};
use crate::hid_report::{clamp_i8, MouseReport};

const EVDEV_POLL_TIMEOUT: Duration = Duration::from_millis(100);

/// 鼠标 evdev 转发器。
pub struct MouseForwarder {
    hidg_device: std::path::PathBuf,
    runtime: Option<(Arc<DeviceRuntimeRegistry>, String)>,
}

impl MouseForwarder {
    /// 创建鼠标转发器。
    pub fn new(hidg_device: std::path::PathBuf) -> Self {
        Self {
            hidg_device,
            runtime: None,
        }
    }

    /// 注入运行态上下文。
    ///
    /// 本模块只上报鼠标发布阶段，不执行准入决策。
    pub fn with_runtime(
        mut self,
        registry: Arc<DeviceRuntimeRegistry>,
        runtime_id: String,
    ) -> Self {
        self.runtime = Some((registry, runtime_id));
        self
    }

    /// 在 spawn_blocking 中运行鼠标转发。
    pub fn run(&mut self, input_dev_path: &Path) -> Result<(), HidAccessError> {
        self.run_internal(input_dev_path, None)
    }

    /// 在可取消上下文中运行鼠标转发。
    pub fn run_with_cancel(
        &mut self,
        input_dev_path: &Path,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<(), HidAccessError> {
        self.run_internal(input_dev_path, Some(cancel_rx))
    }

    fn run_internal(
        &mut self,
        input_dev_path: &Path,
        cancel_rx: Option<watch::Receiver<bool>>,
    ) -> Result<(), HidAccessError> {
        let mut dev = Device::open(input_dev_path).map_err(|e| {
            HidAccessError::Internal(format!(
                "打开鼠标 input 设备 {} 失败: {}",
                input_dev_path.display(),
                e
            ))
        })?;

        info!(dev = %input_dev_path.display(), "鼠标映射成功，开始转发");
        self.update_runtime("mapped", "mouse_publish", "", "");

        let mut buttons: u8 = 0;
        let cancel_rx = cancel_rx.as_ref();

        loop {
            if is_cancelled(cancel_rx) {
                info!(dev = %input_dev_path.display(), "鼠标转发器收到取消信号");
                return Ok(());
            }
            if !wait_evdev_readable(&dev, cancel_rx, input_dev_path)? {
                continue;
            }

            let mut dx: i32 = 0;
            let mut dy: i32 = 0;
            let mut wheel: i32 = 0;
            let mut changed = false;

            for ev in dev
                .fetch_events()
                .map_err(|e| HidAccessError::Internal(format!("鼠标读取 evdev 事件失败: {}", e)))?
            {
                match ev.kind() {
                    InputEventKind::Key(Key::BTN_LEFT) => {
                        buttons = update_button(buttons, 0, ev.value());
                        changed = true;
                    }
                    InputEventKind::Key(Key::BTN_RIGHT) => {
                        buttons = update_button(buttons, 1, ev.value());
                        changed = true;
                    }
                    InputEventKind::Key(Key::BTN_MIDDLE) => {
                        buttons = update_button(buttons, 2, ev.value());
                        changed = true;
                    }
                    InputEventKind::RelAxis(RelativeAxisType::REL_X) => {
                        dx += ev.value();
                        changed = true;
                    }
                    InputEventKind::RelAxis(RelativeAxisType::REL_Y) => {
                        dy += ev.value();
                        changed = true;
                    }
                    InputEventKind::RelAxis(RelativeAxisType::REL_WHEEL) => {
                        wheel += ev.value();
                        changed = true;
                    }
                    _ => {}
                }
            }

            if changed {
                let report = MouseReport {
                    buttons,
                    dx: clamp_i8(dx),
                    dy: clamp_i8(dy),
                    wheel: clamp_i8(wheel),
                };

                trace!(report = ?report, "写鼠标 HID report");

                if let Err(e) = write_mouse_report(&self.hidg_device, &report) {
                    warn!(dev = %input_dev_path.display(), ?e, "写鼠标 report 失败，结束转发");
                    return Ok(());
                }
            }
        }
    }

    fn update_runtime(&self, status: &str, stage: &str, fail_code: &str, fail_reason: &str) {
        if let Some((registry, runtime_id)) = self.runtime.as_ref() {
            registry.update(
                runtime_id,
                DeviceRuntimeUpdate {
                    status: status.to_string(),
                    stage: stage.to_string(),
                    fail_code: fail_code.to_string(),
                    fail_reason: fail_reason.to_string(),
                },
            );
        }
    }
}

fn is_cancelled(cancel_rx: Option<&watch::Receiver<bool>>) -> bool {
    cancel_rx.map(|rx| *rx.borrow()).unwrap_or(false)
}

fn wait_evdev_readable(
    dev: &Device,
    cancel_rx: Option<&watch::Receiver<bool>>,
    input_dev_path: &Path,
) -> Result<bool, HidAccessError> {
    if is_cancelled(cancel_rx) {
        return Ok(false);
    }
    match poll_fd_readable(dev.as_raw_fd(), EVDEV_POLL_TIMEOUT) {
        PollResult::Readable => Ok(true),
        PollResult::Timeout | PollResult::Interrupted => Ok(false),
        PollResult::Error(e) => Err(HidAccessError::Internal(format!(
            "等待鼠标 evdev {} 可读失败: {}",
            input_dev_path.display(),
            e
        ))),
    }
}

/// 更新按钮状态位。
fn update_button(buttons: u8, bit: u8, value: i32) -> u8 {
    if value == 0 {
        buttons & !(1 << bit)
    } else {
        buttons | (1 << bit)
    }
}

/// 写鼠标 HID report 到 hidg 设备节点。
fn write_mouse_report(path: &Path, report: &MouseReport) -> Result<(), HidAccessError> {
    let mut file = OpenOptions::new().write(true).open(path).map_err(|e| {
        HidAccessError::Internal(format!("打开 hidg {} 失败: {}", path.display(), e))
    })?;
    file.write_all(&report.to_bytes())
        .map_err(|e| HidAccessError::Internal(format!("写 hidg report 失败: {}", e)))
}
