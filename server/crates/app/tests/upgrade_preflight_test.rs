use std::path::{Path, PathBuf};
use std::sync::Arc;

use system_upgrade::{
    ActiveRelease, ActiveReleaseStore, SystemVersion, UpgradeError, UpgradePreflight,
    UpgradePreflightFailure, UpgradePreflightRequest, UpgradeStateLock,
};
use usb_control_app::upgrade_preflight::{SystemUpgradePreflight, UpgradeHostProbe};

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

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("upgrade");
        let releases = ActiveReleaseStore::new(root.clone()).unwrap();
        let guard = UpgradeStateLock::acquire(&root).unwrap();
        releases
            .commit(
                &guard,
                &ActiveRelease {
                    format_version: 1,
                    version: version("3.0.1"),
                    schema_version: 1,
                    committed_at: 100,
                    online_upgrade_id: None,
                },
            )
            .unwrap();
        Self { _temp: temp, root }
    }

    fn preflight(&self, probe: FakeProbe) -> SystemUpgradePreflight {
        SystemUpgradePreflight::new(
            Arc::new(probe),
            self.root.clone(),
            self.root.join("active-release.json"),
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
fn accepts_consistent_environment_without_lkg() {
    let fixture = Fixture::new();
    fixture
        .preflight(FakeProbe::healthy())
        .check(&request())
        .unwrap();
    assert!(!fixture.root.join("rollback").exists());
}

#[test]
fn preflight_space_budget_excludes_old_release_deb() {
    let fixture = Fixture::new();
    let required = 100 + 50 + 4096 + 256 * 1024 * 1024;
    let mut probe = FakeProbe::healthy();
    probe.available = required - 1;
    assert_failure(
        fixture.preflight(probe).check(&request()),
        UpgradePreflightFailure::InsufficientSpace {
            required,
            available: required - 1,
        },
    );
}

#[test]
fn preflight_rejects_active_release_version_or_schema_mismatch() {
    for (source_version, schema_from) in [(version("3.0.0"), 1), (version("3.0.1"), 2)] {
        let fixture = Fixture::new();
        let mut value = request();
        value.source_version = source_version;
        value.schema_from = schema_from;
        assert!(matches!(
            fixture.preflight(FakeProbe::healthy()).check(&value),
            Err(UpgradeError::Preflight(
                UpgradePreflightFailure::ProbeFailed(_)
            ))
        ));
    }
}

#[test]
fn preflight_checks_dpkg_service_clamav_and_platform() {
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
fn preflight_reports_space_overflow() {
    let fixture = Fixture::new();
    let mut value = request();
    value.package_size = u64::MAX;
    assert_failure(
        fixture.preflight(FakeProbe::healthy()).check(&value),
        UpgradePreflightFailure::ProbeFailed("升级空间预算溢出".into()),
    );
}
