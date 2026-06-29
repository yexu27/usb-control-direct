use file_access::engine::FileAccessEngine;
use std::sync::Arc;
use storage_test_support::initialize_database;

#[test]
fn engine_creation() {
    let tmp_storage = tempfile::NamedTempFile::new().unwrap();
    initialize_database(tmp_storage.path());

    let storage = Arc::new(storage::Storage::open(tmp_storage.path()).unwrap());

    let engine = FileAccessEngine::new(storage, "/dev/nbd0");

    assert!(!engine.is_mapped());
}
