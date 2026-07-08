//! exFAT filesystem mutation commit pipeline.

use crate::types::PolicySnapshot;
use crate::vfs::committer::RealFsCommitter;
use crate::vfs::mutation::{FsMutation, NodeKind};
use crate::vfs::operation_guard::{FsOperation, OperationGuard};
use crate::vfs::VfsIndex;

pub struct CommitPipeline<'a> {
    index: &'a VfsIndex,
    committer: &'a RealFsCommitter,
    snapshot: PolicySnapshot,
}

impl<'a> CommitPipeline<'a> {
    pub fn new(
        index: &'a VfsIndex,
        committer: &'a RealFsCommitter,
        snapshot: PolicySnapshot,
    ) -> Self {
        Self {
            index,
            committer,
            snapshot,
        }
    }

    pub fn check_and_commit_real_fs(&self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        self.check_mutation(mutation)?;
        self.commit_real_fs(mutation).map_err(|err| {
            std::io::Error::new(
                err.kind(),
                format!("exFAT real filesystem commit failed; session must fail-close: {err}"),
            )
        })
    }

    fn check_mutation(&self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::WriteFile { virtual_path, .. }
            | FsMutation::RewriteFile { virtual_path, .. }
            | FsMutation::Truncate { virtual_path, .. }
            | FsMutation::Delete { virtual_path, .. } => {
                self.deny_if_blocked_placeholder(virtual_path, mutation_name(mutation))?;
            }
            FsMutation::Rename { from, to, .. } => {
                self.deny_if_blocked_placeholder(from, "rename_from")?;
                self.deny_if_blocked_placeholder(to, "rename_to")?;
            }
            FsMutation::CreateDir { .. } | FsMutation::CreateFile { .. } => {}
        }

        let guard = OperationGuard::new(self.snapshot.clone());
        match mutation {
            FsMutation::CreateDir { parent, name, .. } => guard.check(&FsOperation::CreateDir {
                virtual_path: join_virtual_path(parent, name),
            }),
            FsMutation::CreateFile { parent, name, .. } => guard.check(&FsOperation::CreateFile {
                virtual_path: join_virtual_path(parent, name),
            }),
            FsMutation::WriteFile { virtual_path, .. }
            | FsMutation::RewriteFile { virtual_path, .. } => {
                guard.check(&FsOperation::WriteFile {
                    virtual_path: virtual_path.clone(),
                })
            }
            FsMutation::Truncate { virtual_path, .. } => guard.check(&FsOperation::Truncate {
                virtual_path: virtual_path.clone(),
            }),
            FsMutation::Rename { from, to, .. } => guard.check(&FsOperation::Rename {
                from: from.clone(),
                to: to.clone(),
            }),
            FsMutation::Delete { virtual_path, .. } => guard.check(&FsOperation::Delete {
                virtual_path: virtual_path.clone(),
            }),
        }
    }

    fn deny_if_blocked_placeholder(
        &self,
        virtual_path: &str,
        operation: &str,
    ) -> Result<(), std::io::Error> {
        if let Some(id) = self.index.lookup_path(virtual_path) {
            if let Some(node) = self.index.node(id) {
                if node.is_blocked_placeholder() {
                    tracing::warn!(
                        virtual_path = %node.virtual_path,
                        reason = node.blocked_reason().unwrap_or("unknown"),
                        operation,
                        "阻断占位文件禁止变更"
                    );
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "阻断占位文件禁止变更",
                    ));
                }
            }
        }
        Ok(())
    }

    fn commit_real_fs(&self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::CreateDir { parent, name, .. } => {
                self.committer.create_dir(&join_virtual_path(parent, name))
            }
            FsMutation::CreateFile {
                parent,
                name,
                data_patches,
                ..
            } => {
                let path = join_virtual_path(parent, name);
                self.committer.create_file(&path)?;
                for patch in data_patches {
                    self.committer
                        .write_at(&patch.virtual_path, patch.offset, &patch.data)?;
                    self.committer.flush_file(&patch.virtual_path)?;
                }
                Ok(())
            }
            FsMutation::WriteFile {
                virtual_path,
                offset,
                data,
            } => {
                self.committer.write_at(virtual_path, *offset, data)?;
                self.committer.flush_file(virtual_path)
            }
            FsMutation::Truncate { virtual_path, len } => {
                self.committer.truncate(virtual_path, *len)?;
                self.committer.flush_file(virtual_path)
            }
            FsMutation::Rename { from, to, .. } => self.committer.rename(from, to),
            FsMutation::Delete { virtual_path, kind } => match kind {
                NodeKind::File => self.committer.delete_file(virtual_path),
                NodeKind::Directory => self.committer.delete_dir(virtual_path),
            },
            FsMutation::RewriteFile {
                virtual_path,
                size,
                data_patches,
                ..
            } => {
                self.committer.truncate(virtual_path, *size)?;
                for patch in data_patches {
                    self.committer
                        .write_at(&patch.virtual_path, patch.offset, &patch.data)?;
                }
                self.committer.flush_file(virtual_path)
            }
        }
    }
}

fn mutation_name(mutation: &FsMutation) -> &'static str {
    match mutation {
        FsMutation::CreateDir { .. } => "create_dir",
        FsMutation::CreateFile { .. } => "create_file",
        FsMutation::WriteFile { .. } => "write_file",
        FsMutation::Truncate { .. } => "truncate",
        FsMutation::Rename { .. } => "rename",
        FsMutation::Delete { .. } => "delete",
        FsMutation::RewriteFile { .. } => "rewrite_file",
    }
}

fn join_virtual_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}
