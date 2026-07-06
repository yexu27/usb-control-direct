//! NBD 块设备发布模块。
//!
//! NBD 只负责 Linux NBD 协议、设备生命周期和块请求转发，不承载策略、病毒、文件树或 gadget 语义。

pub mod device;
pub mod io;
pub mod manager;
pub mod protocol;
pub mod request_loop;
pub mod sysfs;

pub use self::manager::NbdDeviceManager;
pub use self::sysfs::{read_nbd_partition_scan_status, NbdPartitionScanStatus};
