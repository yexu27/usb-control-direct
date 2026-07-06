use file_access::nbd::io::is_retryable_io_error;
use file_access::nbd::protocol::{
    build_reply, NbdCommand, NbdRequest, NBD_REPLY_MAGIC, NBD_REQUEST_MAGIC,
};

fn request(command: u32, handle: u64, from: u64, len: u32) -> [u8; 28] {
    let mut buf = [0u8; 28];
    buf[0..4].copy_from_slice(&NBD_REQUEST_MAGIC.to_be_bytes());
    buf[4..8].copy_from_slice(&command.to_be_bytes());
    buf[8..16].copy_from_slice(&handle.to_be_bytes());
    buf[16..24].copy_from_slice(&from.to_be_bytes());
    buf[24..28].copy_from_slice(&len.to_be_bytes());
    buf
}

#[test]
fn parse_read_request() {
    let req = NbdRequest::parse(&request(0, 42, 1024, 512)).unwrap();

    assert_eq!(req.command, NbdCommand::Read);
    assert_eq!(req.handle, 42);
    assert_eq!(req.from, 1024);
    assert_eq!(req.len, 512);
}

#[test]
fn parse_write_request_masks_flags_from_command_type() {
    let req = NbdRequest::parse(&request(0x0001_0001, 100, 512, 512)).unwrap();

    assert_eq!(req.command, NbdCommand::Write);
    assert_eq!(req.handle, 100);
    assert_eq!(req.from, 512);
}

#[test]
fn parse_disconnect_request() {
    let req = NbdRequest::parse(&request(2, 0, 0, 0)).unwrap();

    assert_eq!(req.command, NbdCommand::Disconnect);
}

#[test]
fn parse_invalid_magic_returns_none() {
    let buf = [0u8; 28];

    assert!(NbdRequest::parse(&buf).is_none());
}

#[test]
fn build_success_reply_header() {
    let reply = build_reply(42, 0);

    assert_eq!(reply.len(), 16);
    assert_eq!(
        u32::from_be_bytes(reply[0..4].try_into().unwrap()),
        NBD_REPLY_MAGIC
    );
    assert_eq!(u32::from_be_bytes(reply[4..8].try_into().unwrap()), 0);
    assert_eq!(u64::from_be_bytes(reply[8..16].try_into().unwrap()), 42);
}

#[test]
fn interrupted_io_is_retryable() {
    let err = std::io::Error::from(std::io::ErrorKind::Interrupted);

    assert!(is_retryable_io_error(&err));
}
