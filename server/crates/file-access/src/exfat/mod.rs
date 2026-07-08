//! 虚拟 exFAT 卷。

pub mod layout;
pub mod boot;
pub mod upcase;
pub mod bitmap;
pub mod bitmap_state;
pub mod fat;
pub mod fat_state;
pub mod commit_pipeline;
pub mod dir_entry;
pub mod volume;
pub mod directory_store;
pub mod directory_parser;
pub mod dir_snapshot;
pub mod fs;
pub mod metadata_overlay;
pub mod metadata_renderer;
pub mod metadata_state;
pub mod runtime_state;
pub mod sector_owner;
pub mod transaction;
pub mod transaction_resolver;
pub mod write_interpreter;
