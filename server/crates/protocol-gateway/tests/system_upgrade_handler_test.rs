mod support;

use std::collections::BTreeSet;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use auth_session::session::SessionInfo;
use common::code::ResultCode;
use common::proto::{CmdUploadSystemUpgrade, RspCommon};
use log_audit::AuditService;
use prost::Message;
use protocol_gateway::codec;
use protocol_gateway::handlers::system::handle_upload_system_upgrade;
use protocol_gateway::post_send::PostSendAction;
use serde_json::json;
use sha2::{Digest, Sha256};
use smcrypto::{sm2, sm3};
use system_upgrade::{
    DebInspector, DebMetadata, PackageStager, PackageVerifier, SystemVersion, UpgradeCoordinator,
    UpgradeError, UpgradePreflight, UpgradePreflightRequest, UpgradeScheduler, UpgradeSourceReader,
    UpgradeSourceState,
};
use tar::{Builder, Header};
use tempfile::TempDir;

use support::request_fixture;

const MAX_PACKAGE_SIZE: u64 = 128 * 1024 * 1024;
const DEB_NAME: &str = "usb-control_V3.1.0_arm64.deb";
const KEY_ID: &str = "upgrade-prod-01";
const TLS_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn accepted_upgrade_returns_success_and_post_send_action() {
    let fixture = PackageFixture::valid();
    let (ctx, _database) = accepted_context(&fixture, 41);
    let command = command(&fixture, sha256_hex(&fixture.package_bytes));

    let outcome = handle_upload_system_upgrade(&ctx, &command.encode_to_vec());
    let response = decode_common(&outcome.response);

    assert!(response.success);
    assert_eq!(response.result_code, ResultCode::Success.as_u16() as i32);
    match outcome.post_send_action {
        Some(PostSendAction::StartSystemUpgrade { upgrade_id }) => {
            assert!(upgrade_id.starts_with("upgrade-"));
            assert!(fixture.root().join("current.json").is_file());
        }
        None => panic!("accepted upgrade must return a post-send action"),
    }
}

#[test]
fn rejected_upgrade_returns_existing_result_code_without_action() {
    let fixture = PackageFixture::valid();
    let (ctx, _database) = accepted_context(&fixture, 42);
    let command = command(&fixture, String::new());

    let outcome = handle_upload_system_upgrade(&ctx, &command.encode_to_vec());
    let response = decode_common(&outcome.response);

    assert!(!response.success);
    assert_eq!(
        response.result_code,
        ResultCode::UpgradeChecksumError.as_u16() as i32
    );
    assert!(outcome.post_send_action.is_none());
}

#[test]
fn accepted_upgrade_does_not_update_system_version_or_log_final_success() {
    let fixture = PackageFixture::valid();
    let (ctx, _database) = accepted_context(&fixture, 43);
    let storage = ctx.storage().expect("test storage");
    storage.config_set("system_version", "3.0.1").unwrap();
    let before_log_count = storage.operation_log_count().unwrap();

    let outcome = handle_upload_system_upgrade(
        &ctx,
        &command(&fixture, sha256_hex(&fixture.package_bytes)).encode_to_vec(),
    );

    assert!(decode_common(&outcome.response).success);
    assert_eq!(
        storage
            .config_get("system_version")
            .unwrap()
            .and_then(|value| value.config_value)
            .as_deref(),
        Some("3.0.1")
    );
    assert_eq!(storage.operation_log_count().unwrap(), before_log_count + 2);
    let logs = storage.operation_log_query_by_time(0, i64::MAX).unwrap();
    assert!(logs.iter().all(|log| {
        log.detail
            .as_deref()
            .and_then(|detail| serde_json::from_str::<serde_json::Value>(detail).ok())
            .and_then(|detail| detail["stage"].as_str().map(str::to_string))
            .is_some_and(|stage| stage == "upload" || stage == "validation")
    }));
}

#[test]
fn unauthorized_admin_cannot_prepare_system_upgrade() {
    let fixture = PackageFixture::valid();
    let (ctx, _database) = context_with_session_and_coordinator(&fixture, 44);

    let outcome = handle_upload_system_upgrade(
        &ctx,
        &command(&fixture, sha256_hex(&fixture.package_bytes)).encode_to_vec(),
    );

    let response = decode_common(&outcome.response);
    assert_eq!(
        response.result_code,
        ResultCode::Unauthorized.as_u16() as i32
    );
    assert!(outcome.post_send_action.is_none());
    assert!(!fixture.root().join("current.json").exists());
    assert_eq!(fixture.staging_count(), 0);
    assert_eq!(audit_stages(&ctx), vec![("authorization".into(), 1)]);
}

#[test]
fn expired_admin_cannot_prepare_system_upgrade() {
    let fixture = PackageFixture::valid();
    let (ctx, _database) = context_with_session_and_coordinator(&fixture, 45);
    let storage = ctx.storage().unwrap();
    storage.config_set("auth_status", "authorized").unwrap();
    storage
        .config_set("auth_expire_time", &common::time::now_unix().to_string())
        .unwrap();

    let outcome = handle_upload_system_upgrade(
        &ctx,
        &command(&fixture, sha256_hex(&fixture.package_bytes)).encode_to_vec(),
    );

    assert_eq!(
        decode_common(&outcome.response).result_code,
        ResultCode::Unauthorized.as_u16() as i32
    );
    assert!(!fixture.root().join("current.json").exists());
    assert_eq!(fixture.staging_count(), 0);
    assert_eq!(audit_stages(&ctx), vec![("authorization".into(), 1)]);
}

#[test]
fn authorized_admin_reaches_package_validation_and_audits_failure() {
    let fixture = PackageFixture::valid();
    let (ctx, _database) = accepted_context(&fixture, 46);
    let outcome =
        handle_upload_system_upgrade(&ctx, &command(&fixture, String::new()).encode_to_vec());

    assert_eq!(
        decode_common(&outcome.response).result_code,
        ResultCode::UpgradeChecksumError.as_u16() as i32
    );
    assert_eq!(
        audit_stages(&ctx),
        vec![("validation".into(), 1), ("upload".into(), 0)]
    );
}

#[test]
fn successful_upload_and_validation_audit_has_stable_detail() {
    let fixture = PackageFixture::valid();
    let (ctx, _database) = accepted_context(&fixture, 47);
    let package_sha = sha256_hex(&fixture.package_bytes);
    let outcome = handle_upload_system_upgrade(
        &ctx,
        &command(&fixture, package_sha.clone()).encode_to_vec(),
    );
    assert!(decode_common(&outcome.response).success);

    let logs = ctx
        .storage()
        .unwrap()
        .operation_log_query_by_time(0, i64::MAX)
        .unwrap();
    assert_eq!(logs.len(), 2);
    for log in logs {
        assert_eq!(log.log_type, "program_upgrade");
        assert_eq!(log.action_type.as_deref(), Some("system_upgrade"));
        let detail: serde_json::Value =
            serde_json::from_str(log.detail.as_deref().unwrap()).unwrap();
        assert_eq!(detail["package_sha256"], package_sha);
        assert_eq!(detail["result_code"], ResultCode::Success.as_u16());
        match detail["stage"].as_str().unwrap() {
            "upload" => {
                assert_eq!(log.related_version.as_deref(), Some("V3.1.0"));
                assert!(detail.get("upgrade_id").is_none());
            }
            "validation" => {
                assert_eq!(log.related_version.as_deref(), Some("3.1.0"));
                assert!(detail["upgrade_id"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("upgrade-")));
            }
            stage => panic!("unexpected stage: {stage}"),
        }
    }
}

#[test]
fn audit_write_failure_does_not_change_response_or_scheduling() {
    let fixture = PackageFixture::valid();
    let (mut ctx, _database) = accepted_context(&fixture, 48);
    let storage = Arc::clone(ctx.storage.as_ref().unwrap());
    ctx.audit_service = Arc::new(AuditService::new(
        storage,
        Path::new("/path/that/does/not/exist/device.db"),
    ));

    let outcome = handle_upload_system_upgrade(
        &ctx,
        &command(&fixture, sha256_hex(&fixture.package_bytes)).encode_to_vec(),
    );

    assert!(decode_common(&outcome.response).success);
    assert!(matches!(
        outcome.post_send_action,
        Some(PostSendAction::StartSystemUpgrade { .. })
    ));
    assert!(fixture.root().join("current.json").is_file());
}

fn accepted_context(
    fixture: &PackageFixture,
    seq_id: u32,
) -> (
    protocol_gateway::context::RequestContext,
    tempfile::TempPath,
) {
    let (context, database) = context_with_session_and_coordinator(fixture, seq_id);
    let storage = context.storage().unwrap();
    storage.config_set("auth_status", "authorized").unwrap();
    storage.config_set("auth_expire_time", "0").unwrap();
    (context, database)
}

fn context_with_session_and_coordinator(
    fixture: &PackageFixture,
    seq_id: u32,
) -> (
    protocol_gateway::context::RequestContext,
    tempfile::TempPath,
) {
    let request = request_fixture(seq_id);
    let mut context = request.context;
    let database = request.database;
    context.session = Some(SessionInfo {
        user_id: 1,
        username: "admin".into(),
        role: 0,
        issue_time: 1,
        last_active_time: 1,
        source_ip: "192.0.2.10".into(),
    });
    context.source_ip = "192.0.2.10".into();
    context.system_upgrade_coordinator = Arc::new(coordinator(fixture));
    (context, database)
}

fn audit_stages(ctx: &protocol_gateway::context::RequestContext) -> Vec<(String, i32)> {
    ctx.storage()
        .unwrap()
        .operation_log_query_by_time(0, i64::MAX)
        .unwrap()
        .into_iter()
        .map(|log| {
            let detail: serde_json::Value =
                serde_json::from_str(log.detail.as_deref().unwrap()).unwrap();
            (detail["stage"].as_str().unwrap().to_string(), log.result)
        })
        .collect()
}

fn command(fixture: &PackageFixture, checksum: String) -> CmdUploadSystemUpgrade {
    CmdUploadSystemUpgrade {
        session_token: "validated-by-middleware".into(),
        upgrade_data: fixture.package_bytes.clone(),
        target_version: "V3.1.0".into(),
        sha256_checksum: checksum,
    }
}

fn decode_common(frame: &[u8]) -> RspCommon {
    let (_, payload, _) = codec::try_decode_frame(frame)
        .expect("decode response frame")
        .expect("complete response frame");
    RspCommon::decode(payload.as_slice()).expect("decode common response")
}

fn coordinator(fixture: &PackageFixture) -> UpgradeCoordinator {
    UpgradeCoordinator::new(
        fixture.root(),
        PackageStager::new(fixture.root(), MAX_PACKAGE_SIZE),
        PackageVerifier::new(fixture.key_dir(), Arc::new(MatchingDebInspector)),
        Arc::new(TestUpgradeSource),
        1,
        Arc::new(TestPreflight),
        Arc::new(NoopScheduler),
    )
    .unwrap()
}

struct TestPreflight;

struct TestUpgradeSource;

impl UpgradeSourceReader for TestUpgradeSource {
    fn read(&self) -> Result<UpgradeSourceState, UpgradeError> {
        Ok(UpgradeSourceState {
            current_version: SystemVersion::parse("3.0.1")?,
            current_schema: 1,
        })
    }
}

impl UpgradePreflight for TestPreflight {
    fn check(&self, _request: &UpgradePreflightRequest) -> Result<(), UpgradeError> {
        Ok(())
    }
}

struct NoopScheduler;

impl UpgradeScheduler for NoopScheduler {
    fn start(&self, _upgrade_id: &str) -> Result<(), UpgradeError> {
        Ok(())
    }
}

struct MatchingDebInspector;

impl DebInspector for MatchingDebInspector {
    fn inspect(&self, _deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        Ok(DebMetadata {
            package: "usb-control".into(),
            version: SystemVersion::parse("3.1.0")?,
            architecture: "arm64".into(),
            expanded_size: 4096,
            files: required_deb_files(),
            tls_cert_sha256: TLS_SHA256.into(),
            supported_schema_min: 1,
            supported_schema_max: 2,
            migration_schema_to: 2,
            upgrade_signing_key_id: KEY_ID.into(),
        })
    }
}

fn required_deb_files() -> BTreeSet<PathBuf> {
    [
        "opt/usb-control/bin/usb-control",
        "opt/usb-control/bin/usb-control-updater",
        "opt/usb-control/bin/usb-control-db-migrate",
        "opt/usb-control/install-meta/release.json",
        "opt/usb-control/install-meta/VERSION",
        "lib/systemd/system/usb-control.service",
        "lib/systemd/system/usb-control-updater.service",
        "etc/usb-control/keys/upgrade_verify.id",
        "etc/usb-control/keys/upgrade_verify.pub",
        "etc/usb-control/tls/server.crt",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

struct PackageFixture {
    _temp_dir: TempDir,
    package_bytes: Vec<u8>,
    root: PathBuf,
    key_dir: PathBuf,
}

impl PackageFixture {
    fn valid() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().join("upgrade");
        let key_dir = temp_dir.path().join("keys");
        fs::create_dir_all(&key_dir).unwrap();
        let (private_key, public_key) = sm2::gen_keypair();
        fs::write(key_dir.join("upgrade_verify.id"), format!("{KEY_ID}\n")).unwrap();
        fs::write(key_dir.join("upgrade_verify.pub"), public_key).unwrap();

        let deb = b"minimal-deb-fixture";
        let manifest = serde_json::to_vec(&json!({
            "format_version": 1,
            "product": "usb-control",
            "package_version": "3.1.0",
            "architecture": "arm64",
            "protocol_version": 1,
            "tls_cert_sha256": TLS_SHA256,
            "deb_file": DEB_NAME,
            "deb_size": deb.len(),
            "deb_sha256": sha256_hex(deb),
            "schema_to": 2,
            "signing_key_id": KEY_ID
        }))
        .unwrap();
        let deb_digest = Sha256::digest(deb);
        let mut signing_input = Vec::new();
        signing_input.extend_from_slice(b"USB-CONTROL-UPGRADE-V1\0");
        signing_input.extend_from_slice(&(manifest.len() as u64).to_be_bytes());
        signing_input.extend_from_slice(&manifest);
        signing_input.extend_from_slice(&deb_digest);
        let sm3_digest = hex::decode(sm3::sm3_hash(&signing_input)).unwrap();
        let signature = sm2::Sign::new(&private_key).sign(&sm3_digest);

        let mut package_bytes = Vec::new();
        {
            let mut tar = Builder::new(&mut package_bytes);
            append(&mut tar, "manifest.json", &manifest);
            append(&mut tar, DEB_NAME, deb);
            append(&mut tar, "signature.sm2", &signature);
            tar.finish().unwrap();
        }
        Self {
            _temp_dir: temp_dir,
            package_bytes,
            root,
            key_dir,
        }
    }

    fn root(&self) -> PathBuf {
        self.root.clone()
    }

    fn key_dir(&self) -> PathBuf {
        self.key_dir.clone()
    }

    fn staging_count(&self) -> usize {
        fs::read_dir(self.root.join("staging"))
            .map(|entries| entries.count())
            .unwrap_or(0)
    }
}

fn append(tar: &mut Builder<&mut Vec<u8>>, name: &str, bytes: &[u8]) {
    let mut header = Header::new_ustar();
    header.set_path(name).unwrap();
    header.set_mode(0o600);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(bytes.len() as u64);
    header.set_cksum();
    tar.append(&header, Cursor::new(bytes)).unwrap();
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
