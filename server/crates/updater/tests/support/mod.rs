// 每个 integration test 会把共享辅助模块编译为独立 target，部分辅助项只由另一 target 使用。
#![allow(dead_code)]

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use system_upgrade::{InstalledRelease, SystemVersion, UpgradeManifest, UpgradeTask};
use usb_control_updater::{
    Clock, CommandOutput, CommandRunner, CommandSpec, PackageRevalidator, RevalidatedPackage,
    UpdaterError, UpgradePaths,
};

pub const TEST_CERTIFICATE_PEM: &str = include_str!("../../../../../deploy/assets/tls/server.crt");

pub fn version(value: &str) -> SystemVersion {
    SystemVersion::parse(value).unwrap()
}

pub fn target_release(tls_cert_sha256: String) -> InstalledRelease {
    InstalledRelease {
        format_version: 1,
        product: "usb-control".into(),
        version: version("3.0.2"),
        architecture: "arm64".into(),
        supported_schema_min: 1,
        supported_schema_max: 1,
        tls_cert_sha256,
        upgrade_signing_key_id: "release-1".into(),
    }
}

#[derive(Clone, Default)]
pub struct FakeCommandRunner {
    calls: Arc<Mutex<Vec<CommandSpec>>>,
    outputs: Arc<Mutex<VecDeque<Result<CommandOutput, UpdaterError>>>>,
    observed_path: Arc<Mutex<Option<PathBuf>>>,
    observations: Arc<Mutex<Vec<bool>>>,
}

impl FakeCommandRunner {
    pub fn push_success(&self, stdout: &str) {
        self.outputs.lock().unwrap().push_back(Ok(CommandOutput {
            success: true,
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
            stdout_truncated: false,
            stderr_truncated: false,
        }));
    }

    pub fn push_failure(&self, stage: &str) {
        self.outputs
            .lock()
            .unwrap()
            .push_back(Err(UpdaterError::CommandFailed {
                stage: stage.into(),
                program: "fake".into(),
                status: Some(1),
            }));
    }

    pub fn calls(&self) -> Vec<CommandSpec> {
        self.calls.lock().unwrap().clone()
    }

    pub fn observe_path(&self, path: PathBuf) {
        *self.observed_path.lock().unwrap() = Some(path);
    }

    pub fn observations(&self) -> Vec<bool> {
        self.observations.lock().unwrap().clone()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(&self, command: &CommandSpec) -> Result<CommandOutput, UpdaterError> {
        self.calls.lock().unwrap().push(command.clone());
        if let Some(path) = self.observed_path.lock().unwrap().as_ref() {
            self.observations.lock().unwrap().push(path.is_file());
        }
        self.outputs
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| panic!("missing fake output for {command:?}"))
    }
}

#[derive(Clone)]
pub struct FakePackageRevalidator {
    pub fail: bool,
    pub target_release: InstalledRelease,
}

impl PackageRevalidator for FakePackageRevalidator {
    fn revalidate(
        &self,
        paths: &UpgradePaths,
        task: &UpgradeTask,
    ) -> Result<RevalidatedPackage, UpdaterError> {
        if self.fail {
            return Err(UpdaterError::TaskInvalid(
                "injected revalidation failure".into(),
            ));
        }
        Ok(RevalidatedPackage {
            manifest: UpgradeManifest {
                format_version: 1,
                product: "usb-control".into(),
                package_version: version("3.0.2"),
                architecture: "arm64".into(),
                minimum_current_version: version("3.0.1"),
                protocol_version: 1,
                tls_cert_sha256: self.target_release.tls_cert_sha256.clone(),
                deb_file: "usb-control_V3.0.2_arm64.deb".into(),
                deb_size: 9,
                deb_sha256: "b".repeat(64),
                schema_from: 1,
                schema_to: 1,
                signing_key_id: "release-1".into(),
            },
            candidate_deb: paths.staging_dir.join(&task.upgrade_id).join("payload.deb"),
            target_release: self.target_release.clone(),
        })
    }
}

pub struct FakeClock {
    values: Mutex<VecDeque<i64>>,
}

impl FakeClock {
    pub fn fixed(value: i64) -> Self {
        Self {
            values: Mutex::new(std::iter::repeat_n(value, 64).collect()),
        }
    }

    pub fn sequence(values: impl IntoIterator<Item = i64>) -> Self {
        Self {
            values: Mutex::new(values.into_iter().collect()),
        }
    }
}

impl Clock for &FakeClock {
    fn now(&self) -> Result<i64, UpdaterError> {
        (*self).now()
    }
}

impl CommandRunner for &FakeCommandRunner {
    fn run(&self, command: &CommandSpec) -> Result<CommandOutput, UpdaterError> {
        (*self).run(command)
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Result<i64, UpdaterError> {
        self.values
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| UpdaterError::TaskInvalid("fake clock exhausted".into()))
    }
}
