mod support;

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;

use sha2::Digest;
use system_upgrade::{
    certificate_sha256, read_active_release, read_last_known_good, read_upgrade_result,
    ActiveCommitError, ActiveRelease, DirectorySync, LastKnownGoodRelease, ReleaseStateStore,
    SystemVersion, UpgradeResult, UpgradeStatus, UpgradeTask,
};

use support::test_certificate_pem;

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

struct FailDirectorySync;

impl DirectorySync for FailDirectorySync {
    fn sync(&self, _path: &Path) -> io::Result<()> {
        Err(io::Error::other("injected parent fsync failure"))
    }
}

#[test]
fn active_rename_success_is_distinguished_from_parent_fsync_failure() {
    let dir = tempfile::tempdir().unwrap();
    let store = ReleaseStateStore::with_directory_sync(
        dir.path().to_path_buf(),
        Arc::new(FailDirectorySync),
    )
    .unwrap();
    let error = store.commit_active_release(&active()).unwrap_err();
    assert!(matches!(error, ActiveCommitError::AfterRename(_)));
    assert_eq!(store.active_release().unwrap(), Some(active()));
}

fn active() -> ActiveRelease {
    ActiveRelease {
        format_version: 1,
        upgrade_id: "upgrade-store".into(),
        version: version("3.0.2"),
        deb_sha256: "a".repeat(64),
        schema_version: 2,
        committed_at: 200,
    }
}

fn result() -> UpgradeResult {
    UpgradeResult {
        format_version: 1,
        upgrade_id: "upgrade-store".into(),
        status: UpgradeStatus::Committed,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        effective_version: version("3.0.2"),
        failed_stage: None,
        original_error: None,
        rollback_error: None,
        finished_at: 200,
    }
}

fn task() -> UpgradeTask {
    UpgradeTask {
        format_version: 1,
        upgrade_id: "upgrade-store".into(),
        status: UpgradeStatus::HealthChecking,
        username: "admin".into(),
        role: 1,
        source_ip: "127.0.0.1".into(),
        source_version: version("3.0.1"),
        target_version: version("3.0.2"),
        package_sha256: "b".repeat(64),
        created_at: 100,
        updated_at: 190,
    }
}

#[test]
fn active_release_and_result_use_shared_strict_atomic_store() {
    let dir = tempfile::tempdir().unwrap();
    let store = ReleaseStateStore::new(dir.path().to_path_buf()).unwrap();
    store.commit_active_release(&active()).unwrap();
    store.write_result(&result()).unwrap();
    assert_eq!(store.active_release().unwrap(), Some(active()));
    assert_eq!(store.result("upgrade-store").unwrap(), Some(result()));
}

#[test]
fn rejects_unknown_format_and_unknown_fields() {
    let dir = tempfile::tempdir().unwrap();
    let store = ReleaseStateStore::new(dir.path().to_path_buf()).unwrap();
    fs::write(
        dir.path().join("active-release.json"),
        r#"{"format_version":2,"upgrade_id":"upgrade-store","version":"3.0.2","deb_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","schema_version":2,"committed_at":200}"#,
    ).unwrap();
    assert!(store.active_release().is_err());
    fs::write(
        dir.path().join("active-release.json"),
        r#"{"format_version":1,"upgrade_id":"upgrade-store","version":"3.0.2","deb_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","schema_version":2,"committed_at":200,"extra":true}"#,
    ).unwrap();
    assert!(store.active_release().is_err());
}

#[test]
fn strict_readers_reject_nonterminal_and_oversized_results() {
    let dir = tempfile::tempdir().unwrap();
    let active_path = dir.path().join("active.json");
    let result_path = dir.path().join("result.json");
    fs::write(&active_path, serde_json::to_vec(&active()).unwrap()).unwrap();
    fs::write(&result_path, serde_json::to_vec(&result()).unwrap()).unwrap();
    assert_eq!(read_active_release(&active_path).unwrap(), active());
    assert_eq!(read_upgrade_result(&result_path).unwrap(), result());

    let mut nonterminal = result();
    nonterminal.status = UpgradeStatus::Installing;
    fs::write(&result_path, serde_json::to_vec(&nonterminal).unwrap()).unwrap();
    assert!(read_upgrade_result(&result_path).is_err());

    fs::write(&result_path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(read_upgrade_result(&result_path).is_err());
}

#[test]
fn only_service_runnable_terminal_results_are_business_log_importable() {
    for status in [
        UpgradeStatus::Committed,
        UpgradeStatus::RolledBack,
        UpgradeStatus::ScheduleFailed,
    ] {
        let mut value = result();
        value.status = status;
        assert!(value.is_business_log_importable());
    }
    let mut rollback_failed = result();
    rollback_failed.status = UpgradeStatus::RollbackFailed;
    assert!(!rollback_failed.is_business_log_importable());
}

#[test]
fn committed_result_is_reconstructed_only_for_matching_active_release() {
    let reconstructed = UpgradeResult::committed_from_active(&task(), &active(), 210).unwrap();
    assert_eq!(reconstructed.status, UpgradeStatus::Committed);
    assert_eq!(reconstructed.effective_version, version("3.0.2"));
    assert_eq!(reconstructed.finished_at, 210);

    let mut wrong_id = active();
    wrong_id.upgrade_id = "another-upgrade".into();
    assert!(UpgradeResult::committed_from_active(&task(), &wrong_id, 210).is_err());
    let mut wrong_version = active();
    wrong_version.version = version("3.0.3");
    assert!(UpgradeResult::committed_from_active(&task(), &wrong_version, 210).is_err());
}

#[test]
fn last_known_good_reader_is_strict_and_verifies_deb_digest() {
    let dir = tempfile::tempdir().unwrap();
    let metadata_path = dir.path().join("last-known-good.json");
    let deb_path = dir.path().join("last-known-good.deb");
    fs::write(&deb_path, b"known-good-deb").unwrap();
    let expected = LastKnownGoodRelease {
        format_version: 1,
        version: version("3.0.1"),
        deb_sha256: hex::encode(sha2::Sha256::digest(b"known-good-deb")),
        schema_version: 1,
        tls_cert_sha256: "a".repeat(64),
    };
    fs::write(&metadata_path, serde_json::to_vec(&expected).unwrap()).unwrap();

    assert_eq!(
        read_last_known_good(&metadata_path, &deb_path).unwrap(),
        expected
    );

    let mut unknown = serde_json::to_value(&expected).unwrap();
    unknown["unexpected"] = serde_json::json!(true);
    fs::write(&metadata_path, serde_json::to_vec(&unknown).unwrap()).unwrap();
    assert!(read_last_known_good(&metadata_path, &deb_path).is_err());

    fs::write(&metadata_path, serde_json::to_vec(&expected).unwrap()).unwrap();
    fs::write(&deb_path, b"tampered-deb").unwrap();
    assert!(read_last_known_good(&metadata_path, &deb_path).is_err());
}

#[test]
fn shared_certificate_hash_accepts_exactly_one_pem_certificate() {
    let hash = certificate_sha256(test_certificate_pem()).unwrap();
    assert_eq!(hash.len(), 64);
    assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));

    let mut duplicate = test_certificate_pem().to_vec();
    duplicate.extend_from_slice(test_certificate_pem());
    assert!(certificate_sha256(&duplicate).is_err());
    assert!(certificate_sha256(b"not a certificate").is_err());
}
