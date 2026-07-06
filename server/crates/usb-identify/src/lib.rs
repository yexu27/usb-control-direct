//! S01 USB 设备识别与准入路由。
//!
//! 本 crate 负责:
//! - udev USB 热插拔事件归并
//! - 描述符解析与设备分类
//! - 白名单和权限判定
//! - 将授权 storage 设备路由到 StorageSessionController
//! - 键盘和鼠标 HID 受控链路编排
//!
//! 本 crate 不负责存储运行资源、病毒扫描实现、虚拟介质发布、
//! 启动恢复或 storage 资源清理。

pub mod descriptor;
pub mod error;
pub mod event_source;
pub mod monitor;
pub mod orchestrator;
pub mod traits;

pub use error::UsbIdentifyError;
pub use traits::{
    AuthorizedStorageDevice, ScanResult, Scanner, StorageSessionController, StorageSessionError,
    StorageSessionHandle,
};
