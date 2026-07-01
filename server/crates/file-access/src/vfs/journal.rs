//! S04 写事务。

use crate::vfs::committer::RealFsCommitter;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub enum FileMutation {
    CreateFile {
        virtual_path: String,
    },
    CreateDir {
        virtual_path: String,
    },
    Write {
        virtual_path: String,
        offset: u64,
        data: Vec<u8>,
    },
    Truncate {
        virtual_path: String,
        len: u64,
    },
    Rename {
        from: String,
        to: String,
    },
    DeleteFile {
        virtual_path: String,
    },
    DeleteDir {
        virtual_path: String,
    },
}

#[derive(Debug, Default)]
pub struct WriteJournal {
    mutations: Vec<FileMutation>,
}

impl WriteJournal {
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    pub fn record(&mut self, mutation: FileMutation) {
        self.mutations.push(mutation);
    }

    pub fn is_dirty(&self) -> bool {
        !self.mutations.is_empty()
    }

    pub fn flush(&mut self, committer: &RealFsCommitter) -> Result<(), std::io::Error> {
        for mutation in &self.mutations {
            let result = match mutation {
                FileMutation::CreateFile { virtual_path } => committer.create_file(virtual_path),
                FileMutation::CreateDir { virtual_path } => committer.create_dir(virtual_path),
                FileMutation::Write {
                    virtual_path,
                    offset,
                    data,
                } => committer.write_at(virtual_path, *offset, data),
                FileMutation::Truncate { virtual_path, len } => {
                    committer.truncate(virtual_path, *len)
                }
                FileMutation::Rename { from, to } => committer.rename(from, to),
                FileMutation::DeleteFile { virtual_path } => committer.delete_file(virtual_path),
                FileMutation::DeleteDir { virtual_path } => committer.delete_dir(virtual_path),
            };
            result_with_context(result, "apply", mutation)?;
        }

        for virtual_path in sync_file_paths(&self.mutations) {
            result_with_context(
                committer.flush_file(&virtual_path),
                "sync",
                &FileMutation::Write {
                    virtual_path,
                    offset: 0,
                    data: Vec::new(),
                },
            )?;
        }

        committer.sync_mount_root()?;
        self.mutations.clear();
        Ok(())
    }

    pub fn discard(&mut self) {
        self.mutations.clear();
    }
}

fn sync_file_paths(mutations: &[FileMutation]) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();

    for mutation in mutations {
        match mutation {
            FileMutation::CreateFile { virtual_path }
            | FileMutation::Write { virtual_path, .. }
            | FileMutation::Truncate { virtual_path, .. } => {
                paths.insert(virtual_path.clone());
            }
            FileMutation::Rename { from, to } => {
                paths = paths
                    .into_iter()
                    .map(|path| remap_path_prefix(&path, from, to))
                    .collect();
                paths.insert(to.clone());
            }
            FileMutation::DeleteFile { virtual_path } => {
                paths.remove(virtual_path);
            }
            FileMutation::DeleteDir { virtual_path } => {
                paths.retain(|path| !is_same_path_or_child(path, virtual_path));
            }
            FileMutation::CreateDir { .. } => {}
        }
    }

    paths
}

fn remap_path_prefix(path: &str, from: &str, to: &str) -> String {
    if path == from {
        return to.to_string();
    }

    let from_prefix = format!("{}/", from.trim_end_matches('/'));
    if let Some(rest) = path.strip_prefix(&from_prefix) {
        format!("{}/{}", to.trim_end_matches('/'), rest)
    } else {
        path.to_string()
    }
}

fn is_same_path_or_child(path: &str, parent: &str) -> bool {
    if path == parent {
        return true;
    }
    let parent_prefix = format!("{}/", parent.trim_end_matches('/'));
    path.starts_with(&parent_prefix)
}

fn result_with_context(
    result: Result<(), std::io::Error>,
    phase: &str,
    mutation: &FileMutation,
) -> Result<(), std::io::Error> {
    result.map_err(|err| {
        std::io::Error::new(
            err.kind(),
            format!(
                "提交真实文件系统变更失败: phase={}, mutation={:?}, error={}",
                phase, mutation, err
            ),
        )
    })
}
