use common::types::KeyboardState;
use hid_access::keyboard::{KeyboardChallenge, KeyboardEvent, KeyboardTransitionResult};
use hid_access::HidAccessError;

#[test]
fn grab_success_transitions_to_waiting() {
    let mut kb = KeyboardChallenge::new();
    assert_eq!(kb.state(), KeyboardState::KbDetected);

    let result = kb.transition(KeyboardEvent::GrabSuccess).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbWaiting)
    );
    assert_eq!(kb.state(), KeyboardState::KbWaiting);
}

#[test]
fn grab_failed_transitions_to_rejected() {
    let mut kb = KeyboardChallenge::new();
    let result = kb.transition(KeyboardEvent::GrabFailed).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
}

#[test]
fn correct_1234_transitions_to_mapped() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();

    for &ch in b"123" {
        let r = kb.transition(KeyboardEvent::KeyPress(ch)).unwrap();
        assert_eq!(r, KeyboardTransitionResult::Unchanged);
    }

    let result = kb.transition(KeyboardEvent::KeyPress(b'4')).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
    );
}

#[test]
fn wrong_key_clears_buffer() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();

    kb.transition(KeyboardEvent::KeyPress(b'1')).unwrap();
    kb.transition(KeyboardEvent::KeyPress(b'2')).unwrap();
    let r = kb.transition(KeyboardEvent::KeyPress(b'9')).unwrap();
    assert_eq!(r, KeyboardTransitionResult::Unchanged);
    assert_eq!(kb.state(), KeyboardState::KbWaiting);

    // 重新输入 1234 仍可成功
    for &ch in b"1234" {
        kb.transition(KeyboardEvent::KeyPress(ch)).unwrap();
    }
    assert_eq!(kb.state(), KeyboardState::KbMapped);
}

#[test]
fn modifier_key_ignored_during_challenge() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();

    kb.transition(KeyboardEvent::KeyPress(b'1')).unwrap();
    let r = kb.transition(KeyboardEvent::ModifierKey).unwrap();
    assert_eq!(r, KeyboardTransitionResult::Unchanged);

    for &ch in b"234" {
        kb.transition(KeyboardEvent::KeyPress(ch)).unwrap();
    }
    assert_eq!(kb.state(), KeyboardState::KbMapped);
}

#[test]
fn unplug_from_detected() {
    let mut kb = KeyboardChallenge::new();
    let result = kb.transition(KeyboardEvent::Unplug).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
    );
}

#[test]
fn unplug_from_waiting() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();
    let result = kb.transition(KeyboardEvent::Unplug).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
    );
}

#[test]
fn unplug_from_mapped() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();
    for &ch in b"1234" {
        kb.transition(KeyboardEvent::KeyPress(ch)).unwrap();
    }
    let result = kb.transition(KeyboardEvent::Unplug).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
    );
}

#[test]
fn invalid_transition_grab_in_waiting() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();
    let result = kb.transition(KeyboardEvent::GrabSuccess);
    assert!(result.is_err());
    assert_eq!(kb.state(), KeyboardState::KbWaiting);
}

#[test]
fn no_timeout_stays_waiting() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();
    assert_eq!(kb.state(), KeyboardState::KbWaiting);
    // 没有超时事件，状态保持 KbWaiting
}
