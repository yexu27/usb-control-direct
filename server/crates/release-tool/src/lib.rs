//! 正式发布材料生成工具的可测试核心。

use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use smcrypto::sm2;
use system_upgrade::{
    upgrade_signing_digest, DebInspector, DpkgDebInspector, PackageStager, PackageVerifier,
    UpgradeError, UpgradeManifest, VerificationContext,
};
use tar::{Builder, Header};
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReleaseToolError {
    #[error("升级密钥标识不符合运行时契约")]
    InvalidKeyId,
    #[error("升级密钥目标已存在: {0}")]
    TargetExists(PathBuf),
    #[error("发布工具 IO 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("发布材料原子落盘失败: {0}")]
    Persist(String),
    #[error("发布输入非法: {0}")]
    InvalidInput(String),
    #[error("发布 JSON 失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("系统升级领域校验失败: {0}")]
    Domain(#[from] UpgradeError),
}

pub struct BuildBinRequest<'a> {
    pub deb_path: &'a Path,
    pub key_dir: &'a Path,
    pub output_path: &'a Path,
}

pub fn build_bin(request: BuildBinRequest<'_>) -> Result<UpgradeManifest, ReleaseToolError> {
    build_bin_with_inspector(request, Arc::new(DpkgDebInspector::default()))
}

pub fn build_bin_with_inspector(
    request: BuildBinRequest<'_>,
    inspector: Arc<dyn DebInspector>,
) -> Result<UpgradeManifest, ReleaseToolError> {
    let metadata = inspector.inspect(request.deb_path)?;
    if metadata.migration_schema_to != metadata.supported_schema_max {
        return Err(ReleaseToolError::InvalidInput(
            "目标 DEB 的迁移终点与支持 Schema 上限不一致".into(),
        ));
    }
    let deb_name = request
        .deb_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ReleaseToolError::InvalidInput("DEB 文件名不是 UTF-8".into()))?;
    let expected_name = format!("usb-control_V{}_arm64.deb", metadata.version);
    if deb_name != expected_name {
        return Err(ReleaseToolError::InvalidInput(
            "DEB 文件名与目标版本不一致".into(),
        ));
    }
    let deb = fs::read(request.deb_path)?;
    let deb_sha256: [u8; 32] = Sha256::digest(&deb).into();
    let signing_key_id = read_strict_line(&request.key_dir.join("upgrade_verify.id"), 64)?;
    validate_key_id(&signing_key_id)?;
    let manifest = UpgradeManifest {
        format_version: 1,
        product: metadata.package,
        package_version: metadata.version,
        architecture: metadata.architecture,
        protocol_version: 1,
        tls_cert_sha256: metadata.tls_cert_sha256,
        deb_file: deb_name.into(),
        deb_size: deb.len() as u64,
        deb_sha256: hex::encode(deb_sha256),
        schema_to: metadata.supported_schema_max,
        signing_key_id,
    };
    let manifest_raw = serde_json::to_vec(&manifest)?;
    let digest = upgrade_signing_digest(&manifest_raw, &deb_sha256)?;
    let private_key = read_strict_line(&request.key_dir.join("upgrade_sign.key"), 64)?;
    if private_key.len() != 64 || !private_key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ReleaseToolError::InvalidInput(
            "升级签名私钥格式非法".into(),
        ));
    }
    let public_key = read_strict_line(&request.key_dir.join("upgrade_verify.pub"), 128)?;
    if public_key.len() != 128 || !public_key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ReleaseToolError::InvalidInput(
            "升级验签公钥格式非法".into(),
        ));
    }
    let signature = sm2::Sign::new(&private_key).sign(&digest);
    if !sm2::Verify::new(&public_key).verify(&digest, &signature) {
        return Err(ReleaseToolError::InvalidInput(
            "升级签名密钥对不匹配".into(),
        ));
    }

    let parent = request
        .output_path
        .parent()
        .ok_or_else(|| ReleaseToolError::InvalidInput("BIN 输出缺少父目录".into()))?;
    fs::create_dir_all(parent)?;
    if request.output_path.exists() {
        return Err(ReleaseToolError::TargetExists(
            request.output_path.to_path_buf(),
        ));
    }
    let mut temporary = NamedTempFile::new_in(parent)?;
    {
        let mut archive = Builder::new(&mut temporary);
        append_release_entry(&mut archive, "manifest.json", &manifest_raw)?;
        append_release_entry(&mut archive, deb_name, &deb)?;
        append_release_entry(&mut archive, "signature.sm2", &signature)?;
        archive.finish()?;
    }
    temporary.as_file().sync_all()?;
    temporary
        .persist_noclobber(request.output_path)
        .map_err(|error| ReleaseToolError::Persist(error.error.to_string()))?;
    File::open(parent)?.sync_all()?;
    Ok(manifest)
}

pub fn verify_bin(bin_path: &Path, key_dir: &Path) -> Result<UpgradeManifest, ReleaseToolError> {
    verify_bin_with_inspector(bin_path, key_dir, Arc::new(DpkgDebInspector::default()))
}

pub fn verify_bin_with_inspector(
    bin_path: &Path,
    key_dir: &Path,
    inspector: Arc<dyn DebInspector>,
) -> Result<UpgradeManifest, ReleaseToolError> {
    let package = fs::read(bin_path)?;
    let package_sha256 = hex::encode(Sha256::digest(&package));
    let temporary = tempfile::tempdir()?;
    let staged = PackageStager::new(temporary.path().join("upgrade"), 128 * 1024 * 1024)
        .stage("release-tool-verify", &package)?;
    let metadata = inspector.inspect(&staged.deb_path)?;
    let context = VerificationContext {
        current_schema: metadata.supported_schema_min,
        protocol_version: staged.manifest.protocol_version,
        client_target_version: staged.manifest.package_version.to_string(),
        client_sha256: package_sha256,
    };
    let verified =
        PackageVerifier::new(key_dir.to_path_buf(), inspector).verify(staged, &context)?;
    Ok(verified.staged.manifest)
}

fn append_release_entry<W: Write>(
    archive: &mut Builder<W>,
    name: &str,
    bytes: &[u8],
) -> Result<(), ReleaseToolError> {
    let mut header = Header::new_ustar();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(bytes.len() as u64);
    header.set_cksum();
    archive.append_data(&mut header, name, bytes)?;
    Ok(())
}

fn read_strict_line(path: &Path, maximum: usize) -> Result<String, ReleaseToolError> {
    let raw = fs::read_to_string(path)?;
    let value = raw
        .strip_suffix("\r\n")
        .or_else(|| raw.strip_suffix('\n'))
        .unwrap_or(&raw);
    if value.is_empty() || value.len() > maximum || value.contains(['\r', '\n']) {
        return Err(ReleaseToolError::InvalidInput(format!(
            "发布材料不是有效单行文本: {}",
            path.display()
        )));
    }
    Ok(value.into())
}

pub fn generate_key(key_id: &str, key_dir: &Path) -> Result<(), ReleaseToolError> {
    validate_key_id(key_id)?;
    fs::create_dir_all(key_dir)?;
    let targets = [
        key_dir.join("upgrade_sign.key"),
        key_dir.join("upgrade_verify.pub"),
        key_dir.join("upgrade_verify.id"),
    ];
    if let Some(existing) = targets.iter().find(|path| path.exists()) {
        return Err(ReleaseToolError::TargetExists(existing.clone()));
    }

    let (private_key, public_key) = smcrypto::sm2::gen_keypair();
    let temporary = [
        prepared_file(key_dir, private_key.as_bytes(), 0o600)?,
        prepared_file(key_dir, public_key.as_bytes(), 0o644)?,
        prepared_file(key_dir, key_id.as_bytes(), 0o644)?,
    ];
    let mut persisted = Vec::new();
    for (source, target) in temporary.into_iter().zip(&targets) {
        if let Err(error) = source.persist_noclobber(target) {
            for path in &persisted {
                let _ = fs::remove_file(path);
            }
            return Err(ReleaseToolError::Persist(error.error.to_string()));
        }
        persisted.push(target.clone());
    }
    if let Err(error) = File::open(key_dir).and_then(|directory| directory.sync_all()) {
        for path in &persisted {
            let _ = fs::remove_file(path);
        }
        return Err(ReleaseToolError::Io(error));
    }
    Ok(())
}

fn prepared_file(
    directory: &Path,
    value: &[u8],
    mode: u32,
) -> Result<NamedTempFile, ReleaseToolError> {
    let mut file = NamedTempFile::new_in(directory)?;
    file.as_file()
        .set_permissions(fs::Permissions::from_mode(mode))?;
    file.write_all(value)?;
    file.write_all(b"\n")?;
    file.as_file().sync_all()?;
    Ok(file)
}

fn validate_key_id(value: &str) -> Result<(), ReleaseToolError> {
    let mut bytes = value.bytes();
    if !matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || value.len() > 64
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ReleaseToolError::InvalidKeyId);
    }
    Ok(())
}
