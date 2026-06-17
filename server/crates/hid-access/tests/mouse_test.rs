use common::types::MouseState;
use hid_access::mouse::{MouseAdmission, MouseEvent, MouseTransitionResult};

#[test]
fn map_success_transitions_to_mapped() {
    let mut mouse = MouseAdmission::new();
    assert_eq!(mouse.state(), MouseState::MouseDetected);

    let result = mouse.transition(MouseEvent::MapSuccess).unwrap();
    assert_eq!(
        result,
        MouseTransitionResult::Transitioned(MouseState::MouseMapped)
    );
}

#[test]
fn map_failed_transitions_to_rejected() {
    let mut mouse = MouseAdmission::new();
    let result = mouse.transition(MouseEvent::MapFailed).unwrap();
    assert_eq!(
        result,
        MouseTransitionResult::Transitioned(MouseState::MouseRejected)
    );
}

#[test]
fn unplug_from_detected() {
    let mut mouse = MouseAdmission::new();
    let result = mouse.transition(MouseEvent::Unplug).unwrap();
    assert_eq!(
        result,
        MouseTransitionResult::Transitioned(MouseState::MouseRemoved)
    );
}

#[test]
fn unplug_from_mapped() {
    let mut mouse = MouseAdmission::new();
    mouse.transition(MouseEvent::MapSuccess).unwrap();
    let result = mouse.transition(MouseEvent::Unplug).unwrap();
    assert_eq!(
        result,
        MouseTransitionResult::Transitioned(MouseState::MouseRemoved)
    );
}

#[test]
fn unplug_from_rejected() {
    let mut mouse = MouseAdmission::new();
    mouse.transition(MouseEvent::MapFailed).unwrap();
    let result = mouse.transition(MouseEvent::Unplug).unwrap();
    assert_eq!(
        result,
        MouseTransitionResult::Transitioned(MouseState::MouseRemoved)
    );
}

#[test]
fn invalid_transition_map_in_mapped() {
    let mut mouse = MouseAdmission::new();
    mouse.transition(MouseEvent::MapSuccess).unwrap();
    let result = mouse.transition(MouseEvent::MapSuccess);
    assert!(result.is_err());
    assert_eq!(mouse.state(), MouseState::MouseMapped);
}
