//! S04 文件系统操作准入检查。
//!
//! 本模块只处理路径级策略和只读权限。已有 VFS 节点是否为阻断占位文件，
//! 由 `ExfatRuntimeState` 基于节点状态判断，避免 OperationGuard 依赖 VFS 索引。

use crate::types::{ExecFileType, PolicySnapshot};

#[derive(Debug, Clone)]
pub enum FsOperation {
    CreateFile {
        virtual_path: String,
    },
    CreateDir {
        virtual_path: String,
    },
    WriteFile {
        virtual_path: String,
    },
    WriteExecutable {
        virtual_path: String,
        exec_type: ExecFileType,
    },
    Truncate {
        virtual_path: String,
    },
    Rename {
        from: String,
        to: String,
    },
    Delete {
        virtual_path: String,
    },
}

#[derive(Debug, Clone)]
pub struct OperationGuard {
    snapshot: PolicySnapshot,
}

impl OperationGuard {
    pub fn new(snapshot: PolicySnapshot) -> Self {
        Self { snapshot }
    }

    pub fn check(&self, op: &FsOperation) -> Result<(), std::io::Error> {
        if self.snapshot.permission == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "只读权限禁止写入",
            ));
        }

        match op {
            FsOperation::CreateFile { virtual_path }
            | FsOperation::CreateDir { virtual_path }
            | FsOperation::WriteFile { virtual_path }
            | FsOperation::Truncate { virtual_path } => self.check_path(virtual_path),
            FsOperation::Rename { to, .. } => self.check_path(to),
            FsOperation::Delete { virtual_path } => self.check_path(virtual_path),
            FsOperation::WriteExecutable {
                virtual_path,
                exec_type,
            } => {
                if self.snapshot.exec_control_enabled {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("可执行文件禁止写入: {:?} {}", exec_type, virtual_path),
                    ));
                }
                self.check_path(virtual_path)
            }
        }
    }

    fn check_path(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        if self.snapshot.file_type_blacklist_enabled {
            let lower = virtual_path.to_ascii_lowercase();
            for ext in &self.snapshot.blacklist_extensions {
                if lower.ends_with(ext) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("文件类型黑名单禁止访问: {}", ext),
                    ));
                }
            }
        }
        Ok(())
    }
}
