//! 真实 U 盘文件系统提交器。

use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RealFsCommitter {
    mount_root: PathBuf,
}

impl RealFsCommitter {
    pub fn new(mount_root: PathBuf) -> Self {
        Self { mount_root }
    }

    pub fn create_file(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let path = self.resolve_virtual_path(virtual_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        match OpenOptions::new().create_new(true).write(true).open(&path) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && path.is_file() => {}
            Err(e) => return Err(e),
        }
        self.sync_parent(&path)?;
        Ok(())
    }

    pub fn create_dir(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let path = self.resolve_virtual_path(virtual_path)?;
        match fs::create_dir_all(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => {}
            Err(e) => return Err(e),
        }
        self.sync_parent(&path)?;
        Ok(())
    }

    pub fn write_at(
        &self,
        virtual_path: &str,
        offset: u64,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.resolve_virtual_path(virtual_path)?)?;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(data)?;
        Ok(())
    }

    pub fn truncate(&self, virtual_path: &str, len: u64) -> Result<(), std::io::Error> {
        OpenOptions::new()
            .write(true)
            .open(self.resolve_virtual_path(virtual_path)?)?
            .set_len(len)
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<(), std::io::Error> {
        let from = self.resolve_virtual_path(from)?;
        let to = self.resolve_virtual_path(to)?;
        if !from.exists() && to.exists() {
            self.sync_parent(&to)?;
            return Ok(());
        }
        fs::rename(&from, &to)?;
        self.sync_parent(&from)?;
        self.sync_parent(&to)?;
        Ok(())
    }

    pub fn delete_file(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let path = self.resolve_virtual_path(virtual_path)?;
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        self.sync_parent(&path)
    }

    pub fn delete_dir(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        let path = self.resolve_virtual_path(virtual_path)?;
        match fs::remove_dir_all(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        self.sync_parent(&path)
    }

    pub fn flush_file(&self, virtual_path: &str) -> Result<(), std::io::Error> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.resolve_virtual_path(virtual_path)?)?
            .sync_all()
    }

    pub fn real_path_for_virtual(&self, virtual_path: &str) -> Result<PathBuf, std::io::Error> {
        self.resolve_virtual_path(virtual_path)
    }

    pub fn sync_mount_root(&self) -> Result<(), std::io::Error> {
        OpenOptions::new().read(true).open(&self.mount_root)?.sync_all()
    }

    fn sync_parent(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut current = path.parent();
        while let Some(parent) = current {
            match OpenOptions::new().read(true).open(parent) {
                Ok(dir) => {
                    dir.sync_all()?;
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    current = parent.parent();
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn resolve_virtual_path(&self, virtual_path: &str) -> Result<PathBuf, std::io::Error> {
        if !virtual_path.starts_with('/') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "virtual path must be absolute",
            ));
        }
        let mut resolved = self.mount_root.clone();
        for component in Path::new(virtual_path).components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Normal(part) => resolved.push(part),
                Component::ParentDir | Component::Prefix(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "virtual path escapes mount root",
                    ));
                }
            }
        }
        Ok(resolved)
    }
}
