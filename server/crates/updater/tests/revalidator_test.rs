mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;
use sha2::{Digest, Sha256};
use smcrypto::{sm2, sm3};
use system_upgrade::{
    certificate_sha256, ActiveRelease, ActiveReleaseStore, InstalledRelease, PackageStager,
    SystemVersion, UpgradeStateLock, UpgradeStatus, UpgradeTask,
};
use tar::{Builder, Header};
use usb_control_updater::{PackageRevalidator, SharedPackageRevalidator, UpgradePaths};

use support::TEST_CERTIFICATE_PEM;

const CURRENT_KEY_ID: &str = "upgrade-current";
const TARGET_KEY_ID: &str = "upgrade-next";
const UPGRADE_ID: &str = "upgrade-real-revalidation";

#[test]
fn production_revalidator_accepts_real_signed_deb() {
    let fixture = Fixture::new(false);

    let revalidated = fixture.revalidate().unwrap();

    assert_eq!(revalidated.manifest.package_version, version("3.0.2"));
    assert_eq!(revalidated.target_release.version, version("3.0.2"));
    assert_eq!(
        revalidated.target_release.upgrade_signing_key_id,
        TARGET_KEY_ID
    );
}

#[test]
fn production_revalidator_rejects_installed_release_mismatch() {
    let fixture = Fixture::new(false);
    fixture.write_installed(version("3.0.0"), CURRENT_KEY_ID);

    assert!(fixture.revalidate().is_err());
}

#[test]
fn production_revalidator_rejects_active_release_mismatch() {
    let fixture = Fixture::new(false);
    fixture.write_active(version("3.0.0"));

    assert!(fixture.revalidate().is_err());
}

#[test]
fn production_revalidator_rejects_active_key_id_mismatch() {
    let fixture = Fixture::new(false);
    fs::write(&fixture.active_key_id, "upgrade-other\n").unwrap();

    assert!(fixture.revalidate().is_err());
}

#[test]
fn production_revalidator_rejects_signature_from_untrusted_key() {
    let fixture = Fixture::new(true);

    assert!(fixture.revalidate().is_err());
}

struct Fixture {
    _temp: tempfile::TempDir,
    paths: UpgradePaths,
    verify_key_dir: PathBuf,
    installed_release: PathBuf,
    active_key_id: PathBuf,
    task: UpgradeTask,
    tls_sha256: String,
}

impl Fixture {
    fn new(sign_with_untrusted_key: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("upgrade");
        let paths = UpgradePaths::for_root(root.clone());
        let verify_key_dir = temp.path().join("current-keys");
        let installed_release = temp.path().join("installed-release.json");
        let active_key_id = temp.path().join("active-key.id");
        fs::create_dir_all(&verify_key_dir).unwrap();

        let (current_private_key, current_public_key) = sm2::gen_keypair();
        let (untrusted_private_key, _) = sm2::gen_keypair();
        let (_, target_public_key) = sm2::gen_keypair();
        fs::write(
            verify_key_dir.join("upgrade_verify.id"),
            format!("{CURRENT_KEY_ID}\n"),
        )
        .unwrap();
        fs::write(
            verify_key_dir.join("upgrade_verify.pub"),
            format!("{current_public_key}\n"),
        )
        .unwrap();
        fs::write(&active_key_id, format!("{CURRENT_KEY_ID}\n")).unwrap();

        let tls_sha256 = certificate_sha256(TEST_CERTIFICATE_PEM.as_bytes()).unwrap();
        let deb = build_real_deb(temp.path(), &target_public_key, &tls_sha256);
        let package = signed_package(
            if sign_with_untrusted_key {
                &untrusted_private_key
            } else {
                &current_private_key
            },
            &deb,
            &tls_sha256,
        );
        let package_sha256 = hex::encode(Sha256::digest(&package));
        PackageStager::new(root.clone(), 128 * 1024 * 1024)
            .stage(UPGRADE_ID, &package)
            .unwrap();

        let fixture = Self {
            _temp: temp,
            paths,
            verify_key_dir,
            installed_release,
            active_key_id,
            task: UpgradeTask {
                format_version: 1,
                upgrade_id: UPGRADE_ID.into(),
                status: UpgradeStatus::Accepted,
                username: "admin".into(),
                role: 1,
                source_ip: "127.0.0.1".into(),
                source_version: version("3.0.1"),
                target_version: version("3.0.2"),
                package_sha256,
                created_at: 100,
                updated_at: 101,
            },
            tls_sha256,
        };
        fixture.write_installed(version("3.0.1"), CURRENT_KEY_ID);
        fixture.write_active(version("3.0.1"));
        fixture
    }

    fn revalidate(
        &self,
    ) -> Result<usb_control_updater::RevalidatedPackage, usb_control_updater::UpdaterError> {
        SharedPackageRevalidator::new(
            self.verify_key_dir.clone(),
            self.installed_release.clone(),
            self.active_key_id.clone(),
        )
        .revalidate(&self.paths, &self.task)
    }

    fn write_installed(&self, release_version: SystemVersion, key_id: &str) {
        let release = InstalledRelease {
            format_version: 1,
            product: "usb-control".into(),
            version: release_version,
            architecture: "arm64".into(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            tls_cert_sha256: self.tls_sha256.clone(),
            upgrade_signing_key_id: key_id.into(),
        };
        fs::write(
            &self.installed_release,
            serde_json::to_vec(&release).unwrap(),
        )
        .unwrap();
    }

    fn write_active(&self, release_version: SystemVersion) {
        let store = ActiveReleaseStore::new(self.paths.root.clone()).unwrap();
        let guard = UpgradeStateLock::acquire(&self.paths.root).unwrap();
        store
            .commit(
                &guard,
                &ActiveRelease {
                    format_version: 1,
                    version: release_version,
                    schema_version: 1,
                    committed_at: 100,
                    online_upgrade_id: None,
                },
            )
            .unwrap();
    }
}

fn build_real_deb(root: &Path, target_public_key: &str, tls_sha256: &str) -> Vec<u8> {
    let package_root = root.join("deb-root");
    write(
        &package_root,
        "DEBIAN/control",
        b"Package: usb-control\nVersion: 3.0.2\nArchitecture: arm64\nMaintainer: Test <test@example.invalid>\nDescription: revalidator fixture\n",
    );
    for binary in [
        "usb-control",
        "usb-control-updater",
        "usb-control-db-migrate",
    ] {
        write(
            &package_root,
            &format!("opt/usb-control/bin/{binary}"),
            b"binary",
        );
    }
    write(
        &package_root,
        "lib/systemd/system/usb-control.service",
        b"[Service]\n",
    );
    write(
        &package_root,
        "lib/systemd/system/usb-control-updater.service",
        b"[Service]\n",
    );
    write(
        &package_root,
        "opt/usb-control/defaults/etc/usb-control/usb-control.toml",
        b"[server]\n",
    );
    write(
        &package_root,
        "opt/usb-control/defaults/etc/usb-control/tls/server.crt",
        TEST_CERTIFICATE_PEM.as_bytes(),
    );
    write(
        &package_root,
        "opt/usb-control/defaults/etc/usb-control/tls/server.key",
        b"tls-private-key",
    );
    write(
        &package_root,
        "opt/usb-control/defaults/etc/usb-control/tls/server.crt.sha256",
        format!("{tls_sha256}\n").as_bytes(),
    );
    for (name, contents) in [
        ("license_verify.pub", "license-public-key"),
        ("sm4_policy.key", "sm4-policy-key"),
        ("sm2_policy.key", "sm2-policy-private-key"),
        ("sm2_policy.pub", "sm2-policy-public-key"),
    ] {
        write(
            &package_root,
            &format!("opt/usb-control/defaults/etc/usb-control/keys/{name}"),
            contents.as_bytes(),
        );
    }
    write(
        &package_root,
        "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.id",
        format!("{TARGET_KEY_ID}\n").as_bytes(),
    );
    write(
        &package_root,
        "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.pub",
        format!("{target_public_key}\n").as_bytes(),
    );
    write(
        &package_root,
        "opt/usb-control/db/migrations/0001_init.sql",
        b"SELECT 1;",
    );
    write(
        &package_root,
        "opt/usb-control/db/migrations/0002_upgrade.sql",
        b"SELECT 1;",
    );
    write(
        &package_root,
        "opt/usb-control/db/seeds/0001_default.sql",
        b"SELECT 1;",
    );
    let release = json!({
        "format_version": 1,
        "product": "usb-control",
        "version": "3.0.2",
        "architecture": "arm64",
        "supported_schema_min": 1,
        "supported_schema_max": 2,
        "tls_cert_sha256": tls_sha256,
        "upgrade_signing_key_id": TARGET_KEY_ID
    });
    write(
        &package_root,
        "opt/usb-control/install-meta/release.json",
        &serde_json::to_vec(&release).unwrap(),
    );

    let deb_path = root.join("usb-control_V3.0.2_arm64.deb");
    let output = Command::new("dpkg-deb")
        .args(["--build", "--root-owner-group"])
        .arg(&package_root)
        .arg(&deb_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "dpkg-deb failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read(deb_path).unwrap()
}

fn signed_package(private_key: &str, deb: &[u8], tls_sha256: &str) -> Vec<u8> {
    let manifest = serde_json::to_vec(&json!({
        "format_version": 1,
        "product": "usb-control",
        "package_version": "3.0.2",
        "architecture": "arm64",
        "minimum_current_version": "3.0.1",
        "protocol_version": 1,
        "tls_cert_sha256": tls_sha256,
        "deb_file": "usb-control_V3.0.2_arm64.deb",
        "deb_size": deb.len(),
        "deb_sha256": hex::encode(Sha256::digest(deb)),
        "schema_from": 1,
        "schema_to": 2,
        "signing_key_id": CURRENT_KEY_ID
    }))
    .unwrap();
    let deb_digest = Sha256::digest(deb);
    let mut signing_input = Vec::new();
    signing_input.extend_from_slice(b"USB-CONTROL-UPGRADE-V1\0");
    signing_input.extend_from_slice(&(manifest.len() as u64).to_be_bytes());
    signing_input.extend_from_slice(&manifest);
    signing_input.extend_from_slice(&deb_digest);
    let digest = hex::decode(sm3::sm3_hash(&signing_input)).unwrap();
    let signature = sm2::Sign::new(private_key).sign(&digest);

    let mut package = Vec::new();
    {
        let mut builder = Builder::new(&mut package);
        append(&mut builder, "manifest.json", &manifest);
        append(&mut builder, "usb-control_V3.0.2_arm64.deb", deb);
        append(&mut builder, "signature.sm2", &signature);
        builder.finish().unwrap();
    }
    package
}

fn append(builder: &mut Builder<&mut Vec<u8>>, path: &str, contents: &[u8]) {
    let mut header = Header::new_ustar();
    header.set_mode(0o600);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(contents.len() as u64);
    header.set_cksum();
    builder.append_data(&mut header, path, contents).unwrap();
}

fn write(root: &Path, relative: &str, contents: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}
