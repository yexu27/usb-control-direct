use usb_identify::traits::{MapContext, MappedSession, ScanResult};

#[test]
fn mapped_session_records_nbd_device_path() {
    let session = MappedSession {
        id: "s04_test".to_string(),
        mount_path: "/mnt/usb_raw/sda1".to_string(),
        nbd_device: "/dev/nbd3".to_string(),
    };

    assert_eq!(session.nbd_device, "/dev/nbd3");
}

#[test]
fn map_context_records_source_partition_size() {
    let ctx = MapContext {
        mount_path: "/mnt/usb_raw/sdc2".to_string(),
        scan_result: ScanResult {
            is_clean: true,
            infected_files: vec![],
        },
        permission: 0,
        source_size_bytes: 64 * 1024 * 1024,
        nbd_device: "/dev/nbd3".to_string(),
    };

    assert_eq!(ctx.source_size_bytes, 64 * 1024 * 1024);
}
