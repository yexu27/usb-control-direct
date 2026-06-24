//! S02 键盘 evdev 拦截器。
//!
//! 使用 evdev crate 打开 Linux input 设备，驱动已有的 KeyboardChallenge 状态机
//! 完成 1234 序列验证，验证通过后将按键转为 HID report 写入 /dev/hidgX。
//!
//! 本模块在 tokio::task::spawn_blocking 中运行（evdev 为阻塞 IO）。
//! 审计日志由上层（Orchestrator）负责，本模块只处理 evdev ↔ HID 转换。

use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use evdev::{Device, InputEventKind};
use tracing::{error, info, warn};

use crate::error::HidAccessError;
use crate::hid_report::{keycode_to_hid, KeyboardReport};
use crate::keyboard::{KeyboardChallenge, KeyboardEvent, KeyboardState, KeyboardTransitionResult};

/// 键盘 evdev 拦截器。
///
/// 打开 Linux input 设备后自动 grab（独占），受控主机不再收到该键盘的输入。
pub struct KeyboardInterceptor {
    challenge: KeyboardChallenge,
    hidg_device: std::path::PathBuf,
}

impl KeyboardInterceptor {
    /// 创建键盘拦截器。
    pub fn new(hidg_device: std::path::PathBuf) -> Self {
        Self {
            challenge: KeyboardChallenge::new(),
            hidg_device,
        }
    }

    /// 在 spawn_blocking 中运行键盘拦截与验证。
    ///
    /// 返回 Ok 表示键盘正常拔出或映射结束，Err 表示不可恢复的错误。
    pub fn run(&mut self, input_dev_path: &Path) -> Result<(), HidAccessError> {
        let mut dev = Device::open(input_dev_path).map_err(|e| {
            error!(dev = %input_dev_path.display(), reason = %e, "evdev 设备打开失败");
            HidAccessError::Internal(format!(
                "打开 input 设备 {} 失败: {}",
                input_dev_path.display(),
                e
            ))
        })?;

        // evdev::Device::open 成功后，设备已被 grab，受控主机不再收到事件。
        // 通知状态机 HID 接管成功。
        match self.challenge.transition(KeyboardEvent::GrabSuccess) {
            Ok(KeyboardTransitionResult::Transitioned(KeyboardState::KbWaiting)) => {
                info!(dev = %input_dev_path.display(), "键盘 HID 接管成功，等待 1234 验证");
            }
            _ => {
                return Err(HidAccessError::Internal(
                    "键盘状态机拒绝对 GrabSuccess 事件".into(),
                ));
            }
        }

        // === 阶段 1: 等待 1234 验证序列 ===
        info!(dev = %input_dev_path.display(), "请在该键盘上顺序输入 1 2 3 4");
        'auth: loop {
            for ev in dev.fetch_events().map_err(|e| {
                HidAccessError::Internal(format!("读取 evdev 事件失败: {}", e))
            })? {
                let event = match Self::parse_evdev_event(ev) {
                    Some(e) => e,
                    None => continue,
                };

                match self.challenge.transition(event).map_err(|e| {
                    HidAccessError::Internal(format!("键盘状态机转换失败: {}", e))
                })? {
                    KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped) => {
                        info!(dev = %input_dev_path.display(), hidg = %self.hidg_device.display(),
                              "键盘 1234 验证通过，开始转发");
                        break 'auth; // 进入转发阶段
                    }
                    KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved) => {
                        info!(dev = %input_dev_path.display(), "键盘在验证阶段被拔出");
                        return Ok(());
                    }
                    KeyboardTransitionResult::Transitioned(state) => {
                        return Err(HidAccessError::Internal(format!(
                            "未预期的键盘状态: {:?}",
                            state
                        )));
                    }
                    KeyboardTransitionResult::Unchanged => {
                        // 继续等待（输入中途或错误后已自动重置）
                    }
                }
            }
        }

        // === 阶段 2: 按键转发 ===
        self.forward_loop(&mut dev, input_dev_path)
    }

    /// 将 evdev 事件解析为 KeyboardEvent。
    ///
    /// 仅处理按键按下事件（value == 1），释放事件和重复事件忽略。
    /// 修饰键 → KeyboardEvent::ModifierKey，普通键 → KeyboardEvent::KeyPress(hid_usage)。
    /// 未映射的键返回 None。
    fn parse_evdev_event(ev: evdev::InputEvent) -> Option<KeyboardEvent> {
        if let InputEventKind::Key(key) = ev.kind() {
            if ev.value() != 1 {
                return None;
            }
            match keycode_to_hid(key) {
                Some((modifier, 0)) if modifier != 0 => Some(KeyboardEvent::ModifierKey),
                Some((0, usage)) if usage != 0 => Some(KeyboardEvent::KeyPress(usage)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// 按键转发循环：evdev 事件 → HID report → /dev/hidgX。
    fn forward_loop(
        &mut self,
        dev: &mut Device,
        input_dev_path: &Path,
    ) -> Result<(), HidAccessError> {
        let mut pressed: BTreeSet<u8> = BTreeSet::new();
        let mut modifiers: u8 = 0;

        loop {
            for ev in dev.fetch_events().map_err(|e| {
                HidAccessError::Internal(format!("转发阶段读取 evdev 事件失败: {}", e))
            })? {
                if let InputEventKind::Key(key) = ev.kind() {
                    match keycode_to_hid(key) {
                        Some((mod_bit, 0)) if mod_bit != 0 => match ev.value() {
                            1 => modifiers |= mod_bit,
                            0 => modifiers &= !mod_bit,
                            _ => {}
                        },
                        Some((0, usage)) if usage != 0 => match ev.value() {
                            1 => {
                                pressed.insert(usage);
                            }
                            0 => {
                                pressed.remove(&usage);
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                // 构建并写入 HID report
                let mut report = KeyboardReport::empty();
                report.modifier = modifiers;
                for (i, key) in pressed.iter().take(6).enumerate() {
                    report.keys[i] = *key;
                }

                let mut file = OpenOptions::new()
                    .write(true)
                    .open(&self.hidg_device)
                    .map_err(|e| {
                        HidAccessError::Internal(format!(
                            "打开 hidg 设备 {} 失败: {}",
                            self.hidg_device.display(),
                            e
                        ))
                    })?;

                if let Err(e) = file.write_all(&report.to_bytes()) {
                    warn!(dev = %input_dev_path.display(), ?e, "写 hidg report 失败，结束转发");
                    return Ok(());
                }
            }
        }
    }
}
