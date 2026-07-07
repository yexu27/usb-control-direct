//! 受控虚拟文件系统节点。

use std::path::PathBuf;

use crate::types::ExecFileType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsNodeKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VfsFileView {
    /// 普通真实文件，读写允许时映射到真实 U 盘文件。
    RealFile,
    /// 被策略阻断的占位文件，读路径只能返回占位内容。
    BlockedPlaceholder {
        reason: String,
        real_size: u64,
        placeholder_size: u64,
    },
}

#[derive(Debug, Clone)]
pub struct VfsNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub name: String,
    pub virtual_path: String,
    pub real_path: PathBuf,
    pub kind: VfsNodeKind,
    pub size: u64,
    pub file_view: Option<VfsFileView>,
    pub first_cluster: Option<u32>,
    pub is_virus: bool,
    pub exec_type: Option<ExecFileType>,
    pub extension: String,
    pub is_autorun_target: bool,
    pub is_autorun_inf: bool,
    pub is_root_shell_script: bool,
    pub children: Vec<NodeId>,
}

impl VfsNode {
    pub fn is_dir(&self) -> bool {
        self.kind == VfsNodeKind::Directory
    }

    pub fn is_blocked_placeholder(&self) -> bool {
        matches!(
            self.file_view,
            Some(VfsFileView::BlockedPlaceholder { .. })
        )
    }

    pub fn blocked_reason(&self) -> Option<&str> {
        match &self.file_view {
            Some(VfsFileView::BlockedPlaceholder { reason, .. }) => Some(reason.as_str()),
            _ => None,
        }
    }
}
