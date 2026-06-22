//! S02 鼠标 evdev 转发器。
//!
//! 使用 evdev crate 打开 Linux mouse input 设备，将相对位移和按键事件
//! 转为 HID mouse report 写入 /dev/hidgX。无验证步骤，插入即转发。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use evdev::{Device, InputEventKind, Key, RelativeAxisType};
use tracing::{debug, info, warn};

use crate::error::HidAccessError;
use crate::hid_report::{clamp_i8, MouseReport};

/// 鼠标 evdev 转发器。
pub struct MouseForwarder {
    hidg_device: std::path::PathBuf,
}

impl MouseForwarder {
    /// 创建鼠标转发器。
    pub fn new(hidg_device: std::path::PathBuf) -> Self {
        Self { hidg_device }
    }

    /// 在 spawn_blocking 中运行鼠标转发。
    pub fn run(&mut self, input_dev_path: &Path) -> Result<(), HidAccessError> {
        let mut dev = Device::open(input_dev_path).map_err(|e| {
            HidAccessError::Internal(format!(
                "打开鼠标 input 设备 {} 失败: {}",
                input_dev_path.display(),
                e
            ))
        })?;

        info!(dev = %input_dev_path.display(), "鼠标映射成功，开始转发");

        let mut buttons: u8 = 0;

        loop {
            let mut dx: i32 = 0;
            let mut dy: i32 = 0;
            let mut wheel: i32 = 0;
            let mut changed = false;

            for ev in dev.fetch_events().map_err(|e| {
                HidAccessError::Internal(format!("鼠标读取 evdev 事件失败: {}", e))
            })? {
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

                debug!(?report, "写鼠标 HID report");

                if let Err(e) = write_mouse_report(&self.hidg_device, &report) {
                    warn!(dev = %input_dev_path.display(), ?e, "写鼠标 report 失败，结束转发");
                    return Ok(());
                }
            }
        }
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
    file.write_all(&report.to_bytes()).map_err(|e| {
        HidAccessError::Internal(format!("写 hidg report 失败: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_button_bits() {
        assert_eq!(update_button(0, 0, 1), 0x01);
        assert_eq!(update_button(0x01, 1, 1), 0x03);
        assert_eq!(update_button(0x03, 0, 0), 0x02);
        assert_eq!(update_button(0x02, 2, 1), 0x06);
    }
}
