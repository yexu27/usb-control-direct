//! Storage 集成测试：覆盖 T01-T11 全部 CRUD。

use storage::model::*;
use storage::{Storage, StorageError};
use storage_test_support::TestDb;

fn setup() -> (Storage, TestDb) {
    let db = TestDb::new();
    let s = Storage::open(db.path()).unwrap();
    (s, db)
}

#[test]
fn open_rejects_uninitialized_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.db");
    let result = Storage::open(&path);
    assert!(matches!(
        result,
        Err(StorageError::DatabaseNotInitialized(_))
    ));
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
    let id = s.blacklist_insert(".custom", Some("自定义后缀")).unwrap();
    assert!(id > 0);
    let found = s.blacklist_query_by_ext(".CUSTOM").unwrap().unwrap();
    assert_eq!(found.is_default, 0);
}

#[test]
fn t02_blacklist_default_38() {
    let (s, _tmp) = setup();
    let count = s.blacklist_count().unwrap();
    assert_eq!(count, 38);
}

#[test]
fn t02_blacklist_delete_default_succeeds() {
    let (s, _tmp) = setup();
    s.blacklist_delete(".ps1").unwrap();
    assert!(s.blacklist_query_by_ext(".ps1").unwrap().is_none());
}

fn imported_whitelist(serial_number: &str) -> UsbWhitelistInsert {
    UsbWhitelistInsert {
        serial_number: serial_number.into(),
        vid: None,
        pid: None,
        device_name: None,
        capacity_bytes: None,
        device_type: "storage".into(),
        description: None,
        permission: 0,
        add_method: 0,
    }
}

fn complete_policy_updates() -> Vec<(String, i32)> {
    vec![
        ("exec_control".into(), 1),
        ("auto_read_control".into(), 1),
        ("file_type_blacklist_control".into(), 1),
    ]
}

#[test]
fn policy_import_replaces_entire_blacklist_including_local_defaults() {
    let (s, _tmp) = setup();
    let imported = vec![(".ONLY".into(), Some("导入项".into()), 1)];

    s.policy_import_transaction(&[], &complete_policy_updates(), &imported)
        .unwrap();

    let blacklist = s.blacklist_query_all().unwrap();
    assert_eq!(blacklist.len(), 1);
    assert_eq!(blacklist[0].extension, ".only");
    assert_eq!(blacklist[0].is_default, 1);
}

#[test]
fn policy_import_rejects_invalid_or_normalized_duplicate_extensions_before_changes() {
    for blacklist in [
        vec![("exe".into(), None, 0)],
        vec![(".EXE".into(), None, 1), (" .exe ".into(), None, 0)],
    ] {
        let (s, _tmp) = setup();
        s.whitelist_insert(&imported_whitelist("ORIGINAL")).unwrap();
        let original_blacklist_count = s.blacklist_count().unwrap();

        let result = s.policy_import_transaction(
            &[imported_whitelist("IMPORTED")],
            &complete_policy_updates(),
            &blacklist,
        );

        assert!(matches!(result, Err(StorageError::Validation(_))));
        assert!(s.whitelist_query_by_sn("ORIGINAL").unwrap().is_some());
        assert!(s.whitelist_query_by_sn("IMPORTED").unwrap().is_none());
        assert_eq!(s.blacklist_count().unwrap(), original_blacklist_count);
        assert_eq!(s.policy_query("exec_control").unwrap().unwrap().enabled, 0);
    }
}

#[test]
fn policy_import_rejects_duplicate_whitelist_serial_before_changes() {
    let (s, _tmp) = setup();
    s.whitelist_insert(&imported_whitelist("ORIGINAL")).unwrap();

    let result = s.policy_import_transaction(
        &[imported_whitelist("DUP"), imported_whitelist("DUP")],
        &complete_policy_updates(),
        &[],
    );

    assert!(matches!(result, Err(StorageError::Validation(_))));
    assert!(s.whitelist_query_by_sn("ORIGINAL").unwrap().is_some());
    assert_eq!(s.policy_query("exec_control").unwrap().unwrap().enabled, 0);
    assert_eq!(s.blacklist_count().unwrap(), 38);
}

#[test]
fn policy_import_rejects_blank_whitelist_serial_before_changes() {
    let (s, _tmp) = setup();
    s.whitelist_insert(&imported_whitelist("ORIGINAL")).unwrap();

    let result = s.policy_import_transaction(
        &[imported_whitelist("   ")],
        &complete_policy_updates(),
        &[],
    );

    assert!(matches!(result, Err(StorageError::Validation(_))));
    assert!(s.whitelist_query_by_sn("ORIGINAL").unwrap().is_some());
    assert_eq!(s.policy_query("exec_control").unwrap().unwrap().enabled, 0);
    assert_eq!(s.blacklist_count().unwrap(), 38);
}

#[test]
fn policy_import_rejects_invalid_field_values_before_changes() {
    for field in [
        "permission",
        "add_method",
        "device_type",
        "capacity_bytes",
        "enabled",
        "is_default",
    ] {
        let (s, _tmp) = setup();
        s.whitelist_insert(&imported_whitelist("ORIGINAL")).unwrap();
        let original_whitelist: Vec<_> = s
            .whitelist_query_all()
            .unwrap()
            .into_iter()
            .map(|item| (item.serial_number, item.permission, item.add_method))
            .collect();
        let original_policies: Vec<_> = s
            .policy_query_all()
            .unwrap()
            .into_iter()
            .map(|item| (item.policy_key, item.enabled, item.updated_at))
            .collect();
        let original_blacklist: Vec<_> = s
            .blacklist_query_all()
            .unwrap()
            .into_iter()
            .map(|item| (item.extension, item.description, item.is_default))
            .collect();
        let mut whitelist = imported_whitelist("IMPORTED");
        let mut policies = complete_policy_updates();
        let mut blacklist = vec![(".imported".into(), None, 0)];
        match field {
            "permission" => whitelist.permission = 2,
            "add_method" => whitelist.add_method = -1,
            "device_type" => whitelist.device_type = "keyboard".into(),
            "capacity_bytes" => whitelist.capacity_bytes = Some(-1),
            "enabled" => policies[0].1 = 2,
            "is_default" => blacklist[0].2 = 2,
            _ => unreachable!(),
        }

        let result = s.policy_import_transaction(&[whitelist], &policies, &blacklist);

        assert!(
            matches!(result, Err(StorageError::Validation(_))),
            "应拒绝非法字段: {field}"
        );
        assert_eq!(
            s.whitelist_query_all()
                .unwrap()
                .into_iter()
                .map(|item| (item.serial_number, item.permission, item.add_method))
                .collect::<Vec<_>>(),
            original_whitelist,
            "非法字段 {field} 不得修改白名单"
        );
        assert_eq!(
            s.policy_query_all()
                .unwrap()
                .into_iter()
                .map(|item| (item.policy_key, item.enabled, item.updated_at))
                .collect::<Vec<_>>(),
            original_policies,
            "非法字段 {field} 不得修改策略开关"
        );
        assert_eq!(
            s.blacklist_query_all()
                .unwrap()
                .into_iter()
                .map(|item| (item.extension, item.description, item.is_default))
                .collect::<Vec<_>>(),
            original_blacklist,
            "非法字段 {field} 不得修改黑名单"
        );
    }
}

#[test]
fn policy_import_rejects_missing_unknown_or_duplicate_policy_keys_before_changes() {
    let invalid_policy_sets = [
        vec![
            ("exec_control".into(), 1),
            ("auto_read_control".into(), 1),
        ],
        vec![
            ("exec_control".into(), 1),
            ("auto_read_control".into(), 1),
            ("file_type_blacklist_control".into(), 1),
            ("unknown".into(), 1),
        ],
        vec![
            ("exec_control".into(), 1),
            ("exec_control".into(), 0),
            ("auto_read_control".into(), 1),
            ("file_type_blacklist_control".into(), 1),
        ],
    ];

    for policies in invalid_policy_sets {
        let (s, _tmp) = setup();
        s.whitelist_insert(&imported_whitelist("ORIGINAL")).unwrap();

        let result = s.policy_import_transaction(
            &[imported_whitelist("IMPORTED")],
            &policies,
            &[(".imported".into(), None, 0)],
        );

        assert!(matches!(result, Err(StorageError::Validation(_))));
        assert!(s.whitelist_query_by_sn("ORIGINAL").unwrap().is_some());
        assert!(s.whitelist_query_by_sn("IMPORTED").unwrap().is_none());
        assert_eq!(s.policy_query("exec_control").unwrap().unwrap().enabled, 0);
        assert_eq!(s.blacklist_count().unwrap(), 38);
    }
}

#[test]
fn policy_import_rejects_policy_update_that_affects_no_row() {
    let (s, tmp) = setup();
    s.whitelist_insert(&imported_whitelist("ORIGINAL")).unwrap();
    let injector = rusqlite::Connection::open(tmp.path()).unwrap();
    injector
        .execute_batch(
            "CREATE TRIGGER ignore_auto_read_policy_update \
             BEFORE UPDATE ON file_access_policy \
             WHEN NEW.policy_key = 'auto_read_control' \
             BEGIN SELECT RAISE(IGNORE); END;",
        )
        .unwrap();

    let result = s.policy_import_transaction(
        &[imported_whitelist("IMPORTED")],
        &complete_policy_updates(),
        &[],
    );

    assert!(matches!(result, Err(StorageError::NotFound(_))));
    assert!(s.whitelist_query_by_sn("ORIGINAL").unwrap().is_some());
    assert!(s.whitelist_query_by_sn("IMPORTED").unwrap().is_none());
    for policy_key in [
        "exec_control",
        "auto_read_control",
        "file_type_blacklist_control",
    ] {
        assert_eq!(s.policy_query(policy_key).unwrap().unwrap().enabled, 0);
    }
    assert_eq!(s.blacklist_count().unwrap(), 38);
}

#[test]
fn policy_import_rolls_back_all_tables_when_blacklist_insert_fails_after_clears() {
    let (s, tmp) = setup();
    s.whitelist_insert(&imported_whitelist("ORIGINAL")).unwrap();
    s.policy_update("auto_read_control", true).unwrap();
    let original_whitelist: Vec<_> = s
        .whitelist_query_all()
        .unwrap()
        .into_iter()
        .map(|item| {
            (
                item.id,
                item.serial_number,
                item.vid,
                item.pid,
                item.device_name,
                item.capacity_bytes,
                item.device_type,
                item.description,
                item.permission,
                item.add_method,
                item.created_at,
            )
        })
        .collect();
    let original_policies: Vec<_> = s
        .policy_query_all()
        .unwrap()
        .into_iter()
        .map(|item| (item.policy_key, item.enabled, item.updated_at))
        .collect();
    let original_blacklist: Vec<_> = s
        .blacklist_query_all()
        .unwrap()
        .into_iter()
        .map(|item| {
            (
                item.id,
                item.extension,
                item.description,
                item.is_default,
                item.created_at,
            )
        })
        .collect();
    let injector = rusqlite::Connection::open(tmp.path()).unwrap();
    injector
        .execute_batch(
            "CREATE TRIGGER fail_policy_import_blacklist \
             BEFORE INSERT ON file_type_blacklist \
             WHEN NEW.extension = '.trigger_fail' \
             BEGIN SELECT RAISE(ABORT, 'injected blacklist insert failure'); END;",
        )
        .unwrap();

    let result = s.policy_import_transaction(
        &[imported_whitelist("IMPORTED")],
        &[
            ("exec_control".into(), 1),
            ("auto_read_control".into(), 0),
            ("file_type_blacklist_control".into(), 1),
        ],
        &[(".trigger_fail".into(), None, 0)],
    );

    assert!(matches!(result, Err(StorageError::Sqlite(_))));
    let whitelist_after: Vec<_> = s
        .whitelist_query_all()
        .unwrap()
        .into_iter()
        .map(|item| {
            (
                item.id,
                item.serial_number,
                item.vid,
                item.pid,
                item.device_name,
                item.capacity_bytes,
                item.device_type,
                item.description,
                item.permission,
                item.add_method,
                item.created_at,
            )
        })
        .collect();
    let policies_after: Vec<_> = s
        .policy_query_all()
        .unwrap()
        .into_iter()
        .map(|item| (item.policy_key, item.enabled, item.updated_at))
        .collect();
    let blacklist_after: Vec<_> = s
        .blacklist_query_all()
        .unwrap()
        .into_iter()
        .map(|item| {
            (
                item.id,
                item.extension,
                item.description,
                item.is_default,
                item.created_at,
            )
        })
        .collect();
    assert_eq!(whitelist_after, original_whitelist);
    assert_eq!(policies_after, original_policies);
    assert_eq!(blacklist_after, original_blacklist);
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
    assert_eq!(c.config_value.as_deref(), Some("(AD USB protection dev)USB Device"));

    s.config_set("device_description", "NewDesc").unwrap();
    let c = s.config_get("device_description").unwrap().unwrap();
    assert_eq!(c.config_value.as_deref(), Some("NewDesc"));
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
