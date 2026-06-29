//! 日志审计集成测试。

use std::sync::Arc;

use log_audit::AuditService;
use storage::model::*;
use storage::Storage;
use storage_test_support::initialize_database;
use tempfile::NamedTempFile;

fn setup() -> (AuditService, tempfile::TempPath) {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.into_temp_path();
    initialize_database(&path);
    let storage = Arc::new(Storage::open(&path).unwrap());
    let service = AuditService::new(storage, &path);
    (service, path)
}

#[test]
fn log_usb_audit_inserts_record() {
    let (service, _path) = setup();
    let mut item = UsbAuditLogInsert {
        event_time: 0,
        device_type: Some("storage".into()),
        interface_type: Some("mass_storage".into()),
        interface_class: Some(8),
        interface_subclass: None,
        interface_protocol: None,
        device_name: Some("TestUSB".into()),
        device_sn: Some("SN001".into()),
        vid: None,
        pid: None,
        event_type: "device_insert".into(),
        permission: None,
        capacity_bytes: None,
        file_path: None,
        matched_policy: None,
        result: "allowed".into(),
        fail_reason: None,
        detail: None,
    };
    let id = service.log_usb_audit(&mut item).unwrap();
    assert!(id > 0);
    assert!(item.event_time > 0);
}

#[test]
fn log_malware_inserts_record() {
    let (service, _path) = setup();
    let mut item = MalwareLogInsert {
        scan_time: 0,
        device_sn: Some("SN001".into()),
        device_name: None,
        file_path: Some("/test.exe".into()),
        scan_result: 1,
        virus_name: Some("Eicar".into()),
        virus_db_version: None,
        process_result: Some(1),
        fail_reason: None,
        detail: None,
    };
    let id = service.log_malware(&mut item).unwrap();
    assert!(id > 0);
}

#[test]
fn log_operation_inserts_record() {
    let (service, _path) = setup();
    let mut item = OperationLogInsert {
        op_time: 0,
        username: "admin".into(),
        role: 0,
        log_type: "login_auth".into(),
        action_type: Some("login".into()),
        target: None,
        before_value: None,
        after_value: None,
        related_file: None,
        related_version: None,
        result: 0,
        fail_reason: None,
        source_ip: None,
        app_version: None,
        session_id: None,
        request_id: None,
        detail: None,
    };
    let id = service.log_operation(&mut item).unwrap();
    assert!(id > 0);
}

#[test]
fn concurrent_writes_no_loss() {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.into_temp_path();
    initialize_database(&path);
    let storage = Arc::new(Storage::open(&path).unwrap());
    let service = std::sync::Arc::new(AuditService::new(storage, &path));

    let mut handles = Vec::new();
    for i in 0..10 {
        let svc = service.clone();
        handles.push(std::thread::spawn(move || {
            let mut item = OperationLogInsert {
                op_time: 0,
                username: format!("user{}", i),
                role: 0,
                log_type: "login_auth".into(),
                action_type: Some("login".into()),
                target: None,
                before_value: None,
                after_value: None,
                related_file: None,
                related_version: None,
                result: 0,
                fail_reason: None,
                source_ip: None,
                app_version: None,
                session_id: None,
                request_id: None,
                detail: None,
            };
            svc.log_operation(&mut item).unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let count = service.storage().operation_log_count().unwrap();
    assert_eq!(count, 10);
}
