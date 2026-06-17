use file_access::nbd::{NbdRequest, NbdCommand, NBD_REQUEST_MAGIC};

#[test]
fn parse_read_request() {
    let mut buf = [0u8; 28];
    buf[0..4].copy_from_slice(&NBD_REQUEST_MAGIC.to_be_bytes());
    buf[4..8].copy_from_slice(&0u32.to_be_bytes());
    buf[8..16].copy_from_slice(&42u64.to_be_bytes());
    buf[16..24].copy_from_slice(&(1024u64).to_be_bytes());
    buf[24..28].copy_from_slice(&512u32.to_be_bytes());

    let req = NbdRequest::parse(&buf).unwrap();
    assert_eq!(req.command, NbdCommand::Read);
    assert_eq!(req.handle, 42);
    assert_eq!(req.from, 1024);
    assert_eq!(req.len, 512);
}

#[test]
fn parse_write_request() {
    let mut buf = [0u8; 28];
    buf[0..4].copy_from_slice(&NBD_REQUEST_MAGIC.to_be_bytes());
    buf[4..8].copy_from_slice(&1u32.to_be_bytes());
    buf[8..16].copy_from_slice(&100u64.to_be_bytes());
    buf[16..24].copy_from_slice(&(512u64).to_be_bytes());
    buf[24..28].copy_from_slice(&512u32.to_be_bytes());

    let req = NbdRequest::parse(&buf).unwrap();
    assert_eq!(req.command, NbdCommand::Write);
    assert_eq!(req.handle, 100);
    assert_eq!(req.from, 512);
}

#[test]
fn parse_disconnect_request() {
    let mut buf = [0u8; 28];
    buf[0..4].copy_from_slice(&NBD_REQUEST_MAGIC.to_be_bytes());
    buf[4..8].copy_from_slice(&2u32.to_be_bytes());
    buf[8..16].copy_from_slice(&0u64.to_be_bytes());
    buf[16..24].copy_from_slice(&0u64.to_be_bytes());
    buf[24..28].copy_from_slice(&0u32.to_be_bytes());

    let req = NbdRequest::parse(&buf).unwrap();
    assert_eq!(req.command, NbdCommand::Disconnect);
}

#[test]
fn parse_invalid_magic() {
    let buf = [0u8; 28];
    let result = NbdRequest::parse(&buf);
    assert!(result.is_none());
}

#[test]
fn build_reply_header() {
    let reply = file_access::nbd::build_reply(42, 0);
    assert_eq!(reply.len(), 16);
    let magic = u32::from_be_bytes(reply[0..4].try_into().unwrap());
    assert_eq!(magic, file_access::nbd::NBD_REPLY_MAGIC);
    let error = u32::from_be_bytes(reply[4..8].try_into().unwrap());
    assert_eq!(error, 0);
    let handle = u64::from_be_bytes(reply[8..16].try_into().unwrap());
    assert_eq!(handle, 42);
}

#[test]
fn build_error_reply() {
    let reply = file_access::nbd::build_reply(99, 5);
    let error = u32::from_be_bytes(reply[4..8].try_into().unwrap());
    assert_eq!(error, 5);
}
