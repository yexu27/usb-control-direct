//! NBD 块设备发布模块。
//!
//! NBD 只负责 Linux NBD 协议、设备生命周期和块请求转发，不承载策略、病毒、文件树或 gadget 语义。

pub mod io;
pub mod protocol;
pub mod request_loop;
pub mod sysfs;

use std::os::unix::io::RawFd;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

pub use self::request_loop::run_request_loop;
pub use self::sysfs::{
    ensure_partition_scan_disabled, nbd_name_from_device_path, parse_nbd_max_part,
    read_nbd_partition_scan_status, NbdPartitionScanStatus,
};
use crate::exfat::layout::SECTOR_SIZE;

// Linux NBD ioctl 常量
const NBD_SET_SOCK: u64 = 0xAB00;
const NBD_SET_BLKSIZE: u64 = 0xAB01;
const NBD_SET_SIZE_BLOCKS: u64 = 0xAB07;
const NBD_SET_FLAGS: u64 = 0xAB0A;
const NBD_DO_IT: u64 = 0xAB03;
const NBD_CLEAR_SOCK: u64 = 0xAB04;
const NBD_CLEAR_QUE: u64 = 0xAB05;
const NBD_DISCONNECT: u64 = 0xAB08;

// NBD flags
const NBD_FLAG_HAS_FLAGS: u32 = 1;
const NBD_FLAG_READ_ONLY: u32 = 2;
const NBD_FLAG_SEND_FLUSH: u32 = 4;

pub fn disconnect_nbd_device(device: &Path) -> Result<(), std::io::Error> {
    let _ = nbd_name_from_device_path(device)?;
    let nbd_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(device)?;
    use std::os::unix::io::AsRawFd;
    let nbd_fd = nbd_file.as_raw_fd();

    // 安全性: nbd_fd 来自刚打开的 NBD 设备文件，ioctl 参数不携带用户指针。
    unsafe {
        let _ = nbd_ioctl(nbd_fd, NBD_DISCONNECT, 0);
        let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
        let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
    }

    Ok(())
}

pub fn disconnect_nbd_pool(pool_size: u32) {
    for idx in 0..pool_size {
        let device = PathBuf::from(format!("/dev/nbd{idx}"));
        if !device.exists() {
            continue;
        }
        match disconnect_nbd_device(&device) {
            Ok(()) => info!(device = %device.display(), "启动恢复: 断开旧 NBD 连接"),
            Err(e) => warn!(
                device = %device.display(),
                error = %e,
                "启动恢复: 断开旧 NBD 连接失败"
            ),
        }
    }
}

/// NBD 服务器。
pub struct NbdServer {
    /// /dev/nbdX 路径。
    nbd_device_path: PathBuf,
    /// /dev/nbdX 文件描述符。
    nbd_fd: Option<RawFd>,
    /// 用户空间侧 socket fd。
    user_fd: Option<RawFd>,
    /// 内核侧 socket fd（由 NBD_DO_IT 线程使用）。
    kernel_fd: Option<RawFd>,
    /// NBD 请求处理任务。
    request_loop_handle: Option<tokio::task::JoinHandle<()>>,
    /// NBD_DO_IT 线程完成通知。
    do_it_complete: Option<oneshot::Receiver<()>>,
}

impl NbdServer {
    /// 创建 NBD 服务器。
    ///
    /// 参数:
    ///   - nbd_device: NBD 设备路径（如 /dev/nbd0）。
    pub fn new(nbd_device: &Path) -> Self {
        NbdServer {
            nbd_device_path: nbd_device.to_path_buf(),
            nbd_fd: None,
            user_fd: None,
            kernel_fd: None,
            request_loop_handle: None,
            do_it_complete: None,
        }
    }

    /// 启动 NBD 服务。
    ///
    /// 1. 创建 socketpair
    /// 2. 设置 NBD 参数（block size / size / flags）
    /// 3. spawn_blocking 运行 NBD_DO_IT
    /// 4. 返回用户空间侧 fd 供请求循环使用
    pub fn start(&mut self, total_sectors: u64, readonly: bool) -> Result<RawFd, std::io::Error> {
        use std::os::unix::net::UnixStream;

        // 创建 socketpair
        let (kernel_sock, user_sock) = UnixStream::pair()?;

        // 打开 /dev/nbdX
        let nbd_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.nbd_device_path)?;

        use std::os::unix::io::AsRawFd;
        let nbd_fd = nbd_file.as_raw_fd();
        let kernel_fd = kernel_sock.as_raw_fd();

        unsafe {
            let _ = nbd_ioctl(nbd_fd, NBD_DISCONNECT, 0);
            let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
            let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
        }

        // 设置 NBD 参数
        // 安全性: nbd_fd 和 kernel_fd 来自刚打开的有效文件/socket，ioctl 参数均为合法值。
        unsafe {
            nbd_ioctl(nbd_fd, NBD_SET_BLKSIZE, SECTOR_SIZE as u64)?;
            nbd_ioctl(nbd_fd, NBD_SET_SIZE_BLOCKS, total_sectors)?;
            nbd_ioctl(nbd_fd, NBD_SET_SOCK, kernel_fd as u64)?;
            let mut flags = NBD_FLAG_HAS_FLAGS | NBD_FLAG_SEND_FLUSH;
            if readonly {
                flags |= NBD_FLAG_READ_ONLY;
            }
            nbd_ioctl(nbd_fd, NBD_SET_FLAGS, flags as u64)?;
        }

        let user_fd = user_sock.as_raw_fd();
        self.nbd_fd = Some(nbd_fd);
        self.user_fd = Some(user_fd);
        self.kernel_fd = Some(kernel_fd);

        // 保持文件描述符不被 drop（由 stop() 负责关闭）
        std::mem::forget(nbd_file);
        std::mem::forget(kernel_sock);
        std::mem::forget(user_sock);

        // spawn_blocking 运行 NBD_DO_IT
        let (tx, rx) = oneshot::channel();
        let nbd_fd_copy = nbd_fd;
        tokio::task::spawn_blocking(move || {
            info!("NBD_DO_IT 启动: fd={}", nbd_fd_copy);
            let result = unsafe { nbd_ioctl(nbd_fd_copy, NBD_DO_IT, 0) };
            match result {
                Ok(_) => info!("NBD_DO_IT 正常结束"),
                Err(e) => error!(reason = %e, "NBD_DO_IT 异常结束"),
            }
            unsafe {
                let _ = nbd_ioctl(nbd_fd_copy, NBD_CLEAR_SOCK, 0);
                let _ = nbd_ioctl(nbd_fd_copy, NBD_CLEAR_QUE, 0);
            }
            let _ = tx.send(());
        });
        self.do_it_complete = Some(rx);

        Ok(user_fd)
    }

    pub fn set_request_loop_handle(&mut self, handle: tokio::task::JoinHandle<()>) {
        self.request_loop_handle = Some(handle);
    }

    pub fn wait_ready(
        &self,
        expected_sectors: u64,
        timeout: Duration,
    ) -> Result<(), std::io::Error> {
        self.wait_ready_under(Path::new("/sys/block"), expected_sectors, timeout)
    }

    pub fn wait_ready_under(
        &self,
        sys_block_root: &Path,
        expected_sectors: u64,
        timeout: Duration,
    ) -> Result<(), std::io::Error> {
        let name = self
            .nbd_device_path
            .file_name()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("invalid NBD path: {}", self.nbd_device_path.display()),
                )
            })?
            .to_string_lossy()
            .to_string();

        let nbd_sys = sys_block_root.join(&name);
        let deadline = Instant::now() + timeout;
        let mut stable_matches = 0;

        loop {
            let pid_ready = std::fs::read_to_string(nbd_sys.join("pid"))
                .map(|value| {
                    let value = value.trim();
                    !value.is_empty() && value != "0"
                })
                .unwrap_or(false);
            let size_ready = std::fs::read_to_string(nbd_sys.join("size"))
                .ok()
                .and_then(|value| value.trim().parse::<u64>().ok())
                .map(|size| size == expected_sectors)
                .unwrap_or(false);

            if pid_ready && size_ready {
                stable_matches += 1;
                if stable_matches >= 2 {
                    return Ok(());
                }
            } else {
                stable_matches = 0;
            }

            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "NBD device {} not ready: expected size {} sectors",
                        self.nbd_device_path.display(),
                        expected_sectors
                    ),
                ));
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn wait_disconnected(&self, timeout: Duration) -> Result<(), std::io::Error> {
        self.wait_disconnected_under(Path::new("/sys/block"), timeout)
    }

    pub fn wait_disconnected_under(
        &self,
        sys_block_root: &Path,
        timeout: Duration,
    ) -> Result<(), std::io::Error> {
        let name = self
            .nbd_device_path
            .file_name()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("invalid NBD path: {}", self.nbd_device_path.display()),
                )
            })?
            .to_string_lossy()
            .to_string();

        let nbd_sys = sys_block_root.join(&name);
        let deadline = Instant::now() + timeout;

        loop {
            let pid_connected = std::fs::read_to_string(nbd_sys.join("pid"))
                .map(|value| {
                    let value = value.trim();
                    !value.is_empty() && value != "0"
                })
                .unwrap_or(false);

            if !pid_connected {
                return Ok(());
            }

            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "NBD device {} still connected",
                        self.nbd_device_path.display()
                    ),
                ));
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn disconnect_for_stop(&mut self) {
        if let Some(nbd_fd) = self.nbd_fd {
            // 安全性: nbd_fd 来自 start() 中 mem::forget 保持的有效文件描述符。
            unsafe {
                let _ = nbd_ioctl(nbd_fd, NBD_DISCONNECT, 0);
            }
        }
        if let Some(user_fd) = self.user_fd.take() {
            // 安全性: user_fd 来自 start() 中 mem::forget 保持的有效文件描述符。
            unsafe {
                libc::close(user_fd);
            }
        }
        if let Some(kernel_fd) = self.kernel_fd.take() {
            // 安全性: kernel_fd 来自 start() 中 mem::forget 保持的有效文件描述符。
            unsafe {
                libc::close(kernel_fd);
            }
        }
    }

    fn clear_and_close_nbd_fd(&mut self) {
        if let Some(nbd_fd) = self.nbd_fd.take() {
            // 安全性: nbd_fd 来自 start() 中 mem::forget 保持的有效文件描述符。
            unsafe {
                let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
                let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
                libc::close(nbd_fd);
            }
        }
    }

    pub async fn stop_async(&mut self) {
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

        if let Err(e) = self.wait_disconnected(Duration::from_secs(2)) {
            warn!(error = %e, "NBD 断开状态确认失败");
        }

        self.clear_and_close_nbd_fd();
    }

    /// 停止 NBD 服务。
    pub fn stop(&mut self) {
        self.disconnect_for_stop();
        self.clear_and_close_nbd_fd();
    }

    /// NBD 设备路径。
    pub fn device_path(&self) -> &Path {
        &self.nbd_device_path
    }
}

impl Drop for NbdServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// NBD ioctl 封装。
///
/// 安全性: 调用方必须确保 fd 为有效的 NBD 设备文件描述符。
unsafe fn nbd_ioctl(fd: RawFd, request: u64, arg: u64) -> Result<(), std::io::Error> {
    let ret = libc::ioctl(fd, request, arg);
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
