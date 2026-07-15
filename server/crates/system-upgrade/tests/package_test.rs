mod support;

use std::path::Path;
use std::sync::Arc;

use serde_json::json;
use support::{
    manifest_json, raw_container, MatchingDebInspector, PackageFixture, TarEntry, DEB_NAME,
};
use system_upgrade::{
    DebInspector, DebMetadata, PackageStager, PackageVerifier, SystemVersion, UpgradeError,
};
use tar::EntryType;

fn verify(fixture: &PackageFixture) -> Result<(), system_upgrade::UpgradeError> {
    let staged = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
        .stage("upgrade-1", &fixture.package_bytes)?;
    PackageVerifier::new(fixture.key_dir(), Arc::new(MatchingDebInspector))
        .verify(staged, &fixture.context())?;
    Ok(())
}

#[test]
fn accepts_exact_three_file_signed_container() {
    let fixture = PackageFixture::valid();
    verify(&fixture).expect("valid signed upgrade package must be accepted");

    let staged_root = fixture.root().join("staging/upgrade-1");
    assert!(staged_root.join("package.bin").is_file());
    assert!(staged_root.join("manifest.json").is_file());
    assert!(staged_root.join("payload.deb").is_file());
    assert!(staged_root.join("signature.sm2").is_file());
}

#[test]
fn reopens_staged_package_by_reparsing_container_and_binding_package_digest() {
    let fixture = PackageFixture::valid();
    let stager = PackageStager::new(fixture.root(), 128 * 1024 * 1024);
    stager
        .stage("reopen", &fixture.package_bytes)
        .expect("stage valid package");
    let reopened = stager
        .reopen("reopen", &support::sha256_hex(&fixture.package_bytes))
        .expect("reopen valid staged package");
    assert_eq!(reopened.manifest_raw, fixture.manifest_raw);
    assert_eq!(std::fs::read(reopened.deb_path).unwrap(), fixture.deb_bytes);

    assert!(stager.reopen("reopen", &"0".repeat(64)).is_err());
    std::fs::write(fixture.root().join("staging/reopen/unexpected"), b"extra").unwrap();
    assert!(stager
        .reopen("reopen", &support::sha256_hex(&fixture.package_bytes))
        .is_err());
}

#[test]
fn reopen_reuses_manifest_and_signature_size_limits() {
    let fixture = PackageFixture::valid();
    let stager = PackageStager::new(fixture.root(), 128 * 1024 * 1024);
    stager
        .stage("reopen-limits", &fixture.package_bytes)
        .expect("stage valid package");

    for oversized_entry in [
        TarEntry::regular("manifest.json", &vec![b' '; 64 * 1024 + 1]),
        TarEntry::regular("signature.sm2", &vec![0; 16 * 1024 + 1]),
    ] {
        let entries = [
            if oversized_entry.path == "manifest.json" {
                oversized_entry.clone()
            } else {
                TarEntry::regular("manifest.json", &fixture.manifest_raw)
            },
            TarEntry::regular(DEB_NAME, &fixture.deb_bytes),
            if oversized_entry.path == "signature.sm2" {
                oversized_entry
            } else {
                TarEntry::regular("signature.sm2", b"signature")
            },
        ];
        let package = raw_container(&entries);
        std::fs::write(
            fixture.root().join("staging/reopen-limits/package.bin"),
            &package,
        )
        .unwrap();
        assert!(stager
            .reopen("reopen-limits", &support::sha256_hex(&package))
            .is_err());
    }
}

#[test]
fn rejects_extra_duplicate_and_non_regular_entries() {
    let cases = [
        vec![TarEntry::regular("unexpected.txt", b"extra")],
        vec![TarEntry::regular("manifest.json", b"duplicate")],
        vec![TarEntry::special("directory", EntryType::Directory, None)],
    ];

    for (index, extra_entries) in cases.iter().enumerate() {
        let mut fixture = PackageFixture::valid();
        fixture.resign_with_entries(extra_entries);
        let result = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
            .stage(&format!("invalid-{index}"), &fixture.package_bytes);
        assert!(result.is_err(), "case {index} must be rejected");
    }
}

#[test]
fn rejects_absolute_parent_symlink_hardlink_and_device_entries() {
    let fixture = PackageFixture::valid();
    let base_entries = || {
        vec![
            TarEntry::regular("manifest.json", &fixture.manifest_raw),
            TarEntry::regular(DEB_NAME, &fixture.deb_bytes),
            TarEntry::regular("signature.sm2", b"signature"),
        ]
    };
    let invalid_entries = [
        TarEntry::regular("/absolute", b"bad"),
        TarEntry::regular("../parent", b"bad"),
        TarEntry::special("link", EntryType::Symlink, Some("manifest.json")),
        TarEntry::special("hardlink", EntryType::Link, Some("manifest.json")),
        TarEntry::special("device", EntryType::Char, None),
    ];

    for (index, invalid_entry) in invalid_entries.into_iter().enumerate() {
        let mut entries = base_entries();
        entries.push(invalid_entry);
        let package = raw_container(&entries);
        let result = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
            .stage(&format!("unsafe-{index}"), &package);
        assert!(result.is_err(), "unsafe tar case {index} must be rejected");
    }
}

#[test]
fn rejects_manifest_unknown_field_wrong_product_and_architecture() {
    for overrides in [
        json!({"unknown": true}),
        json!({"product": "another-product"}),
        json!({"architecture": "amd64"}),
    ] {
        let mut fixture = PackageFixture::valid();
        let manifest = manifest_json(&fixture.deb_bytes, overrides);
        fixture.rebuild(manifest, fixture.deb_bytes.clone());
        assert!(verify(&fixture).is_err());
    }
}

#[test]
fn rejects_tampered_manifest_deb_digest_and_signature() {
    let mut manifest_tampered = PackageFixture::valid();
    manifest_tampered.package_bytes[512 + 1] ^= 1;
    assert!(verify(&manifest_tampered).is_err());

    let mut deb_tampered = PackageFixture::valid();
    let manifest = deb_tampered.manifest_raw.clone();
    let mut deb = deb_tampered.deb_bytes.clone();
    deb[0] ^= 1;
    deb_tampered.rebuild(manifest, deb);
    assert!(verify(&deb_tampered).is_err());

    let mut digest_mismatch = PackageFixture::valid();
    let manifest = manifest_json(
        &digest_mismatch.deb_bytes,
        json!({"deb_sha256": "00".repeat(32)}),
    );
    digest_mismatch.rebuild(manifest, digest_mismatch.deb_bytes.clone());
    assert!(verify(&digest_mismatch).is_err());

    let mut signature_tampered = PackageFixture::valid();
    signature_tampered.tamper_entry("signature.sm2");
    assert!(verify(&signature_tampered).is_err());
}

#[test]
fn rejects_client_fields_that_disagree_with_manifest() {
    let fixture = PackageFixture::valid();
    let staged = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
        .stage("client-mismatch", &fixture.package_bytes)
        .expect("stage valid package");
    let verifier = PackageVerifier::new(fixture.key_dir(), Arc::new(MatchingDebInspector));

    let mut wrong_version = fixture.context();
    wrong_version.client_target_version = "3.2.0".to_string();
    assert!(verifier.verify(staged, &wrong_version).is_err());

    let staged = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
        .stage("client-digest-mismatch", &fixture.package_bytes)
        .expect("stage valid package");
    let mut wrong_digest = fixture.context();
    wrong_digest.client_sha256 = "00".repeat(32);
    assert!(verifier.verify(staged, &wrong_digest).is_err());

    let staged = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
        .stage("client-inner-deb-digest", &fixture.package_bytes)
        .expect("stage valid package");
    let mut inner_deb_digest = fixture.context();
    inner_deb_digest.client_sha256 = support::sha256_hex(&fixture.deb_bytes);
    assert!(verifier.verify(staged, &inner_deb_digest).is_err());
}

#[test]
fn accepts_bare_or_single_case_insensitive_v_client_version_only() {
    for target in ["3.1.0", "v3.1.0", "V3.1.0"] {
        let fixture = PackageFixture::valid();
        let staged = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
            .stage("accepted-version", &fixture.package_bytes)
            .expect("stage valid package");
        let mut context = fixture.context();
        context.client_target_version = target.to_string();
        PackageVerifier::new(fixture.key_dir(), Arc::new(MatchingDebInspector))
            .verify(staged, &context)
            .expect("compatible client version must be accepted");
    }

    for target in ["vv3.1.0", "VV3.1.0", "vV3.1.0"] {
        let fixture = PackageFixture::valid();
        let staged = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
            .stage("rejected-version", &fixture.package_bytes)
            .expect("stage valid package");
        let mut context = fixture.context();
        context.client_target_version = target.to_string();
        assert!(
            PackageVerifier::new(fixture.key_dir(), Arc::new(MatchingDebInspector))
                .verify(staged, &context)
                .is_err()
        );
    }
}

struct MigrationMismatchInspector;

impl DebInspector for MigrationMismatchInspector {
    fn inspect(&self, _deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        Ok(DebMetadata {
            package: "usb-control".into(),
            version: SystemVersion::parse("3.1.0")?,
            architecture: "arm64".into(),
            expanded_size: 4096,
            files: support::required_deb_files(),
            tls_cert_sha256: support::TLS_SHA256.into(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            migration_schema_to: 1,
            upgrade_signing_key_id: support::KEY_ID.into(),
        })
    }
}

#[test]
fn rejects_deb_migration_target_that_disagrees_with_manifest() {
    let fixture = PackageFixture::valid();
    let staged = PackageStager::new(fixture.root(), 128 * 1024 * 1024)
        .stage("migration-mismatch", &fixture.package_bytes)
        .expect("stage valid package");
    assert!(
        PackageVerifier::new(fixture.key_dir(), Arc::new(MigrationMismatchInspector))
            .verify(staged, &fixture.context())
            .is_err()
    );
}
