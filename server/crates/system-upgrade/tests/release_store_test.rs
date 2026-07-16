use std::fs;

use system_upgrade::{
    read_installed_release, ActiveRelease, ActiveReleaseStore, InstalledRelease, SystemVersion,
    UpgradeStateLock,
};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn active() -> ActiveRelease {
    ActiveRelease {
        format_version: 1,
        version: version("3.0.2"),
        schema_version: 2,
        committed_at: 200,
        online_upgrade_id: Some("upgrade-store".into()),
    }
}

fn installed() -> InstalledRelease {
    InstalledRelease {
        format_version: 1,
        product: "usb-control".into(),
        version: version("3.0.2"),
        architecture: "arm64".into(),
        supported_schema_min: 1,
        supported_schema_max: 2,
        tls_cert_sha256: "b".repeat(64),
        upgrade_signing_key_id: "upgrade-test-01".into(),
    }
}

#[test]
fn installed_release_reader_is_strict() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("release.json");
    fs::write(&path, serde_json::to_vec(&installed()).unwrap()).unwrap();
    assert_eq!(read_installed_release(&path).unwrap(), installed());

    let mut unknown = serde_json::to_value(installed()).unwrap();
    unknown["legacy"] = serde_json::json!(true);
    fs::write(&path, serde_json::to_vec(&unknown).unwrap()).unwrap();
    assert!(read_installed_release(&path).is_err());
}

#[test]
fn installed_release_reader_rejects_invalid_business_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("release.json");
    for invalid in [
        InstalledRelease {
            product: "other".into(),
            ..installed()
        },
        InstalledRelease {
            architecture: "amd64".into(),
            ..installed()
        },
        InstalledRelease {
            supported_schema_min: 3,
            ..installed()
        },
        InstalledRelease {
            tls_cert_sha256: "A".repeat(64),
            ..installed()
        },
        InstalledRelease {
            upgrade_signing_key_id: "INVALID_KEY".into(),
            ..installed()
        },
    ] {
        fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(
            read_installed_release(&path).is_err(),
            "accepted {invalid:?}"
        );
    }
}

#[test]
fn active_release_store_commits_only_valid_release() {
    let dir = tempfile::tempdir().unwrap();
    let store = ActiveReleaseStore::new(dir.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(dir.path()).unwrap();
    store.commit(&guard, &active()).unwrap();
    assert_eq!(store.current().unwrap(), Some(active()));

    let mut invalid = active();
    invalid.schema_version = 0;
    assert!(store.commit(&guard, &invalid).is_err());
    assert_eq!(store.current().unwrap(), Some(active()));
}

#[test]
fn direct_active_release_accepts_null_online_upgrade_id() {
    let dir = tempfile::tempdir().unwrap();
    let store = ActiveReleaseStore::new(dir.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(dir.path()).unwrap();
    let direct = ActiveRelease {
        online_upgrade_id: None,
        ..active()
    };

    store.commit(&guard, &direct).unwrap();

    assert_eq!(store.current().unwrap(), Some(direct));
}

#[test]
fn online_active_release_requires_valid_upgrade_id() {
    let dir = tempfile::tempdir().unwrap();
    let store = ActiveReleaseStore::new(dir.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(dir.path()).unwrap();
    let invalid = ActiveRelease {
        online_upgrade_id: Some("../escape".into()),
        ..active()
    };

    assert!(store.commit(&guard, &invalid).is_err());
}

#[test]
fn active_release_rejects_removed_upgrade_id_and_deb_sha256_fields() {
    let dir = tempfile::tempdir().unwrap();
    ActiveReleaseStore::new(dir.path().to_path_buf()).unwrap();
    let path = dir.path().join("active-release.json");
    let legacy = serde_json::json!({
        "format_version": 1,
        "upgrade_id": "upgrade-legacy",
        "version": "3.0.2",
        "deb_sha256": "a".repeat(64),
        "schema_version": 2,
        "committed_at": 200
    });
    fs::write(path, serde_json::to_vec(&legacy).unwrap()).unwrap();

    assert!(ActiveReleaseStore::new(dir.path().to_path_buf())
        .unwrap()
        .current()
        .is_err());
}
