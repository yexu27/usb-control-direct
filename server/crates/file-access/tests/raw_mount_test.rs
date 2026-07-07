use file_access::raw_mount::{
    ensure_mount_available_from, mount_entries_from, mount_path_for, mount_target_exists_from,
    planned_project_raw_unmounts,
};

#[test]
fn parses_mount_table_entries() {
    let entries = mount_entries_from(
        "/dev/sda1 /mnt/usb-control/raw/storage__one vfat rw 0 0\n\
         /dev/root / ext4 rw 0 0\n",
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].source, "/dev/sda1");
    assert_eq!(entries[0].target, "/mnt/usb-control/raw/storage__one");
    assert_eq!(entries[0].fs_type, "vfat");
}

#[test]
fn selects_only_project_raw_mounts_for_recovery() {
    let entries = mount_entries_from(
        "/dev/sda1 /mnt/usb-control/raw/storage__one vfat rw 0 0\n\
         /dev/sdb1 /mnt/usb_raw/sdb1 vfat rw 0 0\n\
         /dev/sdc1 /media/user/disk vfat rw 0 0\n\
         /dev/root / ext4 rw 0 0\n",
    );

    let planned = planned_project_raw_unmounts(&entries, "/mnt/usb-control/raw");
    assert_eq!(planned, vec!["/mnt/usb-control/raw/storage__one"]);
}

#[test]
fn detects_existing_mount_target() {
    let entries =
        mount_entries_from("/dev/sda1 /mnt/usb-control/raw/storage__one vfat rw 0 0\n");
    assert!(mount_target_exists_from(
        &entries,
        "/mnt/usb-control/raw/storage__one"
    ));
    assert!(!mount_target_exists_from(
        &entries,
        "/mnt/usb-control/raw/storage__two"
    ));
}

#[test]
fn rejects_mount_point_used_by_different_source() {
    let entries =
        mount_entries_from("/dev/sdb1 /mnt/usb-control/raw/storage__one vfat rw 0 0\n");
    let err = ensure_mount_available_from(
        &entries,
        "/dev/sda1",
        "/mnt/usb-control/raw/storage__one",
    )
    .expect_err("mount point used by another source must fail");

    assert!(err.to_string().contains("挂载点已被其它设备占用"));
}

#[test]
fn builds_mount_path_from_storage_session_id() {
    assert_eq!(
        mount_path_for("storage_7b0f5d7c6a3e2c10").to_string_lossy(),
        "/mnt/usb-control/raw/storage_7b0f5d7c6a3e2c10"
    );
}

#[test]
fn mount_path_is_not_derived_from_block_device_name() {
    assert_ne!(
        mount_path_for("storage_7b0f5d7c6a3e2c10").to_string_lossy(),
        "/mnt/usb-control/raw/sda2"
    );
}
