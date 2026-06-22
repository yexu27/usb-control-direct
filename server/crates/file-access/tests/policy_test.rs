use std::collections::HashSet;
use std::path::PathBuf;

use file_access::policy::evaluate_access;
use file_access::types::{AccessDecision, ControlledEntry, ExecFileType, PolicySnapshot};

fn make_snapshot(
    exec_control: bool,
    blacklist_enabled: bool,
    auto_read: bool,
    blacklist_exts: &[&str],
) -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: exec_control,
        file_type_blacklist_enabled: blacklist_enabled,
        auto_read_control_enabled: auto_read,
        blacklist_extensions: blacklist_exts.iter().map(|s| s.to_string()).collect(),
        permission: 1,
    }
}

fn make_entry(name: &str) -> ControlledEntry {
    ControlledEntry {
        real_path: PathBuf::from(format!("/mnt/usb_raw/{}", name)),
        virtual_name: name.to_string(),
        file_size: 1024,
        is_dir: false,
        is_virus: false,
        exec_type: None,
        extension: name.rsplit('.').next().unwrap_or("").to_lowercase(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![],
    }
}

#[test]
fn l1_virus_always_denied() {
    let snapshot = make_snapshot(false, false, false, &[]);
    let mut entry = make_entry("virus.exe");
    entry.is_virus = true;
    entry.virtual_name = "[病毒禁止访问]virus.exe".to_string();

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L1")));
}

#[test]
fn l2_exec_control_blocks_pe() {
    let snapshot = make_snapshot(true, false, false, &[]);
    let mut entry = make_entry("app.exe");
    entry.exec_type = Some(ExecFileType::Pe);

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L2")));
}

#[test]
fn l2_exec_control_blocks_elf() {
    let snapshot = make_snapshot(true, false, false, &[]);
    let mut entry = make_entry("app.bin");
    entry.exec_type = Some(ExecFileType::Elf);

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L2")));
}

#[test]
fn l2_exec_control_disabled_allows_pe() {
    let snapshot = make_snapshot(false, false, false, &[]);
    let mut entry = make_entry("app.exe");
    entry.exec_type = Some(ExecFileType::Pe);

    let decision = evaluate_access(&entry, &snapshot);
    assert_eq!(decision, AccessDecision::Allow);
}

#[test]
fn l3_blacklist_blocks_extension() {
    let snapshot = make_snapshot(false, true, false, &[".bat", ".cmd"]);
    let entry = make_entry("script.bat");

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L3")));
}

#[test]
fn l3_blacklist_case_insensitive() {
    let snapshot = make_snapshot(false, true, false, &[".exe"]);
    let mut entry = make_entry("APP.EXE");
    entry.extension = "exe".to_string();

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L3")));
}

#[test]
fn l3_blacklist_disabled_allows() {
    let snapshot = make_snapshot(false, false, false, &[".bat"]);
    let entry = make_entry("script.bat");

    let decision = evaluate_access(&entry, &snapshot);
    assert_eq!(decision, AccessDecision::Allow);
}

#[test]
fn l4_autorun_inf_blocked() {
    let snapshot = make_snapshot(false, false, true, &[]);
    let mut entry = make_entry("autorun.inf");
    entry.is_autorun_inf = true;

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L4")));
}

#[test]
fn l4_autorun_target_blocked() {
    let snapshot = make_snapshot(false, false, true, &[]);
    let mut entry = make_entry("setup.exe");
    entry.is_autorun_target = true;

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L4")));
}

#[test]
fn l4_root_shell_script_blocked() {
    let snapshot = make_snapshot(false, false, true, &[]);
    let mut entry = make_entry("run.sh");
    entry.is_root_shell_script = true;

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L4")));
}

#[test]
fn l4_disabled_allows_autorun() {
    let snapshot = make_snapshot(false, false, false, &[]);
    let mut entry = make_entry("autorun.inf");
    entry.is_autorun_inf = true;

    let decision = evaluate_access(&entry, &snapshot);
    assert_eq!(decision, AccessDecision::Allow);
}

#[test]
fn priority_l1_wins_over_l2() {
    let snapshot = make_snapshot(true, true, true, &[".exe"]);
    let mut entry = make_entry("virus.exe");
    entry.is_virus = true;
    entry.exec_type = Some(ExecFileType::Pe);

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L1")));
}

#[test]
fn priority_l2_wins_over_l3() {
    let snapshot = make_snapshot(true, true, false, &[".exe"]);
    let mut entry = make_entry("app.exe");
    entry.exec_type = Some(ExecFileType::Pe);

    let decision = evaluate_access(&entry, &snapshot);
    assert!(matches!(decision, AccessDecision::Deny(ref msg) if msg.contains("L2")));
}

#[test]
fn normal_file_allowed() {
    let snapshot = make_snapshot(true, true, true, &[".bat", ".cmd"]);
    let entry = make_entry("readme.txt");

    let decision = evaluate_access(&entry, &snapshot);
    assert_eq!(decision, AccessDecision::Allow);
}

#[test]
fn directory_always_allowed() {
    let snapshot = make_snapshot(true, true, true, &[".exe"]);
    let mut entry = make_entry("subdir");
    entry.is_dir = true;
    entry.extension = String::new();

    let decision = evaluate_access(&entry, &snapshot);
    assert_eq!(decision, AccessDecision::Allow);
}
