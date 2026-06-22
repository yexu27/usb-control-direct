//! HID report 结构体测试。
//!
//! 验证 KeyboardReport/MouseReport 序列化、keycode 映射、clamp 边界。

use hid_access::hid_report::{clamp_i8, keycode_to_hid, KeyboardReport, MouseReport};
use evdev::Key;

#[test]
fn keyboard_report_empty() {
    let rpt = KeyboardReport::empty();
    assert_eq!(rpt.to_bytes(), [0; 8]);
}

#[test]
fn keyboard_report_to_bytes() {
    let rpt = KeyboardReport {
        modifier: 0x02,
        keys: [0x04, 0x05, 0x06, 0, 0, 0],
    };
    let bytes = rpt.to_bytes();
    assert_eq!(bytes, [0x02, 0, 0x04, 0x05, 0x06, 0, 0, 0]);
}

#[test]
fn keyboard_report_max_6_keys() {
    let rpt = KeyboardReport {
        modifier: 0,
        keys: [0x04, 0x05, 0x06, 0x07, 0x08, 0x09],
    };
    let bytes = rpt.to_bytes();
    assert_eq!(bytes, [0, 0, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]);
}

#[test]
fn mouse_report_empty() {
    let rpt = MouseReport::empty();
    assert_eq!(rpt.to_bytes(), [0, 0, 0, 0]);
}

#[test]
fn mouse_report_to_bytes() {
    let rpt = MouseReport {
        buttons: 0x03,
        dx: -5,
        dy: 10,
        wheel: -1,
    };
    let bytes = rpt.to_bytes();
    assert_eq!(bytes, [0x03, (-5i8) as u8, 10u8, (-1i8) as u8]);
}

#[test]
fn keycode_1234_maps_correctly() {
    assert_eq!(keycode_to_hid(Key::KEY_1), Some((0, 0x1E)));
    assert_eq!(keycode_to_hid(Key::KEY_2), Some((0, 0x1F)));
    assert_eq!(keycode_to_hid(Key::KEY_3), Some((0, 0x20)));
    assert_eq!(keycode_to_hid(Key::KEY_4), Some((0, 0x21)));
}

#[test]
fn keycode_modifiers_map_correctly() {
    assert_eq!(keycode_to_hid(Key::KEY_LEFTCTRL), Some((0x01, 0)));
    assert_eq!(keycode_to_hid(Key::KEY_LEFTSHIFT), Some((0x02, 0)));
    assert_eq!(keycode_to_hid(Key::KEY_LEFTALT), Some((0x04, 0)));
}

#[test]
fn keycode_unknown_returns_none() {
    assert_eq!(keycode_to_hid(Key::KEY_PLAYCD), None);
}

#[test]
fn clamp_i8_bounds() {
    assert_eq!(clamp_i8(0), 0);
    assert_eq!(clamp_i8(127), 127);
    assert_eq!(clamp_i8(-128), -128);
    assert_eq!(clamp_i8(128), 127);
    assert_eq!(clamp_i8(-200), -128);
}
