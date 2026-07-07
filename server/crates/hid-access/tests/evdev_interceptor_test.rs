use evdev::Key;
use hid_access::hid_report::{keycode_to_hid, KeyboardReport};
use hid_access::keyboard::{
    KeyboardChallenge, KeyboardEvent, KeyboardState, KeyboardTransitionResult,
};

#[test]
fn test_1234_sequence_produces_correct_key_events() {
    assert!(matches!(keycode_to_hid(Key::KEY_1), Some((0, 0x1E))));
    assert!(matches!(keycode_to_hid(Key::KEY_2), Some((0, 0x1F))));
    assert!(matches!(keycode_to_hid(Key::KEY_3), Some((0, 0x20))));
    assert!(matches!(keycode_to_hid(Key::KEY_4), Some((0, 0x21))));
}

#[test]
fn test_wrong_key_rejects_instead_of_resetting_sequence() {
    let mut ch = KeyboardChallenge::new();
    ch.transition(KeyboardEvent::GrabSuccess).unwrap();

    ch.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    ch.transition(KeyboardEvent::KeyPress(0x1F)).unwrap();

    let result = ch.transition(KeyboardEvent::KeyPress(0x22)).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    assert_eq!(ch.state(), KeyboardState::KbRejected);

    let later_correct_input = ch.transition(KeyboardEvent::KeyPress(0x1E));
    assert!(later_correct_input.is_err());
    assert_eq!(ch.state(), KeyboardState::KbRejected);
}

#[test]
fn test_modifier_key_rejects_during_verification() {
    let mut ch = KeyboardChallenge::new();
    ch.transition(KeyboardEvent::GrabSuccess).unwrap();

    let result = ch.transition(KeyboardEvent::ModifierKey).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    assert_eq!(ch.state(), KeyboardState::KbRejected);
}

#[test]
fn test_unplug_during_verification() {
    let mut ch = KeyboardChallenge::new();
    ch.transition(KeyboardEvent::GrabSuccess).unwrap();

    ch.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    let result = ch.transition(KeyboardEvent::Unplug).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
    );
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
