use common::types::UsbDeviceState;
use usb_identify::state_machine::{UsbDeviceEvent, UsbStateMachine};

fn new_sm_at(target: UsbDeviceState) -> UsbStateMachine {
    let mut sm = UsbStateMachine::new();
    match target {
        UsbDeviceState::Detected => {}
        UsbDeviceState::Classifying => {
            sm.transition(UsbDeviceEvent::Inserted).unwrap();
        }
        UsbDeviceState::Scanning => {
            sm.transition(UsbDeviceEvent::Inserted).unwrap();
            sm.transition(UsbDeviceEvent::WhitelistApproved).unwrap();
        }
        UsbDeviceState::Clean => {
            sm.transition(UsbDeviceEvent::Inserted).unwrap();
            sm.transition(UsbDeviceEvent::WhitelistApproved).unwrap();
            sm.transition(UsbDeviceEvent::ScanClean).unwrap();
        }
        UsbDeviceState::Infected => {
            sm.transition(UsbDeviceEvent::Inserted).unwrap();
            sm.transition(UsbDeviceEvent::WhitelistApproved).unwrap();
            sm.transition(UsbDeviceEvent::ScanInfected {
                infected_files: vec!["virus.exe".into()],
            })
            .unwrap();
        }
        _ => panic!("helper only supports states up to Infected"),
    }
    sm
}

// ===== 合法转换 =====

#[test]
fn detected_to_classifying() {
    let mut sm = UsbStateMachine::new();
    let r = sm.transition(UsbDeviceEvent::Inserted).unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Classifying);
}

#[test]
fn classifying_to_rejected() {
    let mut sm = new_sm_at(UsbDeviceState::Classifying);
    let r = sm
        .transition(UsbDeviceEvent::ClassifiedAsRejected {
            reason: "非存储设备".into(),
        })
        .unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Rejected);
}

#[test]
fn classifying_to_blocked() {
    let mut sm = new_sm_at(UsbDeviceState::Classifying);
    let r = sm.transition(UsbDeviceEvent::WhitelistDenied).unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Blocked);
}

#[test]
fn classifying_to_scanning() {
    let mut sm = new_sm_at(UsbDeviceState::Classifying);
    let r = sm.transition(UsbDeviceEvent::WhitelistApproved).unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Scanning);
}

#[test]
fn classifying_mount_failed_to_scan_failed() {
    let mut sm = new_sm_at(UsbDeviceState::Classifying);
    let r = sm
        .transition(UsbDeviceEvent::MountFailed {
            reason: "文件系统损坏".into(),
        })
        .unwrap();
    assert_eq!(r.new_state, UsbDeviceState::ScanFailed);
}

#[test]
fn scanning_to_clean() {
    let mut sm = new_sm_at(UsbDeviceState::Scanning);
    let r = sm.transition(UsbDeviceEvent::ScanClean).unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Clean);
}

#[test]
fn scanning_to_infected() {
    let mut sm = new_sm_at(UsbDeviceState::Scanning);
    let r = sm
        .transition(UsbDeviceEvent::ScanInfected {
            infected_files: vec!["malware.exe".into()],
        })
        .unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Infected);
}

#[test]
fn scanning_to_scan_failed() {
    let mut sm = new_sm_at(UsbDeviceState::Scanning);
    let r = sm
        .transition(UsbDeviceEvent::ScanFailed {
            reason: "clamd 不可用".into(),
        })
        .unwrap();
    assert_eq!(r.new_state, UsbDeviceState::ScanFailed);
}

#[test]
fn clean_to_mapped() {
    let mut sm = new_sm_at(UsbDeviceState::Clean);
    let r = sm.transition(UsbDeviceEvent::MapSuccess).unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Mapped);
}

#[test]
fn clean_to_map_failed() {
    let mut sm = new_sm_at(UsbDeviceState::Clean);
    let r = sm
        .transition(UsbDeviceEvent::MapFailed {
            reason: "NBD 启动失败".into(),
        })
        .unwrap();
    assert_eq!(r.new_state, UsbDeviceState::MapFailed);
}

#[test]
fn infected_to_mapped() {
    let mut sm = new_sm_at(UsbDeviceState::Infected);
    let r = sm.transition(UsbDeviceEvent::MapSuccess).unwrap();
    assert_eq!(r.new_state, UsbDeviceState::Mapped);
}

#[test]
fn infected_to_map_failed() {
    let mut sm = new_sm_at(UsbDeviceState::Infected);
    let r = sm
        .transition(UsbDeviceEvent::MapFailed {
            reason: "OTG 绑定失败".into(),
        })
        .unwrap();
    assert_eq!(r.new_state, UsbDeviceState::MapFailed);
}

// ===== 全流程 =====

#[test]
fn full_path_clean_to_mapped() {
    let mut sm = UsbStateMachine::new();
    sm.transition(UsbDeviceEvent::Inserted).unwrap();
    sm.transition(UsbDeviceEvent::WhitelistApproved).unwrap();
    sm.transition(UsbDeviceEvent::ScanClean).unwrap();
    sm.transition(UsbDeviceEvent::MapSuccess).unwrap();
    assert_eq!(sm.state(), UsbDeviceState::Mapped);
}

#[test]
fn full_path_infected_to_mapped() {
    let mut sm = UsbStateMachine::new();
    sm.transition(UsbDeviceEvent::Inserted).unwrap();
    sm.transition(UsbDeviceEvent::WhitelistApproved).unwrap();
    sm.transition(UsbDeviceEvent::ScanInfected {
        infected_files: vec!["virus.exe".into()],
    })
    .unwrap();
    sm.transition(UsbDeviceEvent::MapSuccess).unwrap();
    assert_eq!(sm.state(), UsbDeviceState::Mapped);
}

// ===== 拔出 =====

#[test]
fn unplug_from_every_state() {
    for state in [
        UsbDeviceState::Detected,
        UsbDeviceState::Classifying,
        UsbDeviceState::Scanning,
        UsbDeviceState::Clean,
        UsbDeviceState::Infected,
    ] {
        let mut sm = new_sm_at(state);
        let r = sm.transition(UsbDeviceEvent::Unplug).unwrap();
        assert_eq!(r.new_state, UsbDeviceState::Removed);
    }
}

// ===== 非法转换 =====

#[test]
fn invalid_scan_clean_in_classifying() {
    let mut sm = new_sm_at(UsbDeviceState::Classifying);
    let result = sm.transition(UsbDeviceEvent::ScanClean);
    assert!(result.is_err());
    assert_eq!(sm.state(), UsbDeviceState::Classifying);
}

#[test]
fn invalid_map_success_in_scanning() {
    let mut sm = new_sm_at(UsbDeviceState::Scanning);
    let result = sm.transition(UsbDeviceEvent::MapSuccess);
    assert!(result.is_err());
    assert_eq!(sm.state(), UsbDeviceState::Scanning);
}

#[test]
fn invalid_whitelist_approved_in_scanning() {
    let mut sm = new_sm_at(UsbDeviceState::Scanning);
    let result = sm.transition(UsbDeviceEvent::WhitelistApproved);
    assert!(result.is_err());
    assert_eq!(sm.state(), UsbDeviceState::Scanning);
}

#[test]
fn invalid_inserted_in_classifying() {
    let mut sm = new_sm_at(UsbDeviceState::Classifying);
    let result = sm.transition(UsbDeviceEvent::Inserted);
    assert!(result.is_err());
    assert_eq!(sm.state(), UsbDeviceState::Classifying);
}
