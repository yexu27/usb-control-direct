use std::sync::{Arc, Barrier};

use storage::model::OperationLogInsert;
use storage::{InsertOnceResult, Storage, StorageError};
use storage_test_support::TestDb;

fn item(request_id: Option<&str>) -> OperationLogInsert {
    OperationLogInsert {
        op_time: 1_700_000_000,
        username: "admin".into(),
        role: 1,
        log_type: "program_upgrade".into(),
        action_type: Some("system_upgrade".into()),
        target: Some("3.0.2".into()),
        before_value: Some("3.0.1".into()),
        after_value: Some("3.0.2".into()),
        related_file: None,
        related_version: Some("3.0.2".into()),
        result: 0,
        fail_reason: None,
        source_ip: Some("127.0.0.1".into()),
        app_version: Some("3.0.2".into()),
        session_id: None,
        request_id: request_id.map(str::to_owned),
        detail: Some("系统升级成功".into()),
    }
}

#[test]
fn first_insert_and_repeat_return_the_same_row() {
    let db = TestDb::new();
    let storage = Storage::open(db.path()).unwrap();
    let first = storage
        .operation_log_insert_once_by_request_id(&item(Some("upgrade:u1:result")))
        .unwrap();
    let repeated = storage
        .operation_log_insert_once_by_request_id(&item(Some("upgrade:u1:result")))
        .unwrap();
    let InsertOnceResult::Inserted(first_id) = first else {
        panic!("first call must insert");
    };
    assert_eq!(repeated, InsertOnceResult::AlreadyExists(first_id));
    assert_eq!(storage.operation_log_count().unwrap(), 1);
}

#[test]
fn request_id_collision_with_different_identity_is_rejected() {
    let db = TestDb::new();
    let storage = Storage::open(db.path()).unwrap();
    let request_id = "upgrade:u2:result";
    storage
        .operation_log_insert_once_by_request_id(&item(Some(request_id)))
        .unwrap();
    let mut conflicting = item(Some(request_id));
    conflicting.target = Some("3.0.3".into());
    assert!(matches!(
        storage.operation_log_insert_once_by_request_id(&conflicting),
        Err(StorageError::Validation(_))
    ));
}

#[test]
fn empty_or_missing_request_id_is_rejected() {
    let db = TestDb::new();
    let storage = Storage::open(db.path()).unwrap();
    for invalid in [None, Some(""), Some("   ")] {
        assert!(matches!(
            storage.operation_log_insert_once_by_request_id(&item(invalid)),
            Err(StorageError::Validation(_))
        ));
    }
}

#[test]
fn concurrent_insert_creates_one_business_log() {
    let db = TestDb::new();
    let path = db.path().to_path_buf();
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let path = path.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            let storage = Storage::open(&path).unwrap();
            barrier.wait();
            storage
                .operation_log_insert_once_by_request_id(&item(Some("upgrade:u3:result")))
                .unwrap()
        }));
    }
    barrier.wait();
    let mut results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    results.sort_by_key(|result| match result {
        InsertOnceResult::Inserted(_) => 0,
        InsertOnceResult::AlreadyExists(_) => 1,
    });
    assert!(matches!(results[0], InsertOnceResult::Inserted(_)));
    assert!(matches!(results[1], InsertOnceResult::AlreadyExists(_)));
    let storage = Storage::open(&path).unwrap();
    assert_eq!(storage.operation_log_count().unwrap(), 1);
}
