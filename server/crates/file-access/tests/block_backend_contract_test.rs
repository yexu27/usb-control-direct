use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use file_access::block_backend::BlockBackend;

#[derive(Default)]
struct MockBlockBackend {
    flush_count: AtomicUsize,
    shutdown_called: AtomicBool,
}

impl BlockBackend for MockBlockBackend {
    fn read_at(&self, _offset: u64, len: usize) -> io::Result<Vec<u8>> {
        Ok(vec![0u8; len])
    }

    fn write_at(&self, _offset: u64, _data: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        self.flush_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn shutdown(&self) -> io::Result<()> {
        self.shutdown_called.store(true, Ordering::SeqCst);
        self.flush()
    }
}

#[test]
fn block_backend_shutdown_is_explicit_contract() {
    let backend = MockBlockBackend::default();

    backend.shutdown().unwrap();

    assert_eq!(backend.flush_count.load(Ordering::SeqCst), 1);
    assert!(backend.shutdown_called.load(Ordering::SeqCst));
}
