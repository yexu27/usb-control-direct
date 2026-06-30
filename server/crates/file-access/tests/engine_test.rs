use file_access::engine::FileAccessEngine;
use file_access::gadget::{GadgetRuntime, MassStorageHandle};
use std::path::Path;
use std::sync::Arc;
use storage_test_support::initialize_database;

fn test_gadget(root: &Path) -> GadgetRuntime {
    let gadget_dir = root.join("rockchip");
    let lun_dir = gadget_dir.join("functions/mass_storage.usb0/lun.0");
    std::fs::create_dir_all(&lun_dir).unwrap();
    GadgetRuntime::from_handle(
        MassStorageHandle::new(
            "rockchip".into(),
            "mass_storage.usb0".into(),
            gadget_dir,
            lun_dir,
        ),
        root.join("class_udc"),
    )
}

#[test]
fn engine_creation() {
    let tmp_storage = tempfile::NamedTempFile::new().unwrap();
    let tmp_gadget = tempfile::tempdir().unwrap();
    initialize_database(tmp_storage.path());

    let storage = Arc::new(storage::Storage::open(tmp_storage.path()).unwrap());

    let engine = FileAccessEngine::new(storage, test_gadget(tmp_gadget.path()));

    assert!(!engine.is_mapped());
}
