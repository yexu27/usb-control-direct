//! 主服务就绪文件的原子发布与生命周期管理。

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use system_upgrade::ServiceReady;

static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// 就绪文件守卫。正常退出时删除当前进程发布的文件。
pub struct ReadinessGuard {
    path: PathBuf,
}

impl ReadinessGuard {
    /// 启动初始化前删除上一次异常退出留下的 ready 文件。
    pub fn clear_stale(path: impl AsRef<Path>) -> io::Result<()> {
        remove_and_sync_parent(path.as_ref())
    }

    /// 在所有依赖完成初始化后原子发布 ready 文件。
    pub fn publish(path: impl AsRef<Path>, ready: &ServiceReady) -> io::Result<Self> {
        let path = path.as_ref();
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "ready 文件缺少父目录"))?;
        fs::create_dir_all(parent)?;
        let bytes = serde_json::to_vec(ready)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temporary = temporary_path(path);
        let write_result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, path)?;
            sync_directory(parent)
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        write_result?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl Drop for ReadinessGuard {
    fn drop(&mut self) {
        let _ = remove_and_sync_parent(&self.path);
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ready.json");
    path.with_file_name(format!(".{name}.{}.{}.tmp", std::process::id(), sequence))
}

fn remove_and_sync_parent(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => match path.parent() {
            Some(parent) => sync_directory(parent),
            None => Ok(()),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}
