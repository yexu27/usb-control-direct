//! S04 写事务。

use crate::vfs::committer::RealFsCommitter;

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
            match mutation {
                FileMutation::CreateFile { virtual_path } => committer.create_file(virtual_path)?,
                FileMutation::CreateDir { virtual_path } => committer.create_dir(virtual_path)?,
                FileMutation::Write {
                    virtual_path,
                    offset,
                    data,
                } => committer.write_at(virtual_path, *offset, data)?,
                FileMutation::Truncate { virtual_path, len } => {
                    committer.truncate(virtual_path, *len)?
                }
                FileMutation::Rename { from, to } => committer.rename(from, to)?,
                FileMutation::DeleteFile { virtual_path } => committer.delete_file(virtual_path)?,
                FileMutation::DeleteDir { virtual_path } => committer.delete_dir(virtual_path)?,
            }
        }

        for mutation in &self.mutations {
            match mutation {
                FileMutation::CreateFile { virtual_path }
                | FileMutation::Write { virtual_path, .. }
                | FileMutation::Truncate { virtual_path, .. } => {
                    committer.flush_file(virtual_path)?
                }
                FileMutation::Rename { to, .. } => committer.flush_file(to)?,
                FileMutation::CreateDir { .. }
                | FileMutation::DeleteFile { .. }
                | FileMutation::DeleteDir { .. } => {}
            }
        }

        committer.sync_mount_root()?;
        self.mutations.clear();
        Ok(())
    }

    pub fn discard(&mut self) {
        self.mutations.clear();
    }
}
