//! 虚拟 exFAT 卷。

pub mod bitmap;
pub mod bitmap_state;
pub mod boot;
pub mod commit_pipeline;
pub mod dir_entry;
pub mod dir_snapshot;
pub mod directory_parser;
pub mod directory_store;
pub mod fat;
pub mod fat_state;
pub mod fs;
pub mod layout;
pub mod metadata_overlay;
pub mod metadata_renderer;
pub mod metadata_state;
pub(crate) mod policy_rejection;
pub mod runtime_state;
pub mod sector_owner;
pub mod transaction;
pub mod transaction_resolver;
pub mod upcase;
pub mod volume;
pub mod write_interpreter;
