//! 键盘拦截器测试。
//!
//! 验证 KeyboardChallenge 状态机与拦截器的集成行为：
//! 正常 1234 验证流程、错误序列重置、修饰键不干扰、拔出事件。

use hid_access::hid_report::{keycode_to_hid, KeyboardReport};
use hid_access::keyboard::{
    KeyboardChallenge, KeyboardEvent, KeyboardState, KeyboardTransitionResult,
};
use evdev::Key;

#[test]
fn test_1234_sequence_produces_correct_key_events() {
    assert!(matches!(keycode_to_hid(Key::KEY_1), Some((0, 0x1E))));
    assert!(matches!(keycode_to_hid(Key::KEY_2), Some((0, 0x1F))));
    assert!(matches!(keycode_to_hid(Key::KEY_3), Some((0, 0x20))));
    assert!(matches!(keycode_to_hid(Key::KEY_4), Some((0, 0x21))));
}

#[test]
fn test_modifier_keys_do_not_interrupt_sequence() {
    let mut ch = KeyboardChallenge::new();
    ch.transition(KeyboardEvent::GrabSuccess).unwrap();
    assert_eq!(ch.state(), KeyboardState::KbWaiting);

    // 按 1
    let result = ch.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    assert!(matches!(result, KeyboardTransitionResult::Unchanged));
    assert_eq!(ch.state(), KeyboardState::KbWaiting);

    // Shift 修饰键
    let result = ch.transition(KeyboardEvent::ModifierKey).unwrap();
    assert!(matches!(result, KeyboardTransitionResult::Unchanged));
    assert_eq!(ch.state(), KeyboardState::KbWaiting);

    // 按 2（修饰键不应清空缓冲区）
    let result = ch.transition(KeyboardEvent::KeyPress(0x1F)).unwrap();
    assert!(matches!(result, KeyboardTransitionResult::Unchanged));
    assert_eq!(ch.state(), KeyboardState::KbWaiting);

    // 完成 3 4
    ch.transition(KeyboardEvent::KeyPress(0x20)).unwrap();
    let result = ch.transition(KeyboardEvent::KeyPress(0x21)).unwrap();
    assert!(matches!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
    ));
}

#[test]
fn test_wrong_key_resets_sequence() {
    let mut ch = KeyboardChallenge::new();
    ch.transition(KeyboardEvent::GrabSuccess).unwrap();

    // 正确输入 1 2
    ch.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    ch.transition(KeyboardEvent::KeyPress(0x1F)).unwrap();

    // 输入错误键 5（不是 3）
    let result = ch.transition(KeyboardEvent::KeyPress(0x22)).unwrap();
    assert!(matches!(result, KeyboardTransitionResult::Unchanged));
    assert_eq!(ch.state(), KeyboardState::KbWaiting);

    // 重新输入完整 1234 应成功
    ch.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    ch.transition(KeyboardEvent::KeyPress(0x1F)).unwrap();
    ch.transition(KeyboardEvent::KeyPress(0x20)).unwrap();
    let result = ch.transition(KeyboardEvent::KeyPress(0x21)).unwrap();
    assert!(matches!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
    ));
}

#[test]
fn test_unplug_during_verification() {
    let mut ch = KeyboardChallenge::new();
    ch.transition(KeyboardEvent::GrabSuccess).unwrap();

    // 输入一半，拔出
    ch.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    let result = ch.transition(KeyboardEvent::Unplug).unwrap();
    assert!(matches!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
    ));
}

#[test]
fn test_keyboard_report_builds_correctly_with_modifiers() {
    let rpt = KeyboardReport {
        modifier: 0x02,
        keys: [0x04, 0, 0, 0, 0, 0],
    };
    let bytes = rpt.to_bytes();
    assert_eq!(bytes[0], 0x02);
    assert_eq!(bytes[2], 0x04);
    assert_eq!(bytes[3], 0);
}
