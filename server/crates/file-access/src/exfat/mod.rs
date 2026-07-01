//! 虚拟 exFAT 卷。

pub mod layout;
pub mod boot;
pub mod upcase;
pub mod bitmap;
pub mod fat;
pub mod dir_entry;
pub mod volume;
pub mod allocator;
pub mod directory_parser;
pub mod dir_snapshot;
pub mod diff;
pub mod fs;
