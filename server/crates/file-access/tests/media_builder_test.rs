use std::fs;
use std::sync::Arc;

use file_access::media_builder::VirtualMediaBuilder;
use storage::Storage;
use storage_test_support::initialize_database;
use tempfile::tempdir;
use usb_identify::traits::ScanResult;

#[test]
fn build_media_uses_scan_result_and_source_capacity() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("clean.txt"), b"clean").unwrap();
    fs::write(tmp.path().join("bad.txt"), b"virus").unwrap();

    let db = tempfile::NamedTempFile::new().unwrap();
    initialize_database(db.path());
    let storage = Arc::new(Storage::open(db.path()).unwrap());
    let builder = VirtualMediaBuilder::new(storage);

    let media = builder
        .build(
            tmp.path(),
            ScanResult {
                is_clean: false,
                infected_files: vec!["bad.txt".to_string()],
            },
            0,
            16 * 1024 * 1024,
        )
        .unwrap();

    assert!(media.total_sectors() > 0);
}
