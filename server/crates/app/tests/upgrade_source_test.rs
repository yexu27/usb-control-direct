use std::sync::Arc;

use storage::Storage;
use storage_test_support::TestDb;
use system_upgrade::{SystemVersion, UpgradeSourceReader};
use usb_control_app::upgrade_source::StorageUpgradeSourceReader;

#[test]
fn reads_committed_version_and_schema_on_every_call() {
    let db = TestDb::new();
    let storage = Arc::new(Storage::open(db.path()).unwrap());
    storage.config_set("system_version", "3.0.1").unwrap();
    let reader = StorageUpgradeSourceReader::new(Arc::clone(&storage));

    let first = reader.read().unwrap();
    assert_eq!(
        first.current_version,
        SystemVersion::parse("3.0.1").unwrap()
    );
    assert_eq!(first.current_schema, storage.schema_version().unwrap());

    storage.config_set("system_version", "3.0.2").unwrap();
    let second = reader.read().unwrap();
    assert_eq!(
        second.current_version,
        SystemVersion::parse("3.0.2").unwrap()
    );
}
