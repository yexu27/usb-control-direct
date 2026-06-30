//! NBD 块设备后端。
//!
//! 通过 socketpair + ioctl 对接 Linux 内核 NBD 驱动。
//! 用户空间侧处理 28 字节请求，返回 16 字节响应头 + 数据。
//! NBD_DO_IT ioctl 通过 tokio::task::spawn_blocking 运行。

use std::io::Read;
use std::os::unix::io::RawFd;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

use crate::exfat::layout::SECTOR_SIZE;
use crate::exfat::volume::VirtualVolume;
use crate::types::SectorContent;
use crate::write_back::WriteBackManager;

/// NBD 请求魔数。
pub const NBD_REQUEST_MAGIC: u32 = 0x25609513;

/// NBD 响应魔数。
pub const NBD_REPLY_MAGIC: u32 = 0x67446698;

/// NBD 请求大小（字节）。
const NBD_REQUEST_SIZE: usize = 28;

/// NBD 命令类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbdCommand {
    Read,
    Write,
    Disconnect,
    Flush,
    Unknown(u32),
}

/// NBD 请求。
#[derive(Debug)]
pub struct NbdRequest {
    pub command: NbdCommand,
    pub handle: u64,
    pub from: u64,
    pub len: u32,
}

impl NbdRequest {
    /// 解析 28 字节 NBD 请求。
    pub fn parse(buf: &[u8; NBD_REQUEST_SIZE]) -> Option<Self> {
        let magic = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        if magic != NBD_REQUEST_MAGIC {
            return None;
        }

        let type_val = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let command = match type_val & 0xFFFF {
            0 => NbdCommand::Read,
            1 => NbdCommand::Write,
            2 => NbdCommand::Disconnect,
            3 => NbdCommand::Flush,
            other => NbdCommand::Unknown(other),
        };

        let handle = u64::from_be_bytes(buf[8..16].try_into().unwrap());
        let from = u64::from_be_bytes(buf[16..24].try_into().unwrap());
        let len = u32::from_be_bytes(buf[24..28].try_into().unwrap());

        Some(NbdRequest {
            command,
            handle,
            from,
            len,
        })
    }
}

/// 构建 16 字节 NBD 响应头。
pub fn build_reply(handle: u64, error: u32) -> Vec<u8> {
    let mut reply = Vec::with_capacity(16);
    reply.extend_from_slice(&NBD_REPLY_MAGIC.to_be_bytes());
    reply.extend_from_slice(&error.to_be_bytes());
    reply.extend_from_slice(&handle.to_be_bytes());
    reply
}

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

/// I/O error code (EIO = 5)。
const EIO: u32 = 5;

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

    /// 停止 NBD 服务。
    pub fn stop(&mut self) {
        if let Some(nbd_fd) = self.nbd_fd.take() {
            // 安全性: nbd_fd 来自 start() 中 mem::forget 保持的有效文件描述符。
            unsafe {
                let _ = nbd_ioctl(nbd_fd, NBD_DISCONNECT, 0);
                let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
                let _ = nbd_ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
                libc::close(nbd_fd);
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

/// 请求处理循环。
///
/// 从 user_fd 读取 NBD 请求，按策略处理，写回响应。
pub fn run_request_loop(user_fd: RawFd, volume: &VirtualVolume, write_back: &mut WriteBackManager) {
    let mut request_buf = [0u8; NBD_REQUEST_SIZE];

    loop {
        // 读取 28 字节请求
        if read_exact(user_fd, &mut request_buf).is_err() {
            debug!("NBD 连接断开");
            break;
        }

        let req = match NbdRequest::parse(&request_buf) {
            Some(r) => r,
            None => {
                error!("NBD 请求解析失败");
                break;
            }
        };

        match req.command {
            NbdCommand::Read => handle_read(user_fd, &req, volume),
            NbdCommand::Write => handle_write(user_fd, &req, volume, write_back),
            NbdCommand::Flush => {
                let _ = write_back.flush();
                let reply = build_reply(req.handle, 0);
                let _ = write_all(user_fd, &reply);
            }
            NbdCommand::Disconnect => {
                info!("NBD 收到 DISCONNECT");
                break;
            }
            NbdCommand::Unknown(cmd) => {
                warn!("NBD 未知命令: {}", cmd);
                let reply = build_reply(req.handle, EIO);
                let _ = write_all(user_fd, &reply);
            }
        }
    }
}

/// 处理 READ 请求。
fn handle_read(user_fd: RawFd, req: &NbdRequest, volume: &VirtualVolume) {
    let sector = req.from / SECTOR_SIZE as u64;
    let num_sectors = req.len / SECTOR_SIZE;
    let mut response_data = Vec::with_capacity(16 + req.len as usize);
    let mut has_error = false;

    let mut data_buf = Vec::with_capacity(req.len as usize);

    for i in 0..num_sectors {
        let s = sector + i as u64;
        let content = volume.read_sector(s);

        match content {
            SectorContent::Metadata(data) => {
                data_buf.extend_from_slice(&data);
            }
            SectorContent::FileData {
                real_path,
                offset,
                valid_bytes,
                blocked,
            } => {
                if blocked {
                    has_error = true;
                    break;
                }
                // 从真实文件读取数据
                match read_file_data(&real_path, offset, valid_bytes) {
                    Ok(data) => {
                        let mut sector_data = data;
                        sector_data.resize(SECTOR_SIZE as usize, 0);
                        data_buf.extend_from_slice(&sector_data);
                    }
                    Err(e) => {
                        warn!("读取文件数据失败: {}: {}", real_path.display(), e);
                        has_error = true;
                        break;
                    }
                }
            }
            SectorContent::Zero => {
                data_buf.extend(vec![0u8; SECTOR_SIZE as usize]);
            }
        }
    }

    if has_error {
        let reply = build_reply(req.handle, EIO);
        let _ = write_all(user_fd, &reply);
    } else {
        let reply = build_reply(req.handle, 0);
        response_data.extend_from_slice(&reply);
        response_data.extend_from_slice(&data_buf);
        let _ = write_all(user_fd, &response_data);
    }
}

/// 处理 WRITE 请求。
fn handle_write(
    user_fd: RawFd,
    req: &NbdRequest,
    volume: &VirtualVolume,
    write_back: &mut WriteBackManager,
) {
    // 先读取写入数据
    let mut write_data = vec![0u8; req.len as usize];
    if read_exact(user_fd, &mut write_data).is_err() {
        warn!("NBD WRITE 数据读取失败");
        return;
    }

    let sector = req.from / SECTOR_SIZE as u64;
    let num_sectors = req.len / SECTOR_SIZE;

    let mut error = 0u32;
    for i in 0..num_sectors {
        let s = sector + i as u64;
        let offset = (i as usize) * SECTOR_SIZE as usize;
        let data = &write_data[offset..offset + SECTOR_SIZE as usize];

        if let Err(e) = write_back.handle_write(s, data, volume) {
            warn!("写回失败: sector={}, error={}", s, e);
            error = EIO;
            break;
        }
    }

    let reply = build_reply(req.handle, error);
    let _ = write_all(user_fd, &reply);
}

/// 从真实文件读取数据。
fn read_file_data(path: &Path, offset: u64, valid_bytes: u32) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Seek;

    let mut file = std::fs::File::open(path)?;
    file.seek(std::io::SeekFrom::Start(offset))?;
    let mut buf = vec![0u8; valid_bytes as usize];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

/// 从 fd 精确读取。
fn read_exact(fd: RawFd, buf: &mut [u8]) -> Result<(), std::io::Error> {
    let mut pos = 0;
    while pos < buf.len() {
        // 安全性: fd 为有效文件描述符，buf[pos..] 为有效可写内存区域。
        let n = unsafe {
            libc::read(
                fd,
                buf[pos..].as_mut_ptr() as *mut libc::c_void,
                buf.len() - pos,
            )
        };
        if n <= 0 {
            return Err(std::io::Error::last_os_error());
        }
        pos += n as usize;
    }
    Ok(())
}

/// 向 fd 写入全部数据。
fn write_all(fd: RawFd, data: &[u8]) -> Result<(), std::io::Error> {
    let mut pos = 0;
    while pos < data.len() {
        // 安全性: fd 为有效文件描述符，data[pos..] 为有效只读内存区域。
        let n = unsafe {
            libc::write(
                fd,
                data[pos..].as_ptr() as *const libc::c_void,
                data.len() - pos,
            )
        };
        if n <= 0 {
            return Err(std::io::Error::last_os_error());
        }
        pos += n as usize;
    }
    Ok(())
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
