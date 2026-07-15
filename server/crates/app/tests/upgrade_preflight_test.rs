use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::Digest;
use system_upgrade::{
    certificate_sha256, ActiveRelease, DebInspector, DebMetadata, LastKnownGoodRelease,
    SystemVersion, UpgradeError, UpgradePreflight, UpgradePreflightFailure,
    UpgradePreflightRequest,
};
use usb_control_app::upgrade_preflight::{SystemUpgradePreflight, UpgradeHostProbe};

const CERTIFICATE: &[u8] = b"-----BEGIN CERTIFICATE-----\n\
MIIBfTCCASOgAwIBAgIUB7pdJndRX1G4v3RoOqDDgGWU+oAwCgYIKoZIzj0EAwIw\n\
FjEUMBIGA1UEAwwLdXNiLWNvbnRyb2wwHhcNMjYwNzE0MDAwMDAwWhcNMzYwNzEx\n\
MDAwMDAwWjAWMRQwEgYDVQQDDAt1c2ItY29udHJvbDBZMBMGByqGSM49AgEGCCqG\n\
SM49AwEHA0IABNf6+ETQzh6i1FL2ubLZgqlwQ8wDsy2etVb/ZqD0NqbhFttqfDLE\n\
L8dnwVpWJl9qJVDdhcHnYGqsJrI6uHujUzBRMB0GA1UdDgQWBBRZ8g6D2kT2PY2h\n\
UBb7vLJtvfI31TAfBgNVHSMEGDAWgBRZ8g6D2kT2PY2hUBb7vLJtvfI31TAPBgNV\n\
HRMBAf8EBTADAQH/MAoGCCqGSM49BAMCA0gAMEUCIQDDZQ4+7z45e4Q0zZkY3uVt\n\
dc8r4TtSL2QxDre6VgIgEx4vhfxwnb3q+U/Q5M5STQbh91FlbCqGjDulG7g=\n\
-----END CERTIFICATE-----\n";

#[derive(Clone)]
struct FakeProbe {
    available: u64,
    locks: bool,
    audit: bool,
    service: bool,
    clamav: bool,
    platform: bool,
}

impl FakeProbe {
    fn healthy() -> Self {
        Self {
            available: u64::MAX,
            locks: true,
            audit: true,
            service: true,
            clamav: true,
            platform: true,
        }
    }
}

impl UpgradeHostProbe for FakeProbe {
    fn available_bytes(&self, _path: &Path) -> Result<u64, String> {
        Ok(self.available)
    }

    fn dpkg_locks_available(&self) -> Result<bool, String> {
        Ok(self.locks)
    }

    fn dpkg_audit_clean(&self) -> Result<bool, String> {
        Ok(self.audit)
    }

    fn main_service_active(&self) -> Result<bool, String> {
        Ok(self.service)
    }

    fn clamav_available(&self) -> Result<bool, String> {
        Ok(self.clamav)
    }

    fn platform_compatible(&self) -> Result<bool, String> {
        Ok(self.platform)
    }
}

#[derive(Clone)]
struct LkgInspector {
    metadata: DebMetadata,
}

impl DebInspector for LkgInspector {
    fn inspect(&self, _deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        Ok(self.metadata.clone())
    }
}

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    active: PathBuf,
    lkg_metadata: PathBuf,
    lkg_deb: PathBuf,
    tls: PathBuf,
    lkg_deb_size: u64,
    tls_sha256: String,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("upgrade");
        let rollback = root.join("rollback");
        fs::create_dir_all(&rollback).unwrap();
        let active = root.join("active-release.json");
        let lkg_metadata = rollback.join("last-known-good.json");
        let lkg_deb = rollback.join("last-known-good.deb");
        let tls = temp.path().join("server.crt");
        let deb = b"last-known-good-deb";
        fs::write(&lkg_deb, deb).unwrap();
        fs::write(&tls, CERTIFICATE).unwrap();
        let deb_sha256 = hex::encode(sha2::Sha256::digest(deb));
        let tls_sha256 = certificate_sha256(CERTIFICATE).unwrap();
        fs::write(
            &active,
            serde_json::to_vec(&ActiveRelease {
                format_version: 1,
                upgrade_id: "upgrade-active".into(),
                version: version("3.0.1"),
                deb_sha256: deb_sha256.clone(),
                schema_version: 1,
                committed_at: 100,
            })
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &lkg_metadata,
            serde_json::to_vec(&LastKnownGoodRelease {
                format_version: 1,
                version: version("3.0.1"),
                deb_sha256,
                schema_version: 1,
                tls_cert_sha256: tls_sha256.clone(),
            })
            .unwrap(),
        )
        .unwrap();
        Self {
            _temp: temp,
            root,
            active,
            lkg_metadata,
            lkg_deb,
            tls,
            lkg_deb_size: deb.len() as u64,
            tls_sha256,
        }
    }

    fn preflight(&self, probe: FakeProbe) -> SystemUpgradePreflight {
        self.preflight_with_metadata(probe, self.matching_deb_metadata())
    }

    fn matching_deb_metadata(&self) -> DebMetadata {
        DebMetadata {
            package: "usb-control".into(),
            version: version("3.0.1"),
            architecture: "arm64".into(),
            expanded_size: 4096,
            files: BTreeSet::new(),
            tls_cert_sha256: self.tls_sha256.clone(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            migration_schema_to: 1,
            upgrade_signing_key_id: "upgrade-prod-01".into(),
        }
    }

    fn preflight_with_metadata(
        &self,
        probe: FakeProbe,
        metadata: DebMetadata,
    ) -> SystemUpgradePreflight {
        SystemUpgradePreflight::new(
            Arc::new(probe),
            Arc::new(LkgInspector { metadata }),
            self.root.clone(),
            self.active.clone(),
            self.lkg_metadata.clone(),
            self.lkg_deb.clone(),
            self.tls.clone(),
        )
    }
}

fn request() -> UpgradePreflightRequest {
    UpgradePreflightRequest {
        package_size: 100,
        deb_size: 50,
        expanded_size: 4096,
        source_version: version("3.0.1"),
        target_version: version("3.1.0"),
        schema_from: 1,
        schema_to: 2,
    }
}

fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

fn assert_failure(result: Result<(), UpgradeError>, expected: UpgradePreflightFailure) {
    assert!(matches!(result, Err(UpgradeError::Preflight(actual)) if actual == expected));
}

#[test]
fn accepts_consistent_environment() {
    let fixture = Fixture::new();
    fixture
        .preflight(FakeProbe::healthy())
        .check(&request())
        .unwrap();
}

#[test]
fn rejects_insufficient_space() {
    let fixture = Fixture::new();
    let mut probe = FakeProbe::healthy();
    probe.available = 1;
    let required = 100 + 50 + 4096 + fixture.lkg_deb_size + 256 * 1024 * 1024;
    assert_failure(
        fixture.preflight(probe).check(&request()),
        UpgradePreflightFailure::InsufficientSpace {
            required,
            available: 1,
        },
    );
}

#[test]
fn rejects_dpkg_service_clamav_and_platform_failures() {
    let fixture = Fixture::new();
    let cases = [
        (0, UpgradePreflightFailure::DpkgBusy),
        (1, UpgradePreflightFailure::DpkgDamaged),
        (2, UpgradePreflightFailure::ServiceUnavailable),
        (3, UpgradePreflightFailure::ClamAvUnavailable),
        (4, UpgradePreflightFailure::PlatformIncompatible),
    ];
    for (index, expected) in cases {
        let mut probe = FakeProbe::healthy();
        match index {
            0 => probe.locks = false,
            1 => probe.audit = false,
            2 => probe.service = false,
            3 => probe.clamav = false,
            4 => probe.platform = false,
            _ => unreachable!(),
        }
        assert_failure(fixture.preflight(probe).check(&request()), expected);
    }
}

#[test]
fn rejects_lkg_source_version_or_schema_mismatch() {
    let fixture = Fixture::new();
    let mut active: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.active).unwrap()).unwrap();
    active["schema_version"] = serde_json::json!(2);
    fs::write(&fixture.active, serde_json::to_vec(&active).unwrap()).unwrap();
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&request()),
        UpgradePreflightFailure::RollbackUnavailable,
    );
}

#[test]
fn rejects_missing_or_malformed_active_release() {
    let fixture = Fixture::new();
    fs::remove_file(&fixture.active).unwrap();
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&request()),
        UpgradePreflightFailure::RollbackUnavailable,
    );

    fs::write(&fixture.active, b"not json").unwrap();
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&request()),
        UpgradePreflightFailure::RollbackUnavailable,
    );
}

#[test]
fn rejects_missing_or_hash_mismatched_lkg() {
    let fixture = Fixture::new();
    fs::write(&fixture.lkg_deb, b"tampered").unwrap();
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&request()),
        UpgradePreflightFailure::RollbackUnavailable,
    );

    fs::remove_file(&fixture.lkg_deb).unwrap();
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&request()),
        UpgradePreflightFailure::RollbackUnavailable,
    );
}

#[test]
fn rejects_lkg_deb_metadata_mismatch() {
    let fixture = Fixture::new();
    let mut metadata = fixture.matching_deb_metadata();
    metadata.package = "other-product".into();
    assert_failure(
        fixture
            .preflight_with_metadata(FakeProbe::healthy(), metadata)
            .check(&request()),
        UpgradePreflightFailure::RollbackUnavailable,
    );
}

#[test]
fn rejects_installed_tls_mismatch() {
    let fixture = Fixture::new();
    fs::write(&fixture.tls, b"invalid certificate").unwrap();
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&request()),
        UpgradePreflightFailure::RollbackUnavailable,
    );
}

#[test]
fn rejects_space_budget_overflow() {
    let fixture = Fixture::new();
    let mut value = request();
    value.package_size = u64::MAX;
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&value),
        UpgradePreflightFailure::ProbeFailed("升级空间预算溢出".into()),
    );
}
