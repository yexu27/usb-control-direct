use hid_access::keyboard::{
    KeyboardChallenge, KeyboardEvent, KeyboardState, KeyboardTransitionResult,
};

const KEY_1: u8 = 0x1E;
const KEY_2: u8 = 0x1F;
const KEY_3: u8 = 0x20;
const KEY_4: u8 = 0x21;
const KEY_5: u8 = 0x22;
const KEY_9: u8 = 0x26;

fn start_waiting() -> KeyboardChallenge {
    let mut kb = KeyboardChallenge::new();
    let result = kb.transition(KeyboardEvent::GrabSuccess).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbWaiting)
    );
    kb
}

#[test]
fn grab_failed_transitions_to_rejected() {
    let mut kb = KeyboardChallenge::new();
    let result = kb.transition(KeyboardEvent::GrabFailed).unwrap();
    assert_eq!(
        result,
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    assert_eq!(kb.state(), KeyboardState::KbRejected);
}

#[test]
fn correct_1234_transitions_to_mapped() {
    let mut kb = start_waiting();

    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_1)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_2)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_3)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_4)).unwrap(),
        KeyboardTransitionResult::Transitioned(KeyboardState::KbMapped)
    );
    assert_eq!(kb.state(), KeyboardState::KbMapped);
}

#[test]
fn wrong_first_key_rejects_immediately() {
    let mut kb = start_waiting();

    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_9)).unwrap(),
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    assert_eq!(kb.state(), KeyboardState::KbRejected);
}

#[test]
fn wrong_middle_key_rejects_immediately() {
    let mut kb = start_waiting();

    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_1)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_2)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_5)).unwrap(),
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    assert_eq!(kb.state(), KeyboardState::KbRejected);
}

#[test]
fn wrong_last_key_rejects_immediately() {
    let mut kb = start_waiting();

    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_1)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_2)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_3)).unwrap(),
        KeyboardTransitionResult::Unchanged
    );
    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_5)).unwrap(),
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    assert_eq!(kb.state(), KeyboardState::KbRejected);
}

#[test]
fn modifier_key_during_verification_rejects() {
    let mut kb = start_waiting();

    assert_eq!(
        kb.transition(KeyboardEvent::ModifierKey).unwrap(),
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    assert_eq!(kb.state(), KeyboardState::KbRejected);
}

#[test]
fn rejected_state_does_not_accept_later_correct_code() {
    let mut kb = start_waiting();

    assert_eq!(
        kb.transition(KeyboardEvent::KeyPress(KEY_9)).unwrap(),
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRejected)
    );
    let result = kb.transition(KeyboardEvent::KeyPress(KEY_1));
    assert!(result.is_err());
    assert_eq!(kb.state(), KeyboardState::KbRejected);
}

#[test]
fn unplug_from_waiting_transitions_to_removed() {
    let mut kb = start_waiting();

    assert_eq!(
        kb.transition(KeyboardEvent::Unplug).unwrap(),
        KeyboardTransitionResult::Transitioned(KeyboardState::KbRemoved)
    );
    assert_eq!(kb.state(), KeyboardState::KbRemoved);
}
