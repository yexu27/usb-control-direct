//! S04 受控虚拟文件系统模型。

pub mod committer;
pub mod index;
pub mod mutation;
pub mod node;
pub mod operation_guard;

pub use index::VfsIndex;
pub use mutation::{ClusterChain, FileDataPatch, FsMutation, NodeKind};
pub use node::{NodeId, VfsNode, VfsNodeKind};
