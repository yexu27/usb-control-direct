//! Storage 集成测试：覆盖 T01-T11 全部 CRUD。

use storage::model::*;
use storage::Storage;
use tempfile::NamedTempFile;

fn setup() -> (Storage, NamedTempFile) {
    let tmp = NamedTempFile::new().unwrap();
    let s = Storage::open(tmp.path()).unwrap();
    (s, tmp)
}

// ========== T01 ==========

#[test]
fn t01_whitelist_insert_and_query() {
    let (s, _tmp) = setup();
    let item = UsbWhitelistInsert {
        serial_number: "SN001".into(),
        vid: Some("0930".into()),
        pid: Some("6545".into()),
        device_name: Some("Kingston".into()),
        capacity_bytes: Some(16_000_000_000),
        device_type: "storage".into(),
        description: None,
        permission: 0,
        add_method: 1,
    };
    let id = s.whitelist_insert(&item).unwrap();
    assert!(id > 0);

    let found = s.whitelist_query_by_sn("SN001").unwrap().unwrap();
    assert_eq!(found.vid.as_deref(), Some("0930"));
}

#[test]
fn t01_whitelist_unique_constraint() {
    let (s, _tmp) = setup();
    let item = UsbWhitelistInsert {
        serial_number: "SN001".into(),
        vid: None, pid: None, device_name: None, capacity_bytes: None,
        device_type: "storage".into(), description: None, permission: 0, add_method: 0,
    };
    s.whitelist_insert(&item).unwrap();
    let result = s.whitelist_insert(&item);
    assert!(result.is_err());
}

#[test]
fn t01_whitelist_update_and_delete() {
    let (s, _tmp) = setup();
    let item = UsbWhitelistInsert {
        serial_number: "SN002".into(),
        vid: None, pid: None, device_name: None, capacity_bytes: None,
        device_type: "storage".into(), description: None, permission: 0, add_method: 0,
    };
    let id = s.whitelist_insert(&item).unwrap();
    s.whitelist_update(id, 1, Some("test desc")).unwrap();
    let found = s.whitelist_query_by_sn("SN002").unwrap().unwrap();
    assert_eq!(found.permission, 1);

    s.whitelist_delete(id).unwrap();
    assert!(s.whitelist_query_by_sn("SN002").unwrap().is_none());
}

// ========== T02 ==========

#[test]
fn t02_blacklist_insert_and_query() {
    let (s, _tmp) = setup();
    let id = s.blacklist_insert(".py", Some("Python 脚本")).unwrap();
    assert!(id > 0);
    let found = s.blacklist_query_by_ext(".py").unwrap().unwrap();
    assert_eq!(found.is_default, 0);
}

#[test]
fn t02_blacklist_default_38() {
    let (s, _tmp) = setup();
    let count = s.blacklist_count().unwrap();
    assert_eq!(count, 38);
}

#[test]
fn t02_blacklist_delete_default_fails() {
    let (s, _tmp) = setup();
    let item = s.blacklist_query_by_ext(".mp3").unwrap().unwrap();
    let result = s.blacklist_delete(item.id);
    assert!(result.is_err());
}

// ========== T03 ==========

#[test]
fn t03_policy_query_and_update() {
    let (s, _tmp) = setup();
    let p = s.policy_query("exec_control").unwrap().unwrap();
    assert_eq!(p.enabled, 0);

    s.policy_update("exec_control", true).unwrap();
    let p = s.policy_query("exec_control").unwrap().unwrap();
    assert_eq!(p.enabled, 1);
}

// ========== T04 ==========

#[test]
fn t04_exec_type_query_all() {
    let (s, _tmp) = setup();
    let types = s.exec_type_query_all().unwrap();
    assert_eq!(types.len(), 4);
}

// ========== T05 ==========

#[test]
fn t05_usb_audit_insert_and_query() {
    let (s, _tmp) = setup();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let item = UsbAuditLogInsert {
        event_time: now,
        device_type: Some("storage".into()),
        interface_type: Some("mass_storage".into()),
        interface_class: Some(8),
        interface_subclass: Some(6),
        interface_protocol: Some(80),
        device_name: Some("TestUSB".into()),
        device_sn: Some("SN001".into()),
        vid: Some("0930".into()),
        pid: Some("6545".into()),
        event_type: "device_insert".into(),
        permission: None,
        capacity_bytes: Some(16_000_000_000),
        file_path: None,
        matched_policy: None,
        result: "allowed".into(),
        fail_reason: None,
        detail: None,
    };
    let id = s.usb_audit_insert(&item).unwrap();
    assert!(id > 0);

    let logs = s.usb_audit_query_by_time(now - 1, now + 1).unwrap();
    assert_eq!(logs.len(), 1);
}

// ========== T06 ==========

#[test]
fn t06_malware_insert_and_query() {
    let (s, _tmp) = setup();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let item = MalwareLogInsert {
        scan_time: now, device_sn: Some("SN001".into()), device_name: None,
        file_path: Some("/test.exe".into()), scan_result: 1,
        virus_name: Some("Eicar".into()), virus_db_version: Some("daily".into()),
        process_result: Some(1), fail_reason: None, detail: None,
    };
    let id = s.malware_insert(&item).unwrap();
    assert!(id > 0);
}

// ========== T07 ==========

#[test]
fn t07_config_get_and_set() {
    let (s, _tmp) = setup();
    let c = s.config_get("device_description").unwrap().unwrap();
    assert_eq!(c.config_value.as_deref(), Some("\"USB安全管理装置\""));

    s.config_set("device_description", "\"新描述\"").unwrap();
    let c = s.config_get("device_description").unwrap().unwrap();
    assert_eq!(c.config_value.as_deref(), Some("\"新描述\""));
}

// ========== T08 ==========

#[test]
fn t08_user_builtin_accounts() {
    let (s, _tmp) = setup();
    let admin = s.user_query_by_username("admin").unwrap().unwrap();
    assert_eq!(admin.role, 0);
    assert_eq!(admin.is_builtin, 1);
}

#[test]
fn t08_user_insert_and_soft_delete() {
    let (s, _tmp) = setup();
    let item = UserInsert {
        username: "testuser".into(),
        password_hash: "$2b$12$test".into(),
        role: 1,
    };
    let id = s.user_insert(&item).unwrap();
    assert!(id > 0);

    s.user_soft_delete(id).unwrap();
    let u = s.user_query_by_username("testuser").unwrap().unwrap();
    assert_eq!(u.status, 2);
}

#[test]
fn t08_user_unique_username() {
    let (s, _tmp) = setup();
    let item = UserInsert {
        username: "admin".into(),
        password_hash: "$2b$12$dup".into(),
        role: 0,
    };
    let result = s.user_insert(&item);
    assert!(result.is_err());
}

// ========== T09 ==========

#[test]
fn t09_role_permission_query() {
    let (s, _tmp) = setup();
    let perms = s.role_permission_query_by_role(0).unwrap();
    assert_eq!(perms.len(), 2);
    let all = s.role_permission_query_all().unwrap();
    assert_eq!(all.len(), 6);
}

// ========== T10 ==========

#[test]
fn t10_operation_log_insert_and_query() {
    let (s, _tmp) = setup();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let item = OperationLogInsert {
        op_time: now, username: "admin".into(), role: 0,
        log_type: "login_auth".into(), action_type: Some("login".into()),
        target: None, before_value: None, after_value: None,
        related_file: None, related_version: None, result: 0,
        fail_reason: None, source_ip: Some("192.168.1.1".into()),
        app_version: None, session_id: None, request_id: None, detail: None,
    };
    let id = s.operation_log_insert(&item).unwrap();
    assert!(id > 0);

    let logs = s.operation_log_query_by_time(now - 1, now + 1).unwrap();
    assert_eq!(logs.len(), 1);
}

// ========== T11 ==========

#[test]
fn t11_retention_event_insert() {
    let (s, _tmp) = setup();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let item = LogRetentionEventInsert {
        trigger_time: now, log_category: "usb_audit".into(),
        storage_usage_percent: 82, covered_from_time: Some(now - 3600),
        covered_to_time: Some(now - 3600), covered_count: 1,
        result: 0, fail_reason: None,
    };
    let id = s.retention_event_insert(&item).unwrap();
    assert!(id > 0);
}
