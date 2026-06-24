use file_access::engine::FileAccessEngine;
use std::sync::Arc;

#[test]
fn engine_creation() {
    let tmp_storage = tempfile::NamedTempFile::new().unwrap();
    let tmp_audit = tempfile::NamedTempFile::new().unwrap();

    let storage = Arc::new(storage::Storage::open(tmp_storage.path()).unwrap());
    let audit_storage = Arc::new(storage::Storage::open(tmp_audit.path()).unwrap());
    let audit = log_audit::AuditService::new(audit_storage, tmp_audit.path());

    let engine = FileAccessEngine::new(storage, Arc::new(audit), "/dev/nbd0");

    assert!(!engine.is_mapped());
}
