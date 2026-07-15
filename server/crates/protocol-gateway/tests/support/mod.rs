// 每个 integration test 都会把 support 编译为独立模块，仅使用其中一部分辅助项。
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use auth_session::{AuthService, SessionManager};
use log_audit::AuditService;
use protocol_gateway::context::RequestContext;
use protocol_gateway::post_send::{PostSendAction, PostSendActionExecutor};
use protocol_gateway::GatewayError;
use storage::Storage;
use storage_test_support::initialize_database;
use system_upgrade::{
    DebInspector, DebMetadata, PackageStager, PackageVerifier, SystemVersion, UpgradeCoordinator,
    UpgradeEnvironment, UpgradeError, UpgradePreflight, UpgradePreflightRequest, UpgradeScheduler,
};
use tempfile::{NamedTempFile, TempDir, TempPath};
use tokio::io::AsyncWrite;

pub struct RequestFixture {
    pub context: RequestContext,
    pub storage: Arc<Storage>,
    pub upgrade_root: TempDir,
    pub database: TempPath,
}

pub fn request_fixture(seq_id: u32) -> RequestFixture {
    let tmp = NamedTempFile::new().expect("create temporary database");
    let path = tmp.into_temp_path();
    initialize_database(&path);
    let storage = Arc::new(Storage::open(&path).expect("open temporary database"));
    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&storage),
        SessionManager::new(),
    ));
    let audit_service = Arc::new(AuditService::new(Arc::clone(&storage), &path));

    let upgrade_root = tempfile::tempdir().expect("create temporary upgrade root");
    let coordinator = Arc::new(
        UpgradeCoordinator::new(
            upgrade_root.path().to_path_buf(),
            PackageStager::new(upgrade_root.path().to_path_buf(), 128 * 1024 * 1024),
            PackageVerifier::new(upgrade_root.path().join("keys"), Arc::new(TestDebInspector)),
            UpgradeEnvironment {
                current_version: SystemVersion::parse("3.0.1").unwrap(),
                current_schema: 1,
                supported_schema_max: 2,
                protocol_version: 1,
            },
            Arc::new(TestUpgradePreflight),
            Arc::new(TestUpgradeScheduler),
        )
        .expect("create test upgrade coordinator"),
    );
    let context = RequestContext {
        seq_id,
        session: None,
        source_ip: "127.0.0.1".into(),
        auth_service,
        audit_service,
        whitelist_manager: None,
        device_manager: None,
        device_runtime_registry: None,
        storage: Some(Arc::clone(&storage)),
        policy_service: None,
        license_validator: None,
        system_upgrade_coordinator: coordinator,
        system_upgrade_root: upgrade_root.path().to_path_buf(),
        virusdb_upgrade_mgr: None,
    };
    RequestFixture {
        context,
        storage,
        upgrade_root,
        database: path,
    }
}

struct TestDebInspector;

impl DebInspector for TestDebInspector {
    fn inspect(&self, _deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        Ok(DebMetadata {
            package: "usb-control".into(),
            version: SystemVersion::parse("3.1.0")?,
            architecture: "arm64".into(),
            expanded_size: 4096,
            files: BTreeSet::new(),
            tls_cert_sha256: String::new(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            migration_schema_to: 2,
            upgrade_signing_key_id: String::new(),
        })
    }
}

struct TestUpgradePreflight;

impl UpgradePreflight for TestUpgradePreflight {
    fn check(&self, _request: &UpgradePreflightRequest) -> Result<(), UpgradeError> {
        Ok(())
    }
}

struct TestUpgradeScheduler;

impl UpgradeScheduler for TestUpgradeScheduler {
    fn start(&self, _upgrade_id: &str) -> Result<(), UpgradeError> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorEvent {
    Executed(String),
    Cancelled(String),
}

#[derive(Default)]
pub struct RecordingExecutor {
    events: Mutex<Vec<ExecutorEvent>>,
    execute_error: Option<String>,
}

impl RecordingExecutor {
    pub fn failing_execute(message: impl Into<String>) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            execute_error: Some(message.into()),
        }
    }

    pub fn events(&self) -> Vec<ExecutorEvent> {
        self.events.lock().expect("event lock poisoned").clone()
    }
}

impl PostSendActionExecutor for RecordingExecutor {
    fn execute(&self, action: PostSendAction) -> Result<(), GatewayError> {
        let PostSendAction::StartSystemUpgrade { upgrade_id } = action;
        self.events
            .lock()
            .expect("event lock poisoned")
            .push(ExecutorEvent::Executed(upgrade_id));
        match self.execute_error.as_ref() {
            Some(message) => Err(GatewayError::TlsConfig(message.clone())),
            None => Ok(()),
        }
    }

    fn cancel(&self, action: &PostSendAction) {
        let PostSendAction::StartSystemUpgrade { upgrade_id } = action;
        self.events
            .lock()
            .expect("event lock poisoned")
            .push(ExecutorEvent::Cancelled(upgrade_id.clone()));
    }
}

pub struct FlushFailWriter {
    written: Vec<u8>,
    error_kind: io::ErrorKind,
}

impl FlushFailWriter {
    pub fn new(error_kind: io::ErrorKind) -> Self {
        Self {
            written: Vec::new(),
            error_kind,
        }
    }

    pub fn written(&self) -> &[u8] {
        &self.written
    }
}

impl AsyncWrite for FlushFailWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.written.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Err(io::Error::new(
            self.error_kind,
            "injected flush failure",
        )))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}
