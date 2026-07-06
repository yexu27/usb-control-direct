use file_access::raw_mount::{
    ensure_mount_available_from, mount_entries_from, mount_path_for, mount_target_exists_from,
    planned_usb_raw_unmounts,
};

#[test]
fn parses_mount_table_entries() {
    let entries = mount_entries_from(
        "/dev/sda1 /mnt/usb_raw/sda1 vfat rw 0 0\n\
         /dev/root / ext4 rw 0 0\n",
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].source, "/dev/sda1");
    assert_eq!(entries[0].target, "/mnt/usb_raw/sda1");
    assert_eq!(entries[0].fs_type, "vfat");
}

#[test]
fn selects_only_usb_raw_mounts_for_recovery() {
    let entries = mount_entries_from(
        "/dev/sda1 /mnt/usb_raw/sda1 vfat rw 0 0\n\
         /dev/sdb1 /media/user/disk vfat rw 0 0\n\
         /dev/root / ext4 rw 0 0\n",
    );

    let planned = planned_usb_raw_unmounts(&entries, "/mnt/usb_raw");
    assert_eq!(planned, vec!["/mnt/usb_raw/sda1"]);
}

#[test]
fn detects_existing_mount_target() {
    let entries = mount_entries_from("/dev/sda1 /mnt/usb_raw/sda1 vfat rw 0 0\n");
    assert!(mount_target_exists_from(&entries, "/mnt/usb_raw/sda1"));
    assert!(!mount_target_exists_from(&entries, "/mnt/usb_raw/sda2"));
}

#[test]
fn rejects_mount_point_used_by_different_source() {
    let entries = mount_entries_from("/dev/sdb1 /mnt/usb_raw/sda1 vfat rw 0 0\n");
    let err = ensure_mount_available_from(&entries, "/dev/sda1", "/mnt/usb_raw/sda1")
        .expect_err("mount point used by another source must fail");

    assert!(err.to_string().contains("挂载点已被其它设备占用"));
}

#[test]
fn builds_stable_mount_path_from_device_name() {
    assert_eq!(
        mount_path_for("sda1").to_string_lossy(),
        "/mnt/usb_raw/sda1"
    );
}
