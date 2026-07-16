// 每个 integration test 会把共享辅助模块编译为独立 target，部分辅助项只由另一 target 使用。
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use smcrypto::{sm2, sm3};
use system_upgrade::{DebInspector, DebMetadata, SystemVersion, UpgradeError, VerificationContext};
use tar::{Builder, EntryType, Header};
use tempfile::TempDir;

pub const KEY_ID: &str = "upgrade-prod-01";
pub const DEB_NAME: &str = "usb-control_V3.1.0_arm64.deb";
pub const TLS_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub struct PackageFixture {
    pub temp_dir: TempDir,
    pub package_bytes: Vec<u8>,
    pub manifest_raw: Vec<u8>,
    pub deb_bytes: Vec<u8>,
    private_key: String,
}

impl PackageFixture {
    pub fn valid() -> Self {
        let temp_dir = tempfile::tempdir().expect("create temporary test directory");
        let (private_key, public_key) = sm2::gen_keypair();
        let key_dir = temp_dir.path().join("keys");
        fs::create_dir_all(&key_dir).expect("create key directory");
        fs::write(key_dir.join("upgrade_verify.id"), format!("{KEY_ID}\n"))
            .expect("write temporary upgrade key id");
        fs::write(key_dir.join("upgrade_verify.pub"), public_key)
            .expect("write temporary upgrade public key");

        let deb_bytes = b"minimal-deb-fixture-for-package-tests".to_vec();
        let manifest_raw = manifest_json(&deb_bytes, json!({}));
        let package_bytes = signed_container(&private_key, &manifest_raw, &deb_bytes, &[]);

        Self {
            temp_dir,
            package_bytes,
            manifest_raw,
            deb_bytes,
            private_key,
        }
    }

    pub fn root(&self) -> PathBuf {
        self.temp_dir.path().join("upgrade")
    }

    pub fn key_dir(&self) -> PathBuf {
        self.temp_dir.path().join("keys")
    }

    pub fn context(&self) -> VerificationContext {
        VerificationContext {
            current_version: SystemVersion::parse("3.0.1").expect("valid current version"),
            current_schema: 1,
            supported_schema_max: 2,
            protocol_version: 1,
            client_target_version: "v3.1.0".to_string(),
            client_sha256: sha256_hex(&self.package_bytes),
        }
    }

    pub fn rebuild(&mut self, manifest_raw: Vec<u8>, deb_bytes: Vec<u8>) {
        self.package_bytes = signed_container(&self.private_key, &manifest_raw, &deb_bytes, &[]);
        self.manifest_raw = manifest_raw;
        self.deb_bytes = deb_bytes;
    }

    pub fn resign_with_entries(&mut self, entries: &[TarEntry]) {
        self.package_bytes = signed_container(
            &self.private_key,
            &self.manifest_raw,
            &self.deb_bytes,
            entries,
        );
    }

    pub fn tamper_entry(&mut self, entry_name: &str) {
        let mut offset = 0usize;
        while offset + 512 <= self.package_bytes.len() {
            let header = &self.package_bytes[offset..offset + 512];
            if header.iter().all(|byte| *byte == 0) {
                break;
            }
            let name_end = header[..100]
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(100);
            let name = std::str::from_utf8(&header[..name_end]).expect("tar entry name");
            let size_text = std::str::from_utf8(&header[124..136])
                .expect("tar size")
                .trim_matches(['\0', ' ']);
            let size = usize::from_str_radix(size_text, 8).expect("tar entry size");
            if name == entry_name {
                self.package_bytes[offset + 512] ^= 1;
                return;
            }
            offset += 512 + size.div_ceil(512) * 512;
        }
        panic!("tar entry not found: {entry_name}");
    }
}

#[derive(Clone)]
pub struct TarEntry {
    pub path: String,
    pub kind: EntryType,
    pub data: Vec<u8>,
    pub link_name: Option<String>,
}

impl TarEntry {
    pub fn regular(path: &str, data: &[u8]) -> Self {
        Self {
            path: path.to_string(),
            kind: EntryType::Regular,
            data: data.to_vec(),
            link_name: None,
        }
    }

    pub fn special(path: &str, kind: EntryType, link_name: Option<&str>) -> Self {
        Self {
            path: path.to_string(),
            kind,
            data: Vec::new(),
            link_name: link_name.map(str::to_string),
        }
    }
}

pub fn manifest_json(deb_bytes: &[u8], overrides: Value) -> Vec<u8> {
    let mut value = json!({
        "format_version": 1,
        "product": "usb-control",
        "package_version": "3.1.0",
        "architecture": "arm64",
        "minimum_current_version": "3.0.0",
        "protocol_version": 1,
        "tls_cert_sha256": TLS_SHA256,
        "deb_file": DEB_NAME,
        "deb_size": deb_bytes.len(),
        "deb_sha256": sha256_hex(deb_bytes),
        "schema_from": 1,
        "schema_to": 2,
        "signing_key_id": KEY_ID
    });
    if let (Some(target), Some(source)) = (value.as_object_mut(), overrides.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    serde_json::to_vec(&value).expect("serialize manifest")
}

pub fn signed_container(
    private_key: &str,
    manifest_raw: &[u8],
    deb_bytes: &[u8],
    extra_entries: &[TarEntry],
) -> Vec<u8> {
    let deb_digest = Sha256::digest(deb_bytes);
    let mut signing_input = Vec::new();
    signing_input.extend_from_slice(b"USB-CONTROL-UPGRADE-V1\0");
    signing_input.extend_from_slice(&(manifest_raw.len() as u64).to_be_bytes());
    signing_input.extend_from_slice(manifest_raw);
    signing_input.extend_from_slice(&deb_digest);
    let sm3_hex = sm3::sm3_hash(&signing_input);
    let sm3_digest = hex::decode(sm3_hex).expect("decode SM3 digest");
    let signature = sm2::Sign::new(private_key).sign(&sm3_digest);

    let mut output = Vec::new();
    {
        let mut builder = Builder::new(&mut output);
        builder.mode(tar::HeaderMode::Deterministic);
        append_regular(&mut builder, "manifest.json", manifest_raw);
        append_regular(&mut builder, DEB_NAME, deb_bytes);
        append_regular(&mut builder, "signature.sm2", &signature);
        for entry in extra_entries {
            append_entry(&mut builder, entry);
        }
        builder.finish().expect("finish deterministic package tar");
    }
    output
}

pub fn raw_container(entries: &[TarEntry]) -> Vec<u8> {
    let mut output = Vec::new();
    {
        let mut builder = Builder::new(&mut output);
        builder.mode(tar::HeaderMode::Deterministic);
        for entry in entries {
            append_entry(&mut builder, entry);
        }
        builder.finish().expect("finish package tar");
    }
    output
}

fn append_regular(builder: &mut Builder<&mut Vec<u8>>, path: &str, data: &[u8]) {
    append_entry(builder, &TarEntry::regular(path, data));
}

fn append_entry(builder: &mut Builder<&mut Vec<u8>>, entry: &TarEntry) {
    let mut header = Header::new_ustar();
    header.set_entry_type(entry.kind);
    header.set_mode(0o600);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(entry.data.len() as u64);
    if let Some(link_name) = &entry.link_name {
        header.set_link_name(link_name).expect("set tar link name");
    }
    let name = entry.path.as_bytes();
    assert!(name.len() <= 100, "test tar entry name is too long");
    header.as_mut_bytes()[..100].fill(0);
    header.as_mut_bytes()[..name.len()].copy_from_slice(name);
    header.set_cksum();
    builder
        .append(&header, Cursor::new(&entry.data))
        .expect("append tar entry");
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub struct MatchingDebInspector;

impl DebInspector for MatchingDebInspector {
    fn inspect(&self, _deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        Ok(DebMetadata {
            package: "usb-control".to_string(),
            version: SystemVersion::parse("3.1.0")?,
            architecture: "arm64".to_string(),
            expanded_size: 4096,
            files: required_deb_files(),
            tls_cert_sha256: TLS_SHA256.to_string(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            migration_schema_to: 2,
            upgrade_signing_key_id: KEY_ID.to_string(),
        })
    }
}

pub fn required_deb_files() -> BTreeSet<PathBuf> {
    [
        "opt/usb-control/bin/usb-control",
        "opt/usb-control/bin/usb-control-updater",
        "opt/usb-control/bin/usb-control-db-migrate",
        "opt/usb-control/install-meta/release.json",
        "lib/systemd/system/usb-control.service",
        "lib/systemd/system/usb-control-updater.service",
        "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.id",
        "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.pub",
        "opt/usb-control/defaults/etc/usb-control/tls/server.crt",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

#[derive(Clone)]
pub struct DebFixtureOptions {
    pub control_package: &'static str,
    pub control_version: &'static str,
    pub control_architecture: &'static str,
    pub release_product: &'static str,
    pub release_version: &'static str,
    pub release_architecture: &'static str,
    pub release_tls_sha256: Option<String>,
    pub release_key_id: &'static str,
    pub upgrade_key_id: &'static str,
    pub upgrade_public_key: Option<&'static str>,
    pub migration_paths: Vec<&'static str>,
    pub seed_paths: Vec<&'static str>,
    pub extra_paths: Vec<&'static str>,
    pub omit_path: Option<&'static str>,
    pub release_extra_field: bool,
    pub tar_extension: Option<u8>,
    pub special_entry: Option<(&'static str, u8)>,
    pub exceed_entry_count: bool,
    pub exceed_expanded_size: bool,
    pub exceed_selected_file_size: bool,
}

impl Default for DebFixtureOptions {
    fn default() -> Self {
        Self {
            control_package: "usb-control",
            control_version: "3.1.0",
            control_architecture: "arm64",
            release_product: "usb-control",
            release_version: "3.1.0",
            release_architecture: "arm64",
            release_tls_sha256: None,
            release_key_id: KEY_ID,
            upgrade_key_id: KEY_ID,
            upgrade_public_key: None,
            migration_paths: vec![
                "opt/usb-control/db/migrations/0001_init.sql",
                "opt/usb-control/db/migrations/0002_upgrade.sql",
            ],
            seed_paths: vec!["opt/usb-control/db/seeds/0001_default.sql"],
            extra_paths: Vec::new(),
            omit_path: None,
            release_extra_field: false,
            tar_extension: None,
            special_entry: None,
            exceed_entry_count: false,
            exceed_expanded_size: false,
            exceed_selected_file_size: false,
        }
    }
}

pub struct DebFixture {
    pub _temp_dir: TempDir,
    pub path: PathBuf,
    pub tls_sha256: String,
}

pub fn write_deb_fixture(options: DebFixtureOptions) -> DebFixture {
    let temp_dir = tempfile::tempdir().expect("create DEB fixture directory");
    let path = temp_dir.path().join("fixture.deb");
    let certificate = TEST_CERTIFICATE.as_bytes();
    let certificate_der = decode_pem_certificate(certificate);
    let tls_sha256 = sha256_hex(&certificate_der);
    let release_tls_sha256 = options
        .release_tls_sha256
        .clone()
        .unwrap_or_else(|| tls_sha256.clone());

    let mut release = json!({
        "format_version": 1,
        "product": options.release_product,
        "version": options.release_version,
        "architecture": options.release_architecture,
        "supported_schema_min": 1,
        "supported_schema_max": 2,
        "tls_cert_sha256": release_tls_sha256,
        "upgrade_signing_key_id": options.release_key_id
    });
    if options.release_extra_field {
        release
            .as_object_mut()
            .expect("release metadata object")
            .insert("unknown".to_string(), json!(true));
    }
    let release = serde_json::to_vec(&release).expect("serialize release metadata");

    let control = format!(
        "Package: {}\nVersion: {}\nArchitecture: {}\nMaintainer: Test <test@example.invalid>\nDescription: fixture\n",
        options.control_package, options.control_version, options.control_architecture
    );
    let control_tar = build_tar(&[("control", control.as_bytes())]);
    let (_, generated_public_key) = sm2::gen_keypair();
    let upgrade_public_key = options.upgrade_public_key.unwrap_or(&generated_public_key);

    let mut data_entries: Vec<(String, Vec<u8>)> = vec![
        ("opt/usb-control/bin/usb-control".into(), b"main".to_vec()),
        (
            "opt/usb-control/bin/usb-control-updater".into(),
            b"updater".to_vec(),
        ),
        (
            "opt/usb-control/bin/usb-control-db-migrate".into(),
            b"migrate".to_vec(),
        ),
        ("opt/usb-control/install-meta/release.json".into(), release),
        (
            "lib/systemd/system/usb-control.service".into(),
            b"[Service]".to_vec(),
        ),
        (
            "lib/systemd/system/usb-control-updater.service".into(),
            b"[Service]".to_vec(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.id".into(),
            format!("{}\n", options.upgrade_key_id).into_bytes(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.pub".into(),
            format!("{upgrade_public_key}\n").into_bytes(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/tls/server.crt".into(),
            certificate.to_vec(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/tls/server.key".into(),
            b"tls-private-key".to_vec(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/tls/server.crt.sha256".into(),
            format!("{tls_sha256}\n").into_bytes(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/usb-control.toml".into(),
            b"[server]\n".to_vec(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/keys/license_verify.pub".into(),
            b"license-public-key".to_vec(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/keys/sm4_policy.key".into(),
            b"sm4-policy-key".to_vec(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/keys/sm2_policy.key".into(),
            b"sm2-policy-private-key".to_vec(),
        ),
        (
            "opt/usb-control/defaults/etc/usb-control/keys/sm2_policy.pub".into(),
            b"sm2-policy-public-key".to_vec(),
        ),
    ];
    for migration in &options.migration_paths {
        data_entries.push(((*migration).to_string(), b"SELECT 1;".to_vec()));
    }
    for seed in &options.seed_paths {
        data_entries.push(((*seed).to_string(), b"SELECT 1;".to_vec()));
    }
    for extra in &options.extra_paths {
        data_entries.push(((*extra).to_string(), b"forbidden".to_vec()));
    }
    if let Some(omit_path) = options.omit_path {
        data_entries.retain(|(path, _)| path != omit_path);
    }
    let data_tar = build_data_tar(&data_entries, &options);

    let mut deb = Vec::new();
    deb.extend_from_slice(b"!<arch>\n");
    append_ar_member(&mut deb, "debian-binary", b"2.0\n");
    append_ar_member(&mut deb, "control.tar", &control_tar);
    append_ar_member(&mut deb, "data.tar", &data_tar);
    fs::write(&path, deb).expect("write DEB fixture");

    DebFixture {
        _temp_dir: temp_dir,
        path,
        tls_sha256,
    }
}

fn build_tar(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut bytes);
        builder.mode(tar::HeaderMode::Deterministic);
        for (path, contents) in entries {
            append_regular(&mut builder, path, contents);
        }
        builder.finish().expect("finish DEB member tar");
    }
    bytes
}

fn build_data_tar(entries: &[(String, Vec<u8>)], options: &DebFixtureOptions) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Some(entry_type) = options.tar_extension {
        append_raw_tar_entry(&mut bytes, "extension", entry_type, 1, b"x");
    }
    for (path, contents) in entries {
        if options.exceed_selected_file_size && path == "opt/usb-control/install-meta/release.json"
        {
            append_raw_tar_entry(&mut bytes, path, b'0', 1024 * 1024 + 1, b"");
            break;
        }
        append_raw_tar_entry(&mut bytes, path, b'0', contents.len() as u64, contents);
    }
    if options.exceed_entry_count {
        for index in 0..4096 {
            append_raw_tar_entry(
                &mut bytes,
                &format!("opt/usb-control/share/limit-{index}"),
                b'0',
                0,
                b"",
            );
        }
    }
    if options.exceed_expanded_size {
        append_raw_tar_entry(
            &mut bytes,
            "opt/usb-control/share/expanded-limit",
            b'0',
            512 * 1024 * 1024 + 1,
            b"",
        );
    }
    if let Some((path, entry_type)) = options.special_entry {
        append_raw_tar_entry(&mut bytes, path, entry_type, 0, b"");
    }
    bytes.extend_from_slice(&[0u8; 1024]);
    bytes
}

fn append_raw_tar_entry(
    output: &mut Vec<u8>,
    path: &str,
    entry_type: u8,
    declared_size: u64,
    contents: &[u8],
) {
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::new(entry_type));
    header.set_mode(0o600);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(declared_size);
    assert!(path.len() <= 100, "test ustar path must fit name field");
    header.as_mut_bytes()[..100].fill(0);
    header.as_mut_bytes()[..path.len()].copy_from_slice(path.as_bytes());
    header.set_cksum();
    output.extend_from_slice(header.as_bytes());
    output.extend_from_slice(contents);
    if contents.len() as u64 == declared_size {
        let padding = (512 - contents.len() % 512) % 512;
        output.resize(output.len() + padding, 0);
    }
}

fn append_ar_member(output: &mut Vec<u8>, name: &str, contents: &[u8]) {
    let identifier = format!("{name}/");
    writeln!(
        output,
        "{identifier:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`",
        0,
        0,
        0,
        0o100644,
        contents.len()
    )
    .expect("write ar member header");
    output.extend_from_slice(contents);
    if !contents.len().is_multiple_of(2) {
        output.push(b'\n');
    }
}

fn decode_pem_certificate(pem: &[u8]) -> Vec<u8> {
    use base64::Engine;

    let pem = std::str::from_utf8(pem).expect("test certificate is UTF-8");
    let encoded: String = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .expect("decode test certificate DER")
}

const TEST_CERTIFICATE: &str = "-----BEGIN CERTIFICATE-----\n\
MIIBfTCCASOgAwIBAgIUB7pdJndRX1G4v3RoOqDDgGWU+oAwCgYIKoZIzj0EAwIw\n\
FjEUMBIGA1UEAwwLdXNiLWNvbnRyb2wwHhcNMjYwNzE0MDAwMDAwWhcNMzYwNzEx\n\
MDAwMDAwWjAWMRQwEgYDVQQDDAt1c2ItY29udHJvbDBZMBMGByqGSM49AgEGCCqG\n\
SM49AwEHA0IABNf6+ETQzh6i1FL2ubLZgqlwQ8wDsy2etVb/ZqD0NqbhFttqfDLE\n\
L8dnwVpWJl9qJVDdhcHnYGqsJrI6uHujUzBRMB0GA1UdDgQWBBRZ8g6D2kT2PY2h\n\
UBb7vLJtvfI31TAfBgNVHSMEGDAWgBRZ8g6D2kT2PY2hUBb7vLJtvfI31TAPBgNV\n\
HRMBAf8EBTADAQH/MAoGCCqGSM49BAMCA0gAMEUCIQDDZQ4+7z45e4Q0zZkY3uVt\n\
dc8r4TtSL2QxDre6VgIgEx4vhfxwnb3q+U/Q5M5STQbh91FlbCqGjDulG7g=\n\
-----END CERTIFICATE-----\n";

pub fn test_certificate_pem() -> &'static [u8] {
    TEST_CERTIFICATE.as_bytes()
}
