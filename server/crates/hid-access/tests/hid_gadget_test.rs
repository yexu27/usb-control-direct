use std::fs;

use tempfile::tempdir;

use hid_access::hid_gadget::{
    ensure_hid_functions_under, KEYBOARD_FUNCTION, MOUSE_FUNCTION,
};
use hid_access::hid_report::{
    KEYBOARD_REPORT_DESC, KEYBOARD_REPORT_LEN, MOUSE_REPORT_DESC, MOUSE_REPORT_LEN,
};

#[test]
fn ensure_hid_functions_creates_keyboard_and_mouse_functions() {
    let dir = tempdir().unwrap();
    let gadget_dir = dir.path().join("rockchip");

    ensure_hid_functions_under(&gadget_dir).unwrap();

    let keyboard = gadget_dir.join("functions").join(KEYBOARD_FUNCTION);
    let mouse = gadget_dir.join("functions").join(MOUSE_FUNCTION);

    assert_eq!(fs::read_to_string(keyboard.join("protocol")).unwrap(), "1");
    assert_eq!(fs::read_to_string(keyboard.join("subclass")).unwrap(), "1");
    assert_eq!(
        fs::read_to_string(keyboard.join("report_length")).unwrap(),
        KEYBOARD_REPORT_LEN.to_string()
    );
    assert_eq!(fs::read(keyboard.join("report_desc")).unwrap(), KEYBOARD_REPORT_DESC);

    assert_eq!(fs::read_to_string(mouse.join("protocol")).unwrap(), "2");
    assert_eq!(fs::read_to_string(mouse.join("subclass")).unwrap(), "1");
    assert_eq!(
        fs::read_to_string(mouse.join("report_length")).unwrap(),
        MOUSE_REPORT_LEN.to_string()
    );
    assert_eq!(fs::read(mouse.join("report_desc")).unwrap(), MOUSE_REPORT_DESC);
}

#[test]
fn ensure_hid_functions_is_idempotent() {
    let dir = tempdir().unwrap();
    let gadget_dir = dir.path().join("rockchip");

    ensure_hid_functions_under(&gadget_dir).unwrap();
    ensure_hid_functions_under(&gadget_dir).unwrap();

    assert!(gadget_dir.join("functions").join(KEYBOARD_FUNCTION).is_dir());
    assert!(gadget_dir.join("functions").join(MOUSE_FUNCTION).is_dir());
}
