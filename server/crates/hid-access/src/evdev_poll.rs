use std::time::Duration;

pub(crate) enum PollResult {
    Readable,
    Timeout,
    Interrupted,
    Error(std::io::Error),
}

pub(crate) fn poll_fd_readable(fd: i32, timeout: Duration) -> PollResult {
    let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    let mut poll_fd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };

    // 安全性: poll_fd 指向当前栈上的有效 pollfd；nfds=1；timeout 为有限毫秒。
    let result = unsafe { libc::poll(&mut poll_fd, 1, timeout_ms) };
    if result == 0 {
        return PollResult::Timeout;
    }
    if result < 0 {
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::Interrupted {
            return PollResult::Interrupted;
        }
        return PollResult::Error(error);
    }
    if poll_fd.revents & libc::POLLIN != 0 {
        PollResult::Readable
    } else {
        PollResult::Timeout
    }
}
