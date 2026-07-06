//! NBD fd I/O 工具。
//!
//! 本模块只封装 Unix fd 的精确读写，不解析 NBD 协议。

use std::os::unix::io::RawFd;

/// 判断 I/O 错误是否可重试。
pub fn is_retryable_io_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::Interrupted
}

/// 从 fd 精确读取指定长度。
pub fn read_exact_fd(fd: RawFd, buf: &mut [u8]) -> Result<(), std::io::Error> {
    let mut pos = 0;
    while pos < buf.len() {
        // 安全性: fd 由调用方保证为有效文件描述符，buf[pos..] 是有效可写内存区域。
        let n = unsafe {
            libc::read(
                fd,
                buf[pos..].as_mut_ptr() as *mut libc::c_void,
                buf.len() - pos,
            )
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if is_retryable_io_error(&err) {
                continue;
            }
            return Err(err);
        }
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "NBD socket closed while reading",
            ));
        }
        pos += n as usize;
    }
    Ok(())
}

/// 向 fd 写入全部数据。
pub fn write_all_fd(fd: RawFd, data: &[u8]) -> Result<(), std::io::Error> {
    let mut pos = 0;
    while pos < data.len() {
        // 安全性: fd 由调用方保证为有效文件描述符，data[pos..] 是有效只读内存区域。
        let n = unsafe {
            libc::write(
                fd,
                data[pos..].as_ptr() as *const libc::c_void,
                data.len() - pos,
            )
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if is_retryable_io_error(&err) {
                continue;
            }
            return Err(err);
        }
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "NBD socket wrote zero bytes",
            ));
        }
        pos += n as usize;
    }
    Ok(())
}
