use std::sync::Arc;
use std::thread;

use storage::Storage;
use tempfile::NamedTempFile;
use whitelist::service::{AddWhitelistRequest, WhitelistManager};
use whitelist::WhitelistError;

fn setup() -> (WhitelistManager, NamedTempFile) {
    let tmp = NamedTempFile::new().unwrap();
    let storage = Arc::new(Storage::open(tmp.path()).unwrap());
    let manager = WhitelistManager::new(storage).unwrap();
    (manager, tmp)
}

fn make_request(sn: &str) -> AddWhitelistRequest {
    AddWhitelistRequest {
        serial_number: sn.to_string(),
        vid: Some("0930".into()),
        pid: Some("6545".into()),
        device_name: Some("Kingston DataTraveler".into()),
        capacity_bytes: Some(16_000_000_000),
        device_type: "storage".into(),
        description: None,
        permission: 0,
        add_method: 0,
    }
}

#[test]
fn add_and_query_whitelist() {
    let (mgr, _tmp) = setup();
    let id = mgr.add(make_request("SN001")).unwrap();
    assert!(id > 0);

    let result = mgr.is_whitelisted("SN001");
    assert!(result.is_some());
    assert_eq!(result.unwrap().permission, 0);
}

#[test]
fn add_empty_serial_number_rejected() {
    let (mgr, _tmp) = setup();
    let err = mgr.add(make_request("")).unwrap_err();
    assert!(matches!(err, WhitelistError::SerialNumberEmpty));
}

#[test]
fn add_duplicate_serial_number_rejected() {
    let (mgr, _tmp) = setup();
    mgr.add(make_request("SN_DUP")).unwrap();
    let err = mgr.add(make_request("SN_DUP")).unwrap_err();
    assert!(matches!(err, WhitelistError::AlreadyExists(_)));
}

#[test]
fn concurrent_add_same_sn_only_one_succeeds() {
    let tmp = NamedTempFile::new().unwrap();
    let storage = Arc::new(Storage::open(tmp.path()).unwrap());
    let mgr = Arc::new(WhitelistManager::new(storage).unwrap());

    let mut handles = vec![];
    for _ in 0..10 {
        let mgr = Arc::clone(&mgr);
        handles.push(thread::spawn(move || mgr.add(make_request("RACE_SN"))));
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 1, "并发添加同一序列号，只应有 1 次成功");
}

#[test]
fn remove_by_serial_number_uses_id() {
    let (mgr, _tmp) = setup();
    mgr.add(make_request("SN_REMOVE")).unwrap();

    assert!(mgr.is_whitelisted("SN_REMOVE").is_some());
    mgr.remove("SN_REMOVE").unwrap();
    assert!(mgr.is_whitelisted("SN_REMOVE").is_none());
}

#[test]
fn remove_nonexistent_returns_not_found() {
    let (mgr, _tmp) = setup();
    let err = mgr.remove("NONEXISTENT").unwrap_err();
    assert!(matches!(err, WhitelistError::NotFound(_)));
}

#[test]
fn update_permission_by_serial_number() {
    let (mgr, _tmp) = setup();
    mgr.add(make_request("SN_UPDATE")).unwrap();
    assert_eq!(mgr.is_whitelisted("SN_UPDATE").unwrap().permission, 0);

    mgr.update("SN_UPDATE", Some(1), None).unwrap();
    assert_eq!(mgr.is_whitelisted("SN_UPDATE").unwrap().permission, 1);
}

#[test]
fn update_description_only() {
    let (mgr, _tmp) = setup();
    mgr.add(make_request("SN_DESC")).unwrap();

    mgr.update("SN_DESC", None, Some("测试设备")).unwrap();

    let item = mgr.query_by_sn("SN_DESC").unwrap().unwrap();
    assert_eq!(item.description.as_deref(), Some("测试设备"));
    assert_eq!(item.permission, 0);
}

#[test]
fn update_nonexistent_returns_not_found() {
    let (mgr, _tmp) = setup();
    let err = mgr.update("NONEXISTENT", Some(1), None).unwrap_err();
    assert!(matches!(err, WhitelistError::NotFound(_)));
}

#[test]
fn cache_consistent_after_add_remove() {
    let (mgr, _tmp) = setup();

    mgr.add(make_request("SN_CACHE")).unwrap();
    assert!(mgr.is_whitelisted("SN_CACHE").is_some());

    mgr.remove("SN_CACHE").unwrap();
    assert!(mgr.is_whitelisted("SN_CACHE").is_none());

    mgr.add(make_request("SN_CACHE")).unwrap();
    assert!(mgr.is_whitelisted("SN_CACHE").is_some());
}

#[test]
fn query_all_returns_all_entries() {
    let (mgr, _tmp) = setup();
    mgr.add(make_request("SN_A")).unwrap();
    mgr.add(make_request("SN_B")).unwrap();

    let all = mgr.query_all().unwrap();
    assert_eq!(all.len(), 2);
}
