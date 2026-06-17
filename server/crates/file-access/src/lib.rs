//! S04 文件访问控制引擎。
//!
//! 将真实 U 盘内容通过 NBD + 虚拟 exFAT + OTG gadget 以受控方式映射给受控主机，
//! 按 5 级优先级策略决定每个文件的可见性和可访问性。

pub mod error;
pub mod types;
pub mod exec_detect;
pub mod autorun;
pub mod file_tree;
pub mod policy;
pub mod exfat;
pub mod write_back;
pub mod nbd;
pub mod gadget;
pub mod engine;

pub use engine::FileAccessEngine;
pub use error::FileAccessError;
