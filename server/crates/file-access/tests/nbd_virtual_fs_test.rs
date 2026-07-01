use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use file_access::nbd::{NbdBackend, NbdCommandHandler};

#[derive(Default)]
struct MockBackend {
    flush_count: AtomicUsize,
}

impl NbdBackend for MockBackend {
    fn read_at(&self, _offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        Ok(vec![0u8; len])
    }

    fn write_at(&self, _offset: u64, _data: &[u8]) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn flush(&self) -> Result<(), std::io::Error> {
        self.flush_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn nbd_flush_calls_backend_flush() {
    let backend = Arc::new(MockBackend::default());
    let handler = NbdCommandHandler::new(Arc::clone(&backend));
    handler.handle_flush().unwrap();
    assert_eq!(backend.flush_count.load(Ordering::SeqCst), 1);
}

#[test]
fn nbd_disconnect_calls_backend_flush() {
    let backend = Arc::new(MockBackend::default());
    let handler = NbdCommandHandler::new(Arc::clone(&backend));
    handler.handle_disconnect().unwrap();
    assert_eq!(backend.flush_count.load(Ordering::SeqCst), 1);
}
