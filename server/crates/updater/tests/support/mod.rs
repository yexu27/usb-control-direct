use std::collections::{BTreeSet, VecDeque};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};
use smcrypto::{sm2, sm3};
use system_upgrade::{
    ActiveCommitError, ActiveRelease, DebInspector, DebMetadata, SystemVersion, UpgradeError,
    UpgradeManifest, UpgradeTask,
};
use tar::{Builder, Header};
use usb_control_updater::{
    ActiveReleasePublisher, Clock, CommandOutput, CommandRunner, CommandSpec, FileLkgRepository,
    LastKnownGoodRelease, LkgRepository, PackageRevalidator, RevalidatedPackage, UpdaterError,
    UpgradePaths,
};

pub const TEST_CERTIFICATE_PEM: &str = include_str!("../../../../../deploy/assets/tls/server.crt");

pub struct MatchingDebInspector {
    pub tls_cert_sha256: String,
}

impl DebInspector for MatchingDebInspector {
    fn inspect(&self, _deb_path: &std::path::Path) -> Result<DebMetadata, UpgradeError> {
        Ok(DebMetadata {
            package: "usb-control".into(),
            version: SystemVersion::parse("3.0.2")?,
            architecture: "arm64".into(),
            expanded_size: 4096,
            files: BTreeSet::new(),
            tls_cert_sha256: self.tls_cert_sha256.clone(),
            supported_schema_min: 1,
            supported_schema_max: 1,
            migration_schema_to: 1,
            upgrade_signing_key_id: "release-1".into(),
        })
    }
}

pub fn signed_package(manifest_raw: &[u8], deb: &[u8]) -> (Vec<u8>, String) {
    let (private_key, public_key) = sm2::gen_keypair();
    let deb_digest = Sha256::digest(deb);
    let mut signing_input = Vec::new();
    signing_input.extend_from_slice(b"USB-CONTROL-UPGRADE-V1\0");
    signing_input.extend_from_slice(&(manifest_raw.len() as u64).to_be_bytes());
    signing_input.extend_from_slice(manifest_raw);
    signing_input.extend_from_slice(&deb_digest);
    let digest = hex::decode(sm3::sm3_hash(&signing_input)).unwrap();
    let signature = sm2::Sign::new(&private_key).sign(&digest);
    let mut package = Vec::new();
    {
        let mut builder = Builder::new(&mut package);
        builder.mode(tar::HeaderMode::Deterministic);
        append_tar_file(&mut builder, "manifest.json", manifest_raw);
        append_tar_file(&mut builder, "usb-control_V3.0.2_arm64.deb", deb);
        append_tar_file(&mut builder, "signature.sm2", &signature);
        builder.finish().unwrap();
    }
    (package, public_key)
}

fn append_tar_file(builder: &mut Builder<&mut Vec<u8>>, path: &str, bytes: &[u8]) {
    let mut header = Header::new_ustar();
    header.set_path(path).unwrap();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_mode(0o600);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(bytes.len() as u64);
    header.set_cksum();
    builder.append(&header, Cursor::new(bytes)).unwrap();
}

type ScheduledWrite = (usize, PathBuf, Vec<u8>);

#[derive(Clone, Default)]
pub struct FakeCommandRunner {
    calls: Arc<Mutex<Vec<CommandSpec>>>,
    outputs: Arc<Mutex<VecDeque<Result<CommandOutput, UpdaterError>>>>,
    writes_on_call: Arc<Mutex<Vec<ScheduledWrite>>>,
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
                stage: stage.to_string(),
                program: "fake".to_string(),
                status: Some(1),
            }));
    }

    pub fn push_error(&self, error: UpdaterError) {
        self.outputs.lock().unwrap().push_back(Err(error));
    }

    pub fn calls(&self) -> Vec<CommandSpec> {
        self.calls.lock().unwrap().clone()
    }

    pub fn write_on_call(&self, call: usize, path: PathBuf, bytes: Vec<u8>) {
        self.writes_on_call
            .lock()
            .unwrap()
            .push((call, path, bytes));
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(&self, command: &CommandSpec) -> Result<CommandOutput, UpdaterError> {
        let call = {
            let mut calls = self.calls.lock().unwrap();
            calls.push(command.clone());
            calls.len()
        };
        for (_, path, bytes) in self
            .writes_on_call
            .lock()
            .unwrap()
            .iter()
            .filter(|(target_call, _, _)| call == *target_call)
        {
            std::fs::write(path, bytes).unwrap();
        }
        self.outputs
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| panic!("missing fake result for {:?}", command))
    }
}

#[derive(Default)]
pub struct FakePackageRevalidator;

impl PackageRevalidator for FakePackageRevalidator {
    fn revalidate(
        &self,
        paths: &UpgradePaths,
        task: &UpgradeTask,
    ) -> Result<RevalidatedPackage, UpdaterError> {
        let staging = paths.staging_dir.join(&task.upgrade_id);
        let manifest: UpgradeManifest =
            serde_json::from_slice(&std::fs::read(staging.join("manifest.json"))?)?;
        let lkg: LastKnownGoodRelease =
            serde_json::from_slice(&std::fs::read(&paths.last_known_good_metadata)?)?;
        Ok(RevalidatedPackage {
            manifest,
            candidate_deb: staging.join("payload.deb"),
            lkg,
        })
    }
}

pub struct FakeClock {
    values: Mutex<VecDeque<i64>>,
}

impl FakeClock {
    pub fn new(values: impl IntoIterator<Item = i64>) -> Self {
        Self {
            values: Mutex::new(values.into_iter().collect()),
        }
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

pub struct FailAfterPreservingPrevious;

pub struct FailPrepareRepository;

impl LkgRepository for FailPrepareRepository {
    fn prepare(
        &self,
        _paths: &UpgradePaths,
        _candidate: &std::path::Path,
        _sha: &str,
    ) -> Result<(), UpdaterError> {
        Err(UpdaterError::TaskInvalid("injected prepare failure".into()))
    }

    fn promote(
        &self,
        paths: &UpgradePaths,
        manifest: &UpgradeManifest,
        sha: &str,
    ) -> Result<(), UpdaterError> {
        FileLkgRepository.promote(paths, manifest, sha)
    }

    fn restore(
        &self,
        paths: &UpgradePaths,
        metadata: &LastKnownGoodRelease,
    ) -> Result<(), UpdaterError> {
        FileLkgRepository.restore(paths, metadata)
    }

    fn abort_prepared(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        FileLkgRepository.abort_prepared(paths)
    }

    fn cleanup_rollback(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        FileLkgRepository.cleanup_rollback(paths)
    }

    fn cleanup_committed(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        FileLkgRepository.cleanup_committed(paths)
    }
}

impl LkgRepository for FailAfterPreservingPrevious {
    fn prepare(
        &self,
        paths: &UpgradePaths,
        candidate: &std::path::Path,
        sha: &str,
    ) -> Result<(), UpdaterError> {
        FileLkgRepository.prepare(paths, candidate, sha)
    }

    fn promote(
        &self,
        paths: &UpgradePaths,
        _manifest: &UpgradeManifest,
        _sha: &str,
    ) -> Result<(), UpdaterError> {
        std::fs::rename(&paths.last_known_good_deb, &paths.previous_deb)?;
        std::fs::File::open(&paths.rollback_dir)?.sync_all()?;
        Err(UpdaterError::TaskInvalid(
            "injected next-to-lkg failure".into(),
        ))
    }

    fn restore(
        &self,
        paths: &UpgradePaths,
        metadata: &LastKnownGoodRelease,
    ) -> Result<(), UpdaterError> {
        FileLkgRepository.restore(paths, metadata)
    }

    fn abort_prepared(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        FileLkgRepository.abort_prepared(paths)
    }

    fn cleanup_rollback(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        FileLkgRepository.cleanup_rollback(paths)
    }

    fn cleanup_committed(&self, paths: &UpgradePaths) -> Result<(), UpdaterError> {
        FileLkgRepository.cleanup_committed(paths)
    }
}

pub struct PublishThenFailDirectorySync;

impl ActiveReleasePublisher for PublishThenFailDirectorySync {
    fn commit(
        &self,
        root: &std::path::Path,
        release: &ActiveRelease,
    ) -> Result<(), ActiveCommitError> {
        std::fs::write(
            root.join("active-release.json"),
            serde_json::to_vec(release).unwrap(),
        )
        .unwrap();
        Err(ActiveCommitError::AfterRename(UpgradeError::Io(
            std::io::Error::other("injected parent directory fsync failure"),
        )))
    }
}
