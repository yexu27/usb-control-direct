//! S01 USB 设备识别与映射引擎。
//!
//! 装置端 USB 设备的全生命周期编排器：
//! - udev 监听 USB 热插拔
//! - 描述符解析与设备分类
//! - U 盘 mount/umount
//! - U 盘状态机管理
//! - 协调 S03 扫描和 S04 映射
//! - 拔出清理

pub mod error;
pub mod traits;
pub mod descriptor;
pub mod state_machine;
pub mod mount;
pub mod monitor;

pub use error::UsbIdentifyError;
pub use traits::{DeviceMapper, MapContext, MappedSession, Scanner, ScanResult};
