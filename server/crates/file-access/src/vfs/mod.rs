//! S04 受控虚拟文件系统模型。

pub mod committer;
pub mod index;
pub mod journal;
pub mod node;
pub mod operation_guard;

pub use index::VfsIndex;
pub use node::{NodeId, VfsNode, VfsNodeKind};
