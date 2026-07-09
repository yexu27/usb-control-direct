use std::io::{Read, Write};
use std::os::fd::IntoRawFd;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread;

use file_access::block_backend::{BlockBackend, BlockWriteOutcome};
use file_access::nbd::protocol::{NBD_REPLY_MAGIC, NBD_REQUEST_MAGIC};

fn write_request(handle: u64, from: u64, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&NBD_REQUEST_MAGIC.to_be_bytes());
    out.extend_from_slice(&1_u32.to_be_bytes());
    out.extend_from_slice(&handle.to_be_bytes());
    out.extend_from_slice(&from.to_be_bytes());
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(data);
    out
}

fn read_reply(stream: &mut UnixStream) -> (u32, u64) {
    let mut reply = [0_u8; 16];
    stream.read_exact(&mut reply).unwrap();
    assert_eq!(
        u32::from_be_bytes(reply[0..4].try_into().unwrap()),
        NBD_REPLY_MAGIC
    );
    (
        u32::from_be_bytes(reply[4..8].try_into().unwrap()),
        u64::from_be_bytes(reply[8..16].try_into().unwrap()),
    )
}

struct OutcomeBackend {
    outcome: BlockWriteOutcome,
}

impl BlockBackend for OutcomeBackend {
    fn read_at(&self, _offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        Ok(vec![0_u8; len])
    }

    fn write_at(&self, _offset: u64, _data: &[u8]) -> std::io::Result<BlockWriteOutcome> {
        Ok(self.outcome.clone())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

struct ErrorBackend;

impl BlockBackend for ErrorBackend {
    fn read_at(&self, _offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        Ok(vec![0_u8; len])
    }

    fn write_at(&self, _offset: u64, _data: &[u8]) -> std::io::Result<BlockWriteOutcome> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "real backend error",
        ))
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn nbd_write_returns_success_for_policy_rejected_and_restored() {
    let (mut client, server) = UnixStream::pair().unwrap();
    let backend = Arc::new(OutcomeBackend {
        outcome: BlockWriteOutcome::PolicyRejectedAndRestored {
            reason: "blocked placeholder rename restored".to_string(),
        },
    });

    let join = thread::spawn(move || {
        file_access::nbd::request_loop::run_request_loop(server.into_raw_fd(), backend);
    });

    client
        .write_all(&write_request(7, 4096, &[1, 2, 3, 4]))
        .unwrap();
    let (error, handle) = read_reply(&mut client);
    assert_eq!(error, 0);
    assert_eq!(handle, 7);
    drop(client);
    join.join().unwrap();
}

#[test]
fn nbd_write_returns_eio_for_real_backend_error() {
    let (mut client, server) = UnixStream::pair().unwrap();
    let backend = Arc::new(ErrorBackend);

    let join = thread::spawn(move || {
        file_access::nbd::request_loop::run_request_loop(server.into_raw_fd(), backend);
    });

    client
        .write_all(&write_request(8, 4096, &[1, 2, 3, 4]))
        .unwrap();
    let (error, handle) = read_reply(&mut client);
    assert_ne!(error, 0);
    assert_eq!(handle, 8);
    drop(client);
    join.join().unwrap();
}
