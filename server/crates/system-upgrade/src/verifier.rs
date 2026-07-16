//! 升级包签名、兼容性和 DEB 元数据校验。

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use smcrypto::sm2;

use crate::{
    upgrade_signing_digest, DebInspector, DebMetadata, StagedPackage, SystemVersion, UpgradeError,
};

pub struct VerificationContext {
    pub current_version: SystemVersion,
    pub current_schema: u32,
    pub supported_schema_max: u32,
    pub protocol_version: u32,
    pub client_target_version: String,
    pub client_sha256: String,
}

pub struct VerifiedPackage {
    pub staged: StagedPackage,
    pub deb_sha256: [u8; 32],
    pub deb_metadata: DebMetadata,
}

pub struct PackageVerifier {
    key_dir: PathBuf,
    deb_inspector: Arc<dyn DebInspector>,
}

impl PackageVerifier {
    pub fn new(key_dir: PathBuf, deb_inspector: Arc<dyn DebInspector>) -> Self {
        Self {
            key_dir,
            deb_inspector,
        }
    }

    pub fn verify(
        &self,
        package: StagedPackage,
        context: &VerificationContext,
    ) -> Result<VerifiedPackage, UpgradeError> {
        let package_sha256 = sha256_file(&package.package_path)?;
        let deb_sha256 = sha256_file(&package.deb_path)?;
        let actual_digest = hex::encode(deb_sha256);
        if !is_lower_hex_64(&package.manifest.deb_sha256)
            || package.manifest.deb_sha256 != actual_digest
        {
            return Err(UpgradeError::DigestMismatch);
        }

        self.verify_signature(&package, &deb_sha256)?;
        validate_manifest(&package, context, &hex::encode(package_sha256))?;

        let deb_metadata = self.deb_inspector.inspect(&package.deb_path)?;
        validate_deb_metadata(&package, context, &deb_metadata)?;
        Ok(VerifiedPackage {
            staged: package,
            deb_sha256,
            deb_metadata,
        })
    }

    fn verify_signature(
        &self,
        package: &StagedPackage,
        deb_sha256: &[u8; 32],
    ) -> Result<(), UpgradeError> {
        if !is_key_id(&package.manifest.signing_key_id) {
            return Err(UpgradeError::SignatureInvalid);
        }
        let active_key_id = read_small_text(&self.key_dir.join("upgrade_verify.id"), 65)?;
        if active_key_id != package.manifest.signing_key_id {
            return Err(UpgradeError::SignatureInvalid);
        }
        let public_key = read_small_text(&self.key_dir.join("upgrade_verify.pub"), 129)?;
        if public_key.len() != 128 || !public_key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(UpgradeError::SignatureInvalid);
        }

        let digest = upgrade_signing_digest(&package.manifest_raw, deb_sha256)?;

        let verified = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            sm2::Verify::new(&public_key).verify(&digest, &package.signature)
        }))
        .unwrap_or(false);
        if !verified {
            return Err(UpgradeError::SignatureInvalid);
        }
        Ok(())
    }
}

fn validate_manifest(
    package: &StagedPackage,
    context: &VerificationContext,
    package_digest: &str,
) -> Result<(), UpgradeError> {
    let manifest = &package.manifest;
    if manifest.format_version != 1 {
        return Err(UpgradeError::Format("不支持的升级包格式版本".into()));
    }
    if manifest.product != "usb-control" {
        return Err(UpgradeError::ProductMismatch);
    }
    if manifest.architecture != "arm64" {
        return Err(UpgradeError::ArchitectureMismatch);
    }
    if !is_lower_hex_64(&manifest.tls_cert_sha256) {
        return Err(UpgradeError::Format("TLS 证书指纹格式非法".into()));
    }
    if manifest.package_version <= context.current_version {
        return Err(UpgradeError::VersionNotGreater);
    }
    if context.current_version < manifest.minimum_current_version {
        return Err(UpgradeError::Format("当前版本低于最低升级版本".into()));
    }
    if manifest.protocol_version != context.protocol_version {
        return Err(UpgradeError::Format("升级包协议版本不兼容".into()));
    }
    if manifest.schema_from != context.current_schema
        || manifest.schema_to < manifest.schema_from
        || manifest.schema_to > context.supported_schema_max
    {
        return Err(UpgradeError::SchemaIncompatible);
    }
    let client_version_text = context
        .client_target_version
        .strip_prefix('v')
        .or_else(|| context.client_target_version.strip_prefix('V'))
        .unwrap_or(&context.client_target_version);
    let client_version = SystemVersion::parse(client_version_text)?;
    if client_version != manifest.package_version
        || !is_lower_hex_64(&context.client_sha256)
        || context.client_sha256 != package_digest
    {
        return Err(UpgradeError::DigestMismatch);
    }
    Ok(())
}

fn validate_deb_metadata(
    package: &StagedPackage,
    context: &VerificationContext,
    metadata: &DebMetadata,
) -> Result<(), UpgradeError> {
    let manifest = &package.manifest;
    if metadata.package != manifest.product {
        return Err(UpgradeError::ProductMismatch);
    }
    if metadata.version != manifest.package_version {
        return Err(UpgradeError::DebInspection(
            "DEB 版本与 manifest 不一致".into(),
        ));
    }
    if metadata.architecture != manifest.architecture {
        return Err(UpgradeError::ArchitectureMismatch);
    }
    if metadata.tls_cert_sha256 != manifest.tls_cert_sha256 {
        return Err(UpgradeError::DebInspection(
            "TLS 证书指纹与 manifest 不一致".into(),
        ));
    }
    if metadata.supported_schema_min > context.current_schema
        || metadata.supported_schema_max < package.manifest.schema_to
        || metadata.supported_schema_min > metadata.supported_schema_max
        || metadata.migration_schema_to != package.manifest.schema_to
    {
        return Err(UpgradeError::SchemaIncompatible);
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<[u8; 32], UpgradeError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().into())
}

fn read_small_text(path: &Path, maximum_size: u64) -> Result<String, UpgradeError> {
    if fs::metadata(path)?.len() > maximum_size {
        return Err(UpgradeError::SignatureInvalid);
    }
    let value = fs::read_to_string(path)?;
    let line = value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix('\n'))
        .unwrap_or(&value);
    if line.is_empty() || line.contains(['\r', '\n']) {
        return Err(UpgradeError::SignatureInvalid);
    }
    Ok(line.to_string())
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_key_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value.len() <= 64
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
