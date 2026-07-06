//! S04 文件访问控制引擎。
//!
//! 将真实 U 盘内容通过 NBD + 虚拟 exFAT + OTG gadget 以受控方式映射给受控主机，
//! 按 5 级优先级策略决定每个文件的可见性和可访问性。

pub mod error;
pub mod block_backend;
pub mod types;
pub mod vfs;
pub mod exec_detect;
pub mod autorun;
pub mod file_tree;
pub mod policy;
pub mod exfat;
pub mod nbd;
pub mod raw_mount;
pub mod gadget;
pub mod gadget_bootstrap;
pub mod media_builder;
pub mod publisher;
pub mod startup_recovery;
pub mod storage_session;

pub use error::FileAccessError;
pub use storage_session::StorageSessionManager;
