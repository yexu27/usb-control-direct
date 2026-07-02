use usb_identify::mount::{
    ensure_mount_available_from, mount_entries_from, planned_usb_raw_unmounts, MountEntry,
    MountPreflightError,
};

#[test]
fn parses_proc_mounts_entries() {
    let mounts = "\
/dev/sda2 /mnt/usb_raw/sda2 fuseblk rw,nosuid,nodev 0 0
/dev/root / ext4 rw,relatime 0 0
";

    let entries = mount_entries_from(mounts);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].source, "/dev/sda2");
    assert_eq!(entries[0].target, "/mnt/usb_raw/sda2");
    assert_eq!(entries[0].fs_type, "fuseblk");
}

#[test]
fn selects_only_usb_raw_mounts_for_recovery() {
    let entries = vec![
        MountEntry::new("/dev/sda2", "/mnt/usb_raw/sda2", "fuseblk"),
        MountEntry::new("/dev/root", "/", "ext4"),
        MountEntry::new("/dev/sdb1", "/media/user/DATA", "vfat"),
    ];

    let targets = planned_usb_raw_unmounts(&entries, "/mnt/usb_raw");

    assert_eq!(targets, vec!["/mnt/usb_raw/sda2".to_string()]);
}

#[test]
fn preflight_rejects_already_mounted_source_dev() {
    let entries = vec![MountEntry::new("/dev/sda2", "/mnt/usb_raw/sda2", "fuseblk")];

    let err = ensure_mount_available_from(&entries, "/dev/sda2", "/mnt/usb_raw/sda2")
        .unwrap_err();

    assert_eq!(
        err,
        MountPreflightError::SourceAlreadyMounted {
            source: "/dev/sda2".to_string(),
            target: "/mnt/usb_raw/sda2".to_string(),
        }
    );
}

#[test]
fn preflight_rejects_occupied_mount_point() {
    let entries = vec![MountEntry::new("/dev/sdb1", "/mnt/usb_raw/sda2", "vfat")];

    let err = ensure_mount_available_from(&entries, "/dev/sda2", "/mnt/usb_raw/sda2")
        .unwrap_err();

    assert_eq!(
        err,
        MountPreflightError::MountPointOccupied {
            source: "/dev/sdb1".to_string(),
            target: "/mnt/usb_raw/sda2".to_string(),
        }
    );
}
