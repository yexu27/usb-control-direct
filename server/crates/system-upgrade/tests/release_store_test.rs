use std::fs;

use system_upgrade::{read_installed_release, InstalledRelease, SystemVersion};

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
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
