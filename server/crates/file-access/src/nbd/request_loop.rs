//! NBD 请求循环。
//!
//! 本模块只把 NBD 协议请求转发到块后端，不理解 exFAT、策略、扫描或 gadget。

use std::os::unix::io::RawFd;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use crate::block_backend::BlockBackend;

use super::io::{read_exact_fd, write_all_fd};
use super::protocol::{build_reply, NbdCommand, NbdRequest, NBD_EIO, NBD_REQUEST_SIZE};

/// 请求处理循环。
pub fn run_request_loop<B: BlockBackend>(user_fd: RawFd, backend: Arc<B>) {
    let mut request_buf = [0u8; NBD_REQUEST_SIZE];

    loop {
        if read_exact_fd(user_fd, &mut request_buf).is_err() {
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
            NbdCommand::Read => handle_read(user_fd, &req, backend.as_ref()),
            NbdCommand::Write => handle_write(user_fd, &req, backend.as_ref()),
            NbdCommand::Flush => handle_flush(user_fd, &req, backend.as_ref()),
            NbdCommand::Disconnect => {
                if let Err(e) = backend.shutdown() {
                    warn!(error = %e, "NBD DISCONNECT 前 shutdown 失败");
                }
                info!("NBD 收到 DISCONNECT");
                break;
            }
            NbdCommand::Unknown(cmd) => {
                warn!("NBD 未知命令: {}", cmd);
                let reply = build_reply(req.handle, NBD_EIO);
                let _ = write_all_fd(user_fd, &reply);
            }
        }
    }
}

fn read_backend_data<B: BlockBackend>(
    backend: &B,
    offset: u64,
    len: usize,
) -> Result<Vec<u8>, std::io::Error> {
    let data = backend.read_at(offset, len)?;
    if data.len() != len {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!(
                "NBD backend returned {} bytes for {} byte read",
                data.len(),
                len
            ),
        ));
    }
    Ok(data)
}

fn handle_read<B: BlockBackend>(user_fd: RawFd, req: &NbdRequest, backend: &B) {
    let mut response_data = Vec::with_capacity(16 + req.len as usize);
    match read_backend_data(backend, req.from, req.len as usize) {
        Ok(data) => {
            let reply = build_reply(req.handle, 0);
            response_data.extend_from_slice(&reply);
            response_data.extend_from_slice(&data);
            let _ = write_all_fd(user_fd, &response_data);
        }
        Err(e) => {
            warn!(
                offset = req.from,
                len = req.len,
                error = %e,
                "NBD READ 失败"
            );
            let reply = build_reply(req.handle, NBD_EIO);
            let _ = write_all_fd(user_fd, &reply);
        }
    }
}

fn handle_write<B: BlockBackend>(user_fd: RawFd, req: &NbdRequest, backend: &B) {
    let mut write_data = vec![0u8; req.len as usize];
    if read_exact_fd(user_fd, &mut write_data).is_err() {
        warn!("NBD WRITE 数据读取失败");
        return;
    }

    let error = match backend.write_at(req.from, &write_data) {
        Ok(()) => 0,
        Err(e) => {
            warn!(
                offset = req.from,
                len = req.len,
                error = %e,
                "NBD WRITE 失败"
            );
            NBD_EIO
        }
    };

    let reply = build_reply(req.handle, error);
    let _ = write_all_fd(user_fd, &reply);
}

fn handle_flush<B: BlockBackend>(user_fd: RawFd, req: &NbdRequest, backend: &B) {
    let error = match backend.flush() {
        Ok(()) => 0,
        Err(e) => {
            warn!(
                offset = req.from,
                len = req.len,
                error = %e,
                "NBD FLUSH 失败"
            );
            NBD_EIO
        }
    };
    let reply = build_reply(req.handle, error);
    let _ = write_all_fd(user_fd, &reply);
}
