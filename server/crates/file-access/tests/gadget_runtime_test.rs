use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use file_access::gadget::GadgetRuntime;

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(unix)]
fn make_lun(root: &Path, function_name: &str, linked: bool) -> PathBuf {
    let gadget = root.join("rockchip");
    let function = gadget.join("functions").join(function_name);
    let lun = function.join("lun.0");
    fs::create_dir_all(&lun).unwrap();
    fs::write(lun.join("file"), "\n").unwrap();
    fs::write(lun.join("ro"), "1\n").unwrap();
    fs::write(lun.join("removable"), "1\n").unwrap();
    fs::write(lun.join("nofua"), "1\n").unwrap();
    fs::write(lun.join("cdrom"), "0\n").unwrap();
    fs::create_dir_all(gadget.join("configs/b.1")).unwrap();
    fs::create_dir_all(gadget.join("strings/0x409")).unwrap();
    fs::write(gadget.join("UDC"), "fcc00000.dwc3\n").unwrap();
    if linked {
        symlink(
            format!("../../functions/{function_name}"),
            gadget.join("configs/b.1").join(function_name),
        )
        .unwrap();
    }
    lun
}

#[cfg(unix)]
#[test]
fn discover_under_prefers_lun_linked_to_active_config() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    make_lun(root, "mass_storage.zzz", false);
    let linked = make_lun(root, "mass_storage.usb0", true);

    let runtime = GadgetRuntime::discover_under(root).unwrap();

    assert_eq!(runtime.lun_dir(), linked.as_path());
    assert_eq!(runtime.gadget_name(), "rockchip");
    assert_eq!(runtime.function_name(), "mass_storage.usb0");
}

#[cfg(unix)]
#[test]
fn attach_sets_lun_attributes_and_backing_path() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let lun = make_lun(root, "mass_storage.usb0", true);
    let backing = dir.path().join("nbd3");
    fs::write(&backing, "").unwrap();

    let runtime = GadgetRuntime::discover_under(root).unwrap();
    runtime.attach_mass_storage(&backing, false).unwrap();

    assert_eq!(fs::read_to_string(lun.join("ro")).unwrap(), "0\n");
    assert_eq!(fs::read_to_string(lun.join("removable")).unwrap(), "1\n");
    assert_eq!(fs::read_to_string(lun.join("nofua")).unwrap(), "1\n");
    assert_eq!(fs::read_to_string(lun.join("cdrom")).unwrap(), "0\n");
    assert_eq!(
        fs::read_to_string(lun.join("file")).unwrap(),
        backing.display().to_string()
    );
    assert_eq!(
        runtime.current_backing().unwrap(),
        backing.display().to_string()
    );
}

#[cfg(unix)]
#[test]
fn attach_binds_available_udc_when_current_binding_is_empty() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let lun = make_lun(root, "mass_storage.usb0", true);
    let backing = dir.path().join("nbd3");
    let class_udc = dir.path().join("class_udc");
    fs::write(root.join("rockchip").join("UDC"), "\n").unwrap();
    fs::write(&backing, "").unwrap();
    fs::create_dir_all(class_udc.join("fcc00000.dwc3")).unwrap();

    let runtime = GadgetRuntime::discover_under_with_udc_root(root, &class_udc).unwrap();
    runtime.attach_mass_storage(&backing, true).unwrap();

    assert_eq!(
        fs::read_to_string(root.join("rockchip").join("UDC")).unwrap(),
        "fcc00000.dwc3\n"
    );
    assert_eq!(
        fs::read_to_string(lun.join("file")).unwrap(),
        backing.display().to_string()
    );
}

#[cfg(unix)]
#[test]
fn detach_clears_backing_file() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let lun = make_lun(root, "mass_storage.usb0", true);
    fs::write(lun.join("file"), "/dev/nbd3\n").unwrap();

    let runtime = GadgetRuntime::discover_under(root).unwrap();
    runtime.detach_mass_storage().unwrap();

    assert_eq!(fs::read_to_string(lun.join("file")).unwrap(), "\n");
    assert_eq!(runtime.current_backing().unwrap(), "");
}

#[cfg(unix)]
#[test]
fn bind_udc_if_empty_binds_available_udc() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    make_lun(root, "mass_storage.usb0", true);
    let class_udc = dir.path().join("class_udc");
    fs::create_dir_all(class_udc.join("fcc00000.dwc3")).unwrap();
    fs::write(root.join("rockchip").join("UDC"), "\n").unwrap();

    let runtime = GadgetRuntime::discover_under(root).unwrap();
    runtime.bind_udc_if_empty_under(&class_udc).unwrap();

    assert_eq!(
        fs::read_to_string(root.join("rockchip").join("UDC")).unwrap(),
        "fcc00000.dwc3\n"
    );
}

#[cfg(unix)]
#[test]
fn bind_udc_if_empty_keeps_existing_binding() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    make_lun(root, "mass_storage.usb0", true);
    let class_udc = dir.path().join("class_udc");
    fs::create_dir_all(class_udc.join("fcc00000.dwc3")).unwrap();
    fs::write(root.join("rockchip").join("UDC"), "already-bound\n").unwrap();

    let runtime = GadgetRuntime::discover_under(root).unwrap();
    runtime.bind_udc_if_empty_under(&class_udc).unwrap();

    assert_eq!(
        fs::read_to_string(root.join("rockchip").join("UDC")).unwrap(),
        "already-bound\n"
    );
}
