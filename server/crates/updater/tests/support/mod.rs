// 每个 integration test 会把共享辅助模块编译为独立 target，部分辅助项只由另一 target 使用。
#![allow(dead_code)]

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use system_upgrade::{InstalledRelease, SystemVersion, UpgradeManifest, UpgradeTask};
use usb_control_db_migrate::UpgradeDatabaseState;
use usb_control_updater::{
    Clock, CommandOutput, CommandRunner, CommandSpec, PackageRevalidator, RevalidatedPackage,
    UpdaterError, UpgradeDatabase, UpgradePaths,
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

    pub fn clear_outputs(&self) {
        self.outputs.lock().unwrap().clear();
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
        _database_state: &UpgradeDatabaseState,
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
                protocol_version: 1,
                tls_cert_sha256: self.target_release.tls_cert_sha256.clone(),
                deb_file: "usb-control_V3.0.2_arm64.deb".into(),
                deb_size: 9,
                deb_sha256: "b".repeat(64),
                schema_to: 1,
                signing_key_id: "release-1".into(),
            },
            candidate_deb: paths.staging_dir.join(&task.upgrade_id).join("payload.deb"),
            target_release: self.target_release.clone(),
        })
    }
}

#[derive(Clone)]
pub struct FakeUpgradeDatabase {
    state: Arc<Mutex<UpgradeDatabaseState>>,
    virus_db_version: Arc<Mutex<String>>,
    virus_db_updated_at: Arc<Mutex<i64>>,
    committed_at: Arc<Mutex<i64>>,
    fail_read: Arc<Mutex<bool>>,
    fail_online_commit: Arc<Mutex<bool>>,
    fail_direct_commit: Arc<Mutex<bool>>,
    read_count: Arc<Mutex<usize>>,
    online_commit_count: Arc<Mutex<usize>>,
    direct_commit_count: Arc<Mutex<usize>>,
}

impl FakeUpgradeDatabase {
    pub fn new(system_version: &str, schema_version: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(UpgradeDatabaseState {
                system_version: system_version.into(),
                schema_version,
            })),
            virus_db_version: Arc::new(Mutex::new(String::new())),
            virus_db_updated_at: Arc::new(Mutex::new(0)),
            committed_at: Arc::new(Mutex::new(0)),
            fail_read: Arc::new(Mutex::new(false)),
            fail_online_commit: Arc::new(Mutex::new(false)),
            fail_direct_commit: Arc::new(Mutex::new(false)),
            read_count: Arc::new(Mutex::new(0)),
            online_commit_count: Arc::new(Mutex::new(0)),
            direct_commit_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn set_system_version(&self, version: &str) {
        self.state.lock().unwrap().system_version = version.into();
    }

    pub fn fail_online_commit(&self) {
        *self.fail_online_commit.lock().unwrap() = true;
    }

    pub fn fail_direct_commit(&self) {
        *self.fail_direct_commit.lock().unwrap() = true;
    }

    pub fn state(&self) -> UpgradeDatabaseState {
        self.state.lock().unwrap().clone()
    }

    pub fn read_count(&self) -> usize {
        *self.read_count.lock().unwrap()
    }

    pub fn install_state(&self) -> (String, String, i64, i64) {
        (
            self.state().system_version,
            self.virus_db_version.lock().unwrap().clone(),
            *self.virus_db_updated_at.lock().unwrap(),
            *self.committed_at.lock().unwrap(),
        )
    }

    pub fn online_commit_count(&self) -> usize {
        *self.online_commit_count.lock().unwrap()
    }

    pub fn direct_commit_count(&self) -> usize {
        *self.direct_commit_count.lock().unwrap()
    }
}

impl UpgradeDatabase for FakeUpgradeDatabase {
    fn read_state(&self) -> Result<UpgradeDatabaseState, UpdaterError> {
        *self.read_count.lock().unwrap() += 1;
        if *self.fail_read.lock().unwrap() {
            return Err(UpdaterError::TaskInvalid(
                "injected database read failure".into(),
            ));
        }
        Ok(self.state())
    }

    fn compare_and_commit_online_install_state(
        &self,
        expected_source: &str,
        target: &str,
        virus_db_version: &str,
        virus_db_updated_at: i64,
        committed_at: i64,
    ) -> Result<(), UpdaterError> {
        *self.online_commit_count.lock().unwrap() += 1;
        if *self.fail_online_commit.lock().unwrap() {
            return Err(UpdaterError::TaskInvalid(
                "injected online commit failure".into(),
            ));
        }
        let mut state = self.state.lock().unwrap();
        if state.system_version != expected_source {
            return Err(UpdaterError::TaskInvalid(
                "database source version changed".into(),
            ));
        }
        state.system_version = target.into();
        *self.virus_db_version.lock().unwrap() = virus_db_version.into();
        *self.virus_db_updated_at.lock().unwrap() = virus_db_updated_at;
        *self.committed_at.lock().unwrap() = committed_at;
        Ok(())
    }

    fn commit_direct_install_state(
        &self,
        target: &str,
        virus_db_version: &str,
        virus_db_updated_at: i64,
        committed_at: i64,
    ) -> Result<(), UpdaterError> {
        *self.direct_commit_count.lock().unwrap() += 1;
        if *self.fail_direct_commit.lock().unwrap() {
            return Err(UpdaterError::TaskInvalid(
                "injected direct commit failure".into(),
            ));
        }
        self.state.lock().unwrap().system_version = target.into();
        *self.virus_db_version.lock().unwrap() = virus_db_version.into();
        *self.virus_db_updated_at.lock().unwrap() = virus_db_updated_at;
        *self.committed_at.lock().unwrap() = committed_at;
        Ok(())
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
