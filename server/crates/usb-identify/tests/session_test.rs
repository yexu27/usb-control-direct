use usb_identify::session::{
    SessionCleanupMark, StorageSession, StorageSessionRegistry, StorageSessionState,
};

fn make_session(id: &str) -> StorageSession {
    StorageSession::new(
        id.to_string(),
        "/sys/devices/test-parent".to_string(),
        "/dev/sda2".to_string(),
        3,
    )
}

#[test]
fn session_state_advances_to_mapped() {
    let mut session = make_session("session-1");

    session.set_mount_path("/mnt/usb_raw/session-1".into());
    session.set_state(StorageSessionState::Scanning);
    session.set_state(StorageSessionState::BuildingMedia);
    session.set_state(StorageSessionState::NbdStarting);
    session.set_state(StorageSessionState::Exposing);
    session.set_state(StorageSessionState::Mapped);

    assert_eq!(session.state(), StorageSessionState::Mapped);
    assert_eq!(
        session.mount_path().unwrap().to_string_lossy(),
        "/mnt/usb_raw/session-1"
    );
    assert_eq!(session.nbd_device(), "/dev/nbd3");
}

#[test]
fn cleanup_mark_is_idempotent() {
    let mut session = make_session("session-1");

    assert_eq!(
        session.mark_cleaning("usb_remove"),
        SessionCleanupMark::Started
    );
    assert_eq!(
        session.mark_cleaning("service_shutdown"),
        SessionCleanupMark::AlreadyCleaning
    );
    assert_eq!(session.cleanup_reason().unwrap(), "usb_remove");
}

#[test]
fn registry_removes_all_sessions_for_shutdown() {
    let mut registry = StorageSessionRegistry::default();
    registry.insert(make_session("session-1"));
    registry.insert(StorageSession::new(
        "session-2".to_string(),
        "/sys/devices/test-parent-2".to_string(),
        "/dev/sdb1".to_string(),
        2,
    ));

    let sessions = registry.take_all_for_shutdown("service_shutdown");

    assert_eq!(sessions.len(), 2);
    assert!(registry.is_empty());
    assert!(sessions
        .iter()
        .all(|session| session.state() == StorageSessionState::Cleaning));
}
