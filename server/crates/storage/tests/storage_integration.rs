//! Storage 集成测试：覆盖 T01-T11 全部 CRUD。

use storage::model::*;
use storage::{Storage, StorageError};
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
    assert!(matches!(result.unwrap_err(), StorageError::AlreadyExists));
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

    let logs = s.malware_query_by_time(now - 1, now + 1).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].virus_name.as_deref(), Some("Eicar"));
    assert_eq!(logs[0].scan_result, 1);

    let empty = s.malware_query_by_time(now + 100, now + 200).unwrap();
    assert!(empty.is_empty());
}

// ========== T07 ==========

#[test]
fn t07_config_get_and_set() {
    let (s, _tmp) = setup();
    let c = s.config_get("device_description").unwrap().unwrap();
    assert_eq!(c.config_value.as_deref(), Some("\"(AD USB protection dev)USB Device\""));

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

// ========== T05 分页查询 / 批量删除 ==========

fn make_usb_audit(event_time: i64, event_type: &str, device_name: &str) -> UsbAuditLogInsert {
    UsbAuditLogInsert {
        event_time,
        device_type: Some("storage".into()),
        interface_type: None,
        interface_class: None,
        interface_subclass: None,
        interface_protocol: None,
        device_name: Some(device_name.into()),
        device_sn: Some("SN-TEST".into()),
        vid: None,
        pid: None,
        event_type: event_type.into(),
        permission: None,
        capacity_bytes: None,
        file_path: None,
        matched_policy: None,
        result: "allowed".into(),
        fail_reason: None,
        detail: None,
    }
}

#[test]
fn t05_usb_audit_paged_query() {
    let (s, _tmp) = setup();
    let base = 1_700_000_000_i64;
    s.usb_audit_insert(&make_usb_audit(base, "device_insert", "KingstonUSB")).unwrap();
    s.usb_audit_insert(&make_usb_audit(base + 1, "device_insert", "SandiskUSB")).unwrap();
    s.usb_audit_insert(&make_usb_audit(base + 2, "file_read", "KingstonUSB")).unwrap();

    // 关键字过滤 Kingston，应匹配 2 条
    let params = LogQueryParams {
        keyword: Some("Kingston".into()),
        page: 1,
        page_size: 10,
        ..Default::default()
    };
    let (rows, total) = s.usb_audit_query_paged(&params).unwrap();
    assert_eq!(total, 2);
    assert_eq!(rows.len(), 2);

    // event_type 过滤，应匹配 1 条
    let params = LogQueryParams {
        event_type: Some("file_read".into()),
        page: 1,
        page_size: 10,
        ..Default::default()
    };
    let (rows, total) = s.usb_audit_query_paged(&params).unwrap();
    assert_eq!(total, 1);
    assert_eq!(rows[0].event_type, "file_read");
}

#[test]
fn t05_usb_audit_paged_query_pagination() {
    let (s, _tmp) = setup();
    let base = 1_700_000_100_i64;
    for i in 0..5_i64 {
        s.usb_audit_insert(&make_usb_audit(base + i, "device_insert", "Device")).unwrap();
    }

    let params = LogQueryParams {
        page: 1,
        page_size: 2,
        ..Default::default()
    };
    let (rows, total) = s.usb_audit_query_paged(&params).unwrap();
    assert_eq!(total, 5);
    assert_eq!(rows.len(), 2);

    // 第 3 页应只有 1 条
    let params = LogQueryParams {
        page: 3,
        page_size: 2,
        ..Default::default()
    };
    let (rows, total) = s.usb_audit_query_paged(&params).unwrap();
    assert_eq!(total, 5);
    assert_eq!(rows.len(), 1);
}

#[test]
fn t05_usb_audit_delete_by_time() {
    let (s, _tmp) = setup();
    let base = 1_700_000_200_i64;
    s.usb_audit_insert(&make_usb_audit(base, "device_insert", "D1")).unwrap();
    s.usb_audit_insert(&make_usb_audit(base + 10, "device_insert", "D2")).unwrap();
    s.usb_audit_insert(&make_usb_audit(base + 20, "device_insert", "D3")).unwrap();

    // 删除前两条
    let deleted = s.usb_audit_delete_by_time(base, base + 10).unwrap();
    assert_eq!(deleted, 2);

    let count = s.usb_audit_count().unwrap();
    assert_eq!(count, 1);
}

// ========== T06 分页查询 / 批量删除 ==========

fn make_malware(scan_time: i64, virus_name: &str, device_name: &str) -> MalwareLogInsert {
    MalwareLogInsert {
        scan_time,
        device_sn: Some("SN-MAL".into()),
        device_name: Some(device_name.into()),
        file_path: Some("/tmp/test.exe".into()),
        scan_result: 1,
        virus_name: Some(virus_name.into()),
        virus_db_version: Some("daily".into()),
        process_result: Some(1),
        fail_reason: None,
        detail: None,
    }
}

#[test]
fn t06_malware_paged_query() {
    let (s, _tmp) = setup();
    let base = 1_700_001_000_i64;
    s.malware_insert(&make_malware(base, "Eicar", "USB-A")).unwrap();
    s.malware_insert(&make_malware(base + 1, "Trojan.X", "USB-B")).unwrap();
    s.malware_insert(&make_malware(base + 2, "Eicar", "USB-C")).unwrap();

    // 关键字过滤 Eicar，应匹配 2 条
    let params = LogQueryParams {
        keyword: Some("Eicar".into()),
        page: 1,
        page_size: 10,
        ..Default::default()
    };
    let (rows, total) = s.malware_query_paged(&params).unwrap();
    assert_eq!(total, 2);
    assert_eq!(rows.len(), 2);

    // 时间范围过滤
    let params = LogQueryParams {
        start_time: Some(base + 1),
        end_time: Some(base + 2),
        page: 1,
        page_size: 10,
        ..Default::default()
    };
    let (rows, total) = s.malware_query_paged(&params).unwrap();
    assert_eq!(total, 2);
    assert_eq!(rows.len(), 2);
}

#[test]
fn t06_malware_delete_by_time() {
    let (s, _tmp) = setup();
    let base = 1_700_001_100_i64;
    s.malware_insert(&make_malware(base, "V1", "D1")).unwrap();
    s.malware_insert(&make_malware(base + 10, "V2", "D2")).unwrap();
    s.malware_insert(&make_malware(base + 20, "V3", "D3")).unwrap();

    let deleted = s.malware_delete_by_time(base, base + 10).unwrap();
    assert_eq!(deleted, 2);

    let count = s.malware_count().unwrap();
    assert_eq!(count, 1);
}

// ========== T10 分页查询 / 批量删除 ==========

fn make_op_log(op_time: i64, log_type: &str, action_type: &str, username: &str) -> OperationLogInsert {
    OperationLogInsert {
        op_time,
        username: username.into(),
        role: 0,
        log_type: log_type.into(),
        action_type: Some(action_type.into()),
        target: Some("target_obj".into()),
        before_value: None,
        after_value: None,
        related_file: None,
        related_version: None,
        result: 0,
        fail_reason: None,
        source_ip: Some("127.0.0.1".into()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: None,
    }
}

#[test]
fn t10_operation_log_paged_query() {
    let (s, _tmp) = setup();
    let base = 1_700_002_000_i64;
    s.operation_log_insert(&make_op_log(base, "login_auth", "login", "admin")).unwrap();
    s.operation_log_insert(&make_op_log(base + 1, "policy_manage", "create", "admin")).unwrap();
    s.operation_log_insert(&make_op_log(base + 2, "login_auth", "logout", "operator")).unwrap();

    // log_category 过滤 login_auth
    let params = LogQueryParams {
        log_category: Some("login_auth".into()),
        page: 1,
        page_size: 10,
        ..Default::default()
    };
    let (rows, total) = s.operation_log_query_paged(&params).unwrap();
    assert_eq!(total, 2);
    assert_eq!(rows.len(), 2);

    // action_type 过滤
    let params = LogQueryParams {
        action_type: Some("create".into()),
        page: 1,
        page_size: 10,
        ..Default::default()
    };
    let (rows, total) = s.operation_log_query_paged(&params).unwrap();
    assert_eq!(total, 1);
    assert_eq!(rows[0].log_type, "policy_manage");

    // 关键字过滤用户名
    let params = LogQueryParams {
        keyword: Some("operator".into()),
        page: 1,
        page_size: 10,
        ..Default::default()
    };
    let (rows, total) = s.operation_log_query_paged(&params).unwrap();
    assert_eq!(total, 1);
    assert_eq!(rows[0].username, "operator");
}

#[test]
fn t10_operation_log_delete_by_time() {
    let (s, _tmp) = setup();
    let base = 1_700_002_100_i64;
    s.operation_log_insert(&make_op_log(base, "login_auth", "login", "admin")).unwrap();
    s.operation_log_insert(&make_op_log(base + 10, "login_auth", "logout", "admin")).unwrap();
    s.operation_log_insert(&make_op_log(base + 20, "policy_manage", "create", "admin")).unwrap();

    let deleted = s.operation_log_delete_by_time(base, base + 10).unwrap();
    assert_eq!(deleted, 2);

    let count = s.operation_log_count().unwrap();
    assert_eq!(count, 1);
}
