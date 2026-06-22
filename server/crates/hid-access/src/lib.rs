//! S02 键鼠准入控制引擎。
//!
//! S02 由 S01 调用：S01 udev 检测到设备 → 读描述符分类为键盘/鼠标 → 调用 S02 执行准入。
//! S02 不自己监听 udev，输入来自 S01 的设备类型识别结果和 HID 事件流。

pub mod error;
pub mod evdev_interceptor;
pub mod hid_gadget;
pub mod hid_report;
pub mod keyboard;
pub mod mouse;

pub use error::HidAccessError;
pub use keyboard::{KeyboardChallenge, KeyboardEvent, KeyboardTransitionResult};
pub use mouse::{MouseAdmission, MouseEvent, MouseTransitionResult};
