use hid_access::keyboard::{KeyboardChallenge, KeyboardEvent, KeyboardState, KeyboardTransitionResult};

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

    for &ch in &[0x1E, 0x1F, 0x20] {
        let r = kb.transition(KeyboardEvent::KeyPress(ch)).unwrap();
        assert_eq!(r, KeyboardTransitionResult::Unchanged);
    }

    let result = kb.transition(KeyboardEvent::KeyPress(0x21)).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
    );
}

#[test]
fn wrong_key_clears_buffer() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();

    kb.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    kb.transition(KeyboardEvent::KeyPress(0x1F)).unwrap();
    let r = kb.transition(KeyboardEvent::KeyPress(0x26)).unwrap();
    assert_eq!(r, KeyboardTransitionResult::Unchanged);
    assert_eq!(kb.state(), KeyboardState::KbWaiting);

    for &ch in &[0x1E, 0x1F, 0x20, 0x21] {
        kb.transition(KeyboardEvent::KeyPress(ch)).unwrap();
    }
    assert_eq!(kb.state(), KeyboardState::KbMapped);
}

#[test]
fn modifier_key_ignored_during_challenge() {
    let mut kb = KeyboardChallenge::new();
    kb.transition(KeyboardEvent::GrabSuccess).unwrap();

    kb.transition(KeyboardEvent::KeyPress(0x1E)).unwrap();
    let r = kb.transition(KeyboardEvent::ModifierKey).unwrap();
    assert_eq!(r, KeyboardTransitionResult::Unchanged);

    for &ch in &[0x1F, 0x20, 0x21] {
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
    for &ch in &[0x1E, 0x1F, 0x20, 0x21] {
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
}
