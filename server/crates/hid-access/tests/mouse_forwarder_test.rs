//! 鼠标转发器测试。
//!
//! 验证 MouseReport 构建、边界 clamp、button 位操作。

use hid_access::hid_report::{clamp_i8, MouseReport};

#[test]
fn test_mouse_report_builds() {
    let rpt = MouseReport {
        buttons: 0x03,
        dx: -10,
        dy: 5,
        wheel: -1,
    };
    let bytes = rpt.to_bytes();
    assert_eq!(bytes[0], 0x03);
    assert_eq!(bytes[1], (-10i8) as u8);
    assert_eq!(bytes[2], 5u8);
    assert_eq!(bytes[3], (-1i8) as u8);
}

#[test]
fn test_mouse_report_empty() {
    let rpt = MouseReport::empty();
    assert_eq!(rpt.to_bytes(), [0; 4]);
}

#[test]
fn test_clamp_dx_dy() {
    assert_eq!(clamp_i8(127), 127);
    assert_eq!(clamp_i8(-128), -128);
    assert_eq!(clamp_i8(300), 127);
    let dx: i32 = 500;
    assert_eq!(clamp_i8(dx), 127);
}

fn update_button(buttons: u8, bit: u8, value: i32) -> u8 {
    if value == 0 {
        buttons & !(1 << bit)
    } else {
        buttons | (1 << bit)
    }
}

#[test]
fn test_update_button_bits() {
    assert_eq!(update_button(0, 0, 1), 0x01);
    assert_eq!(update_button(0x01, 1, 1), 0x03);
    assert_eq!(update_button(0x03, 0, 0), 0x02);
    assert_eq!(update_button(0x02, 2, 1), 0x06);
}
