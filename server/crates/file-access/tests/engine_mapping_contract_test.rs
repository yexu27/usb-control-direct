use usb_identify::traits::MappedSession;

#[test]
fn mapped_session_records_nbd_device_path() {
    let session = MappedSession {
        id: "s04_test".to_string(),
        mount_path: "/mnt/usb_raw/sda1".to_string(),
        nbd_device: "/dev/nbd3".to_string(),
    };

    assert_eq!(session.nbd_device, "/dev/nbd3");
}
