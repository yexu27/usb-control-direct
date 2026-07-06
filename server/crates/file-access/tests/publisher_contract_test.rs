use std::path::Path;

use file_access::publisher::nbd_device_path;

#[test]
fn nbd_device_path_uses_whole_disk_device() {
    assert_eq!(nbd_device_path(3), Path::new("/dev/nbd3"));
}
