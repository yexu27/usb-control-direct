use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use system_upgrade::{DebInspector, DebMetadata, SystemVersion, UpgradeError, UpgradeManifest};
use usb_control_release_tool::{
    build_bin_with_inspector, generate_key, verify_bin_with_inspector, BuildBinRequest,
};

#[test]
fn build_bin_is_deterministic_bounded_and_accepted_by_server_verifier() {
    let fixture = Fixture::new();

    let manifest = fixture.build().unwrap();

    assert_eq!(manifest.schema_to, 2);
    let file = fs::File::open(&fixture.bin).unwrap();
    let mut archive = tar::Archive::new(file);
    let mut names = Vec::new();
    let mut embedded_deb = None;
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let header = entry.header();
        assert_eq!(header.uid().unwrap(), 0);
        assert_eq!(header.gid().unwrap(), 0);
        assert_eq!(header.mtime().unwrap(), 0);
        let name = entry.path().unwrap().to_string_lossy().into_owned();
        if name.ends_with(".deb") {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            embedded_deb = Some(bytes);
        }
        names.push(name);
    }
    assert_eq!(
        names,
        [
            "manifest.json",
            "usb-control_V3.0.2_arm64.deb",
            "signature.sm2"
        ]
    );
    assert_eq!(embedded_deb.unwrap(), fixture.deb_bytes);
    let verified =
        verify_bin_with_inspector(&fixture.bin, &fixture.keys, Arc::new(FakeInspector)).unwrap();
    assert_eq!(verified.package_version, manifest.package_version);
    assert_eq!(verified.schema_to, manifest.schema_to);
    assert_eq!(verified.signing_key_id, manifest.signing_key_id);
}

#[test]
fn private_key_is_never_present_in_bin_bytes_or_tar_entries() {
    let fixture = Fixture::new();
    fixture.build().unwrap();
    let private = fs::read(fixture.keys.join("upgrade_sign.key")).unwrap();
    let private = trim_ascii(&private);
    let package = fs::read(&fixture.bin).unwrap();

    assert!(!package
        .windows(private.len())
        .any(|window| window == private));
    let names = tar::Archive::new(package.as_slice())
        .entries()
        .unwrap()
        .map(|entry| entry.unwrap().path().unwrap().into_owned())
        .collect::<Vec<_>>();
    assert!(!names
        .iter()
        .any(|path| path.to_string_lossy().contains("key")));
}

#[test]
fn tampered_manifest_deb_or_signature_is_rejected() {
    for target in [
        "manifest.json",
        "usb-control_V3.0.2_arm64.deb",
        "signature.sm2",
    ] {
        let fixture = Fixture::new();
        fixture.build().unwrap();
        let file = fs::File::open(&fixture.bin).unwrap();
        let mut entries = Vec::new();
        for entry in tar::Archive::new(file).entries().unwrap() {
            let mut entry = entry.unwrap();
            let name = entry.path().unwrap().to_string_lossy().into_owned();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            if name == target {
                bytes[0] ^= 1;
            }
            entries.push((name, bytes));
        }
        let tampered = fixture._temp.path().join("tampered.bin");
        let mut output = fs::File::create(&tampered).unwrap();
        {
            let mut builder = tar::Builder::new(&mut output);
            for (name, bytes) in entries {
                let mut header = tar::Header::new_ustar();
                header.set_mode(0o644);
                header.set_uid(0);
                header.set_gid(0);
                header.set_mtime(0);
                header.set_size(bytes.len() as u64);
                header.set_cksum();
                builder
                    .append_data(&mut header, name, bytes.as_slice())
                    .unwrap();
            }
            builder.finish().unwrap();
        }
        assert!(
            verify_bin_with_inspector(&tampered, &fixture.keys, Arc::new(FakeInspector)).is_err()
        );
    }
}

struct Fixture {
    _temp: tempfile::TempDir,
    keys: PathBuf,
    deb: PathBuf,
    bin: PathBuf,
    deb_bytes: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let keys = temp.path().join("keys");
        generate_key("upgrade-test-01", &keys).unwrap();
        let deb = temp.path().join("usb-control_V3.0.2_arm64.deb");
        let deb_bytes = b"fixed-release-tool-deb-bytes".to_vec();
        fs::write(&deb, &deb_bytes).unwrap();
        let bin = temp.path().join("usb-control_V3.0.2_arm64.bin");
        Self {
            _temp: temp,
            keys,
            deb,
            bin,
            deb_bytes,
        }
    }

    fn build(&self) -> Result<UpgradeManifest, usb_control_release_tool::ReleaseToolError> {
        build_bin_with_inspector(
            BuildBinRequest {
                deb_path: &self.deb,
                key_dir: &self.keys,
                output_path: &self.bin,
            },
            Arc::new(FakeInspector),
        )
    }
}

struct FakeInspector;

impl DebInspector for FakeInspector {
    fn inspect(&self, _deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        Ok(DebMetadata {
            package: "usb-control".into(),
            version: SystemVersion::parse("3.0.2")?,
            architecture: "arm64".into(),
            expanded_size: 4096,
            files: BTreeSet::new(),
            tls_cert_sha256: "a".repeat(64),
            supported_schema_min: 1,
            supported_schema_max: 2,
            migration_schema_to: 2,
            upgrade_signing_key_id: "upgrade-test-01".into(),
        })
    }
}

fn trim_ascii(mut value: &[u8]) -> &[u8] {
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}
