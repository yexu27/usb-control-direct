//! NBD 设备生命周期。
//!
//! `NbdDevice` 表示一次 active `/dev/nbdX` 连接，负责停止时释放 fd、socket、request loop 和 NBD_DO_IT。

use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

use crate::block_backend::BlockBackend;

use super::request_loop::run_request_loop;
use super::sysfs::{nbd_name_from_device_path, NbdSysfs};

pub(crate) const NBD_SET_SOCK: u64 = 0xAB00;
pub(crate) const NBD_SET_BLKSIZE: u64 = 0xAB01;
pub(crate) const NBD_SET_SIZE_BLOCKS: u64 = 0xAB07;
pub(crate) const NBD_SET_FLAGS: u64 = 0xAB0A;
pub(crate) const NBD_DO_IT: u64 = 0xAB03;
pub(crate) const NBD_CLEAR_SOCK: u64 = 0xAB04;
pub(crate) const NBD_CLEAR_QUE: u64 = 0xAB05;
pub(crate) const NBD_DISCONNECT: u64 = 0xAB08;

const NBD_FLAG_HAS_FLAGS: u32 = 1;
const NBD_FLAG_READ_ONLY: u32 = 2;
const NBD_FLAG_SEND_FLUSH: u32 = 4;
const NBD_BLOCK_SIZE: u64 = 512;

/// 一次 active NBD 连接。
pub struct NbdDevice {
    path: PathBuf,
    nbd_fd: Option<RawFd>,
    user_fd: Option<RawFd>,
    kernel_fd: Option<RawFd>,
    request_loop_handle: Option<tokio::task::JoinHandle<()>>,
    do_it_complete: Option<oneshot::Receiver<()>>,
}

impl NbdDevice {
    pub(crate) fn start(
        path: PathBuf,
        total_sectors: u64,
        readonly: bool,
        backend: Arc<dyn BlockBackend>,
    ) -> Result<Self, std::io::Error> {
        use std::os::unix::net::UnixStream;

        let (kernel_sock, user_sock) = UnixStream::pair()?;
        let nbd_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;
        let nbd_fd_for_ioctl = nbd_file.as_raw_fd();
        let kernel_fd_for_ioctl = kernel_sock.as_raw_fd();

        // 安全性: nbd_fd 和 kernel_fd 来自刚打开的有效文件/socket，ioctl 参数均为合法值。
        unsafe {
            nbd_ioctl(nbd_fd_for_ioctl, NBD_SET_BLKSIZE, NBD_BLOCK_SIZE)?;
            nbd_ioctl(nbd_fd_for_ioctl, NBD_SET_SIZE_BLOCKS, total_sectors)?;
            nbd_ioctl(nbd_fd_for_ioctl, NBD_SET_SOCK, kernel_fd_for_ioctl as u64)?;
            let mut flags = NBD_FLAG_HAS_FLAGS | NBD_FLAG_SEND_FLUSH;
            if readonly {
                flags |= NBD_FLAG_READ_ONLY;
            }
            nbd_ioctl(nbd_fd_for_ioctl, NBD_SET_FLAGS, flags as u64)?;
        }

        let nbd_fd = nbd_file.into_raw_fd();
        let kernel_fd = kernel_sock.into_raw_fd();
        let user_fd = user_sock.into_raw_fd();

        let (tx, rx) = oneshot::channel();
        let do_it_fd = nbd_fd;
        tokio::task::spawn_blocking(move || {
            info!(fd = do_it_fd, "NBD_DO_IT 启动");
            let result = unsafe { nbd_ioctl(do_it_fd, NBD_DO_IT, 0) };
            match result {
                Ok(_) => info!("NBD_DO_IT 正常结束"),
                Err(e) => error!(reason = %e, "NBD_DO_IT 异常结束"),
            }
            // 安全性: do_it_fd 是本 NbdDevice 持有的有效 NBD fd。
            unsafe {
                let _ = nbd_ioctl(do_it_fd, NBD_CLEAR_SOCK, 0);
                let _ = nbd_ioctl(do_it_fd, NBD_CLEAR_QUE, 0);
            }
            let _ = tx.send(());
        });

        let request_loop_handle = tokio::task::spawn_blocking(move || {
            run_request_loop(user_fd, backend);
        });

        Ok(Self {
            path,
            nbd_fd: Some(nbd_fd),
            user_fd: Some(user_fd),
            kernel_fd: Some(kernel_fd),
            request_loop_handle: Some(request_loop_handle),
            do_it_complete: Some(rx),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn stop(&mut self) {
        self.disconnect_for_stop();

        if let Some(handle) = self.request_loop_handle.take() {
            if let Err(e) = handle.await {
                warn!(error = %e, "NBD request loop join 失败");
            }
        }

        if let Some(done) = self.do_it_complete.take() {
            match tokio::time::timeout(Duration::from_secs(2), done).await {
                Ok(Ok(())) => debug!("NBD_DO_IT 已退出"),
                Ok(Err(_)) => warn!("NBD_DO_IT 完成通知已关闭"),
                Err(_) => warn!("等待 NBD_DO_IT 退出超时"),
            }
        }

        if let Ok(name) = nbd_name_from_device_path(&self.path) {
            if let Err(e) = NbdSysfs::default().wait_disconnected(&name, Duration::from_secs(2)) {
                warn!(error = %e, "NBD 断开状态确认失败");
            }
        }

        self.clear_and_close_nbd_fd();
    }

    fn disconnect_for_stop(&mut self) {
        if let Some(nbd_fd) = self.nbd_fd {
            // 安全性: nbd_fd 由本 NbdDevice 持有，尚未 close。
            unsafe {
                let _ = nbd_ioctl(nbd_fd, NBD_DISCONNECT, 0);
            }
        }
        self.close_socket_fds();
    }

    fn close_socket_fds(&mut self) {
        if let Some(user_fd) = self.user_fd.take() {
            // 安全性: user_fd 由本 NbdDevice 持有，尚未 close。
            unsafe { libc::close(user_fd) };
        }
        if let Some(kernel_fd) = self.kernel_fd.take() {
            // 安全性: kernel_fd 由本 NbdDevice 持有，尚未 close。
            unsafe { libc::close(kernel_fd) };
        }
    }

    fn clear_and_close_nbd_fd(&mut self) {
        if let Some(nbd_fd) = self.nbd_fd.take() {
            // 安全性: nbd_fd 由本 NbdDevice 持有，尚未 close。
            unsafe {
                let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
                let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
                libc::close(nbd_fd);
            }
        }
    }
}

impl Drop for NbdDevice {
    fn drop(&mut self) {
        self.disconnect_for_stop();
        self.clear_and_close_nbd_fd();
    }
}

pub(crate) unsafe fn nbd_ioctl(fd: RawFd, request: u64, arg: u64) -> Result<(), std::io::Error> {
    let ret = libc::ioctl(fd, request, arg);
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
