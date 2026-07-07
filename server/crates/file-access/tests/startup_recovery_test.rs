use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use file_access::raw_mount::MountOperations;
use file_access::startup_recovery::{
    clear_lun_backing_file, recover_raw_mounts_under, StartupRecoveryReport,
};

#[derive(Default)]
struct FakeMountOps {
    umounted: Arc<Mutex<Vec<String>>>,
}

impl MountOperations for FakeMountOps {
    fn is_mounted(&self, _dev_path: &str) -> Result<bool, file_access::FileAccessError> {
        Ok(false)
    }

    fn mount(
        &self,
        _dev_path: &str,
        _mount_point: &str,
        _fs_type: &str,
    ) -> Result<(), file_access::FileAccessError> {
        Ok(())
    }

    fn umount(&self, mount_point: &str) -> Result<(), file_access::FileAccessError> {
        self.umounted.lock().unwrap().push(mount_point.to_string());
        Ok(())
    }

    fn detect_fs_type(&self, _dev_path: &str) -> Result<String, file_access::FileAccessError> {
        Ok("vfat".into())
    }
}

#[test]
fn clears_lun_backing_file() {
    let dir = tempfile::tempdir().unwrap();
    let lun_file = dir.path().join("file");
    fs::write(&lun_file, "/dev/nbd3\n").unwrap();

    clear_lun_backing_file(&lun_file).unwrap();

    assert_eq!(fs::read_to_string(&lun_file).unwrap(), "\n");
}

#[test]
fn recovers_only_project_raw_mounts_from_mount_table_text() {
    let ops = FakeMountOps::default();
    let report = recover_raw_mounts_under(
        &ops,
        Path::new("/mnt/usb-control/raw"),
        "/dev/sda1 /mnt/usb-control/raw/storage__one vfat rw 0 0\n\
         /dev/sdb1 /media/legacy_raw/sdb1 vfat rw 0 0\n\
         /dev/sdc1 /media/user/disk vfat rw 0 0\n",
    )
    .unwrap();

    assert_eq!(
        ops.umounted.lock().unwrap().as_slice(),
        ["/mnt/usb-control/raw/storage__one"]
    );
    assert_eq!(
        report,
        StartupRecoveryReport {
            cleared_lun: false,
            disconnected_nbd: 0,
            recovered_mounts: 1,
        }
    );
}
