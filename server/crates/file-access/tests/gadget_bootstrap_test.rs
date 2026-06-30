use std::path::Path;

use file_access::gadget_bootstrap::{GadgetBootstrap, GadgetBootstrapConfig};

fn base_config(root: &Path, udc_root: &Path) -> GadgetBootstrapConfig {
    GadgetBootstrapConfig {
        configfs_root: root.to_path_buf(),
        udc_root: udc_root.to_path_buf(),
        gadget_name: "rockchip".into(),
        config_name: "b.1".into(),
        udc: Some("fcc00000.dwc3".into()),
        keep_adb: false,
        storage_function: "mass_storage.usb0".into(),
        storage_lun: 0,
        keyboard_function: "hid.keyboard".into(),
        mouse_function: "hid.mouse".into(),
    }
}

#[cfg(unix)]
#[test]
fn bootstrap_creates_business_functions_and_leaves_lun_empty() {
    let dir = tempfile::tempdir().unwrap();
    let udc = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(udc.path().join("fcc00000.dwc3")).unwrap();

    let runtime = GadgetBootstrap::prepare(base_config(dir.path(), udc.path())).unwrap();

    let gadget = dir.path().join("rockchip");
    let lun = gadget.join("functions/mass_storage.usb0/lun.0");
    assert!(lun.is_dir());
    assert_eq!(
        std::fs::read_to_string(lun.join("file")).unwrap().trim(),
        ""
    );
    assert_eq!(
        std::fs::read_to_string(lun.join("removable"))
            .unwrap()
            .trim(),
        "1"
    );
    assert_eq!(std::fs::read_to_string(lun.join("ro")).unwrap().trim(), "0");
    assert!(gadget.join("functions/hid.keyboard").is_dir());
    assert!(gadget.join("functions/hid.mouse").is_dir());
    assert!(gadget.join("configs/b.1/mass_storage.usb0").exists());
    assert!(gadget.join("configs/b.1/hid.keyboard").exists());
    assert!(gadget.join("configs/b.1/hid.mouse").exists());
    assert_eq!(runtime.mass_storage().function_name(), "mass_storage.usb0");
    assert_eq!(
        std::fs::read_to_string(gadget.join("UDC")).unwrap(),
        "fcc00000.dwc3\n"
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_removes_adb_link_when_keep_adb_is_false() {
    let dir = tempfile::tempdir().unwrap();
    let udc = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(udc.path().join("fcc00000.dwc3")).unwrap();
    let cfg_dir = dir.path().join("rockchip/configs/b.1");
    let adb_dir = dir.path().join("rockchip/functions/ffs.adb");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::create_dir_all(&adb_dir).unwrap();
    std::os::unix::fs::symlink(&adb_dir, cfg_dir.join("f-ffs.adb")).unwrap();

    GadgetBootstrap::prepare(base_config(dir.path(), udc.path())).unwrap();

    assert!(!cfg_dir.join("f-ffs.adb").exists());
}

#[cfg(unix)]
#[test]
fn bootstrap_clears_stale_lun_backing() {
    let dir = tempfile::tempdir().unwrap();
    let udc = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(udc.path().join("fcc00000.dwc3")).unwrap();
    let lun = dir
        .path()
        .join("rockchip/functions/mass_storage.usb0/lun.0");
    std::fs::create_dir_all(&lun).unwrap();
    std::fs::write(lun.join("file"), "/dev/nbd3\n").unwrap();

    GadgetBootstrap::prepare(base_config(dir.path(), udc.path())).unwrap();

    assert_eq!(
        std::fs::read_to_string(lun.join("file")).unwrap().trim(),
        ""
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let udc = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(udc.path().join("fcc00000.dwc3")).unwrap();
    let cfg = base_config(dir.path(), udc.path());

    GadgetBootstrap::prepare(cfg.clone()).unwrap();
    GadgetBootstrap::prepare(cfg).unwrap();

    let gadget = dir.path().join("rockchip");
    assert!(gadget.join("functions/mass_storage.usb0/lun.0").is_dir());
    assert!(gadget.join("functions/hid.keyboard").is_dir());
    assert!(gadget.join("functions/hid.mouse").is_dir());
    assert_eq!(
        std::fs::read_to_string(gadget.join("UDC")).unwrap(),
        "fcc00000.dwc3\n"
    );
}
