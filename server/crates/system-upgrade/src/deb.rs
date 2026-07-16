//! 正式 DEB 的有界、只读检查器。

use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};

use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use smcrypto::sm2;

use crate::{SystemVersion, UpgradeError};

const RELEASE_METADATA_PATH: &str = "opt/usb-control/install-meta/release.json";
const CERTIFICATE_PATH: &str = "opt/usb-control/defaults/etc/usb-control/tls/server.crt";
const UPGRADE_KEY_ID_PATH: &str = "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.id";
const UPGRADE_PUBLIC_KEY_PATH: &str =
    "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.pub";
const DEFAULT_MAX_ENTRIES: usize = 4096;
const DEFAULT_MAX_EXPANDED_SIZE: u64 = 512 * 1024 * 1024;
const DEFAULT_MAX_SELECTED_FILE_SIZE: u64 = 1024 * 1024;
const MAX_CONTROL_FIELD_SIZE: u64 = 16 * 1024;
const MAX_RELEASE_METADATA_SIZE: u64 = 64 * 1024;
const MAX_CERTIFICATE_SIZE: u64 = 256 * 1024;
const MAX_UPGRADE_KEY_ID_SIZE: u64 = 66;
const MAX_UPGRADE_PUBLIC_KEY_SIZE: u64 = 130;

pub trait DebInspector: Send + Sync {
    fn inspect(&self, deb_path: &Path) -> Result<DebMetadata, UpgradeError>;
}

#[derive(Debug, Clone)]
pub struct DebMetadata {
    pub package: String,
    pub version: SystemVersion,
    pub architecture: String,
    pub expanded_size: u64,
    pub files: BTreeSet<PathBuf>,
    pub tls_cert_sha256: String,
    pub supported_schema_min: u32,
    pub supported_schema_max: u32,
    pub migration_schema_to: u32,
    pub upgrade_signing_key_id: String,
}

pub struct DpkgDebInspector {
    executable: PathBuf,
    max_entries: usize,
    max_expanded_size: u64,
    max_selected_file_size: u64,
}

impl Default for DpkgDebInspector {
    fn default() -> Self {
        Self {
            executable: PathBuf::from("dpkg-deb"),
            max_entries: DEFAULT_MAX_ENTRIES,
            max_expanded_size: DEFAULT_MAX_EXPANDED_SIZE,
            max_selected_file_size: DEFAULT_MAX_SELECTED_FILE_SIZE,
        }
    }
}

impl DebInspector for DpkgDebInspector {
    fn inspect(&self, deb_path: &Path) -> Result<DebMetadata, UpgradeError> {
        let package = self.read_control_field(deb_path, "Package")?;
        let version_text = self.read_control_field(deb_path, "Version")?;
        let architecture = self.read_control_field(deb_path, "Architecture")?;
        let version = SystemVersion::parse(&version_text)
            .map_err(|error| UpgradeError::DebInspection(error.to_string()))?;

        let archive = self.read_filesystem_tar(deb_path)?;
        let release: ReleaseMetadata = serde_json::from_slice(&archive.release_json)
            .map_err(|error| UpgradeError::DebInspection(format!("release.json 无效: {error}")))?;
        validate_release_metadata(&release)?;

        if package != "usb-control"
            || package != release.product
            || architecture != "arm64"
            || architecture != release.architecture
            || version != release.version
        {
            return Err(UpgradeError::DebInspection(
                "DEB 控制字段与 release.json 不一致".into(),
            ));
        }

        validate_required_files(&archive.files)?;
        let migration_schema_to = validate_migrations(&archive.files)?;
        validate_seeds(&archive.files)?;
        validate_upgrade_trust_root(
            &archive.upgrade_key_id,
            &archive.upgrade_public_key,
            &release.upgrade_signing_key_id,
        )?;
        if certificate_sha256(&archive.certificate)? != release.tls_cert_sha256 {
            return Err(UpgradeError::DebInspection("TLS 证书指纹不一致".into()));
        }

        Ok(DebMetadata {
            package,
            version,
            architecture,
            expanded_size: archive.expanded_size,
            files: archive.files,
            tls_cert_sha256: release.tls_cert_sha256,
            supported_schema_min: release.supported_schema_min,
            supported_schema_max: release.supported_schema_max,
            migration_schema_to,
            upgrade_signing_key_id: release.upgrade_signing_key_id,
        })
    }
}

impl DpkgDebInspector {
    fn read_control_field(&self, deb_path: &Path, field: &str) -> Result<String, UpgradeError> {
        let mut child = Command::new(&self.executable)
            .args(["--field"])
            .arg(deb_path)
            .arg(field)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| UpgradeError::DebInspection(error.to_string()))?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| UpgradeError::DebInspection("无法读取 dpkg-deb 输出".into()))?;
        let bytes = match read_bounded(&mut stdout, MAX_CONTROL_FIELD_SIZE) {
            Ok(bytes) => bytes,
            Err(error) => {
                terminate_and_wait(&mut child);
                return Err(error);
            }
        };
        let status = child
            .wait()
            .map_err(|error| UpgradeError::DebInspection(error.to_string()))?;
        if !status.success() {
            return Err(UpgradeError::DebInspection(format!(
                "dpkg-deb 读取 {field} 失败"
            )));
        }
        let value = std::str::from_utf8(&bytes)
            .map_err(|_| UpgradeError::DebInspection(format!("{field} 不是 UTF-8")))?
            .trim()
            .to_string();
        if value.is_empty() {
            return Err(UpgradeError::DebInspection(format!("{field} 为空")));
        }
        Ok(value)
    }

    fn read_filesystem_tar(&self, deb_path: &Path) -> Result<InspectedArchive, UpgradeError> {
        let mut child = Command::new(&self.executable)
            .arg("--fsys-tarfile")
            .arg(deb_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| UpgradeError::DebInspection(error.to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| UpgradeError::DebInspection("无法读取 DEB 文件系统流".into()))?;
        let archive = match inspect_tar_stream(
            stdout,
            self.max_entries,
            self.max_expanded_size,
            self.max_selected_file_size,
        ) {
            Ok(archive) => archive,
            Err(error) => {
                terminate_and_wait(&mut child);
                return Err(error);
            }
        };
        let status = child
            .wait()
            .map_err(|error| UpgradeError::DebInspection(error.to_string()))?;
        if !status.success() {
            return Err(UpgradeError::DebInspection(
                "dpkg-deb 文件系统检查失败".into(),
            ));
        }
        Ok(archive)
    }
}

struct InspectedArchive {
    expanded_size: u64,
    files: BTreeSet<PathBuf>,
    release_json: Vec<u8>,
    certificate: Vec<u8>,
    upgrade_key_id: Vec<u8>,
    upgrade_public_key: Vec<u8>,
}

fn inspect_tar_stream<R: Read>(
    reader: R,
    max_entries: usize,
    max_expanded_size: u64,
    max_selected_file_size: u64,
) -> Result<InspectedArchive, UpgradeError> {
    let mut reader = reader;
    let mut files = BTreeSet::new();
    let mut release_json = None;
    let mut certificate = None;
    let mut upgrade_key_id = None;
    let mut upgrade_public_key = None;
    let mut total_size = 0u64;
    let mut entry_count = 0usize;

    loop {
        let header = read_tar_block(&mut reader)?;
        if header.iter().all(|byte| *byte == 0) {
            let second = read_tar_block(&mut reader)?;
            if second.iter().any(|byte| *byte != 0) {
                return Err(UpgradeError::DebInspection("DEB tar 结束块无效".into()));
            }
            reject_nonzero_tar_tail(&mut reader)?;
            break;
        }
        entry_count += 1;
        if entry_count > max_entries {
            return Err(UpgradeError::DebInspection("DEB 条目数量超过上限".into()));
        }
        validate_tar_checksum(&header)?;
        let entry_type = header[156];
        if matches!(entry_type, b'L' | b'K' | b'x' | b'g') {
            return Err(UpgradeError::DebInspection(
                "DEB tar 禁止 GNU longname 或 PAX 扩展".into(),
            ));
        }
        let size = parse_tar_octal(&header[124..136], "size")?;
        total_size = total_size
            .checked_add(size)
            .ok_or_else(|| UpgradeError::DebInspection("DEB 展开大小溢出".into()))?;
        if total_size > max_expanded_size {
            return Err(UpgradeError::DebInspection("DEB 展开大小超过上限".into()));
        }
        let path = parse_ustar_path(&header)?;
        match entry_type {
            b'5' if is_allowed_release_directory(&path) => {
                skip_exact(&mut reader, size)?;
                skip_tar_padding(&mut reader, size)?;
                continue;
            }
            b'5' => {
                return Err(UpgradeError::DebInspection(format!(
                    "DEB 包含发布边界外目录: {}",
                    path.display()
                )));
            }
            0 | b'0' if is_allowed_release_file(&path) => {}
            0 | b'0' => {
                return Err(UpgradeError::DebInspection(format!(
                    "DEB 包含发布边界外文件: {}",
                    path.display()
                )));
            }
            _ => {
                return Err(UpgradeError::DebInspection(format!(
                    "DEB 禁止链接或特殊文件: {}",
                    path.display()
                )));
            }
        }
        if !files.insert(path.clone()) {
            return Err(UpgradeError::DebInspection("DEB 包含重复文件".into()));
        }
        let selected = match path.to_str() {
            Some(RELEASE_METADATA_PATH) => Some((&mut release_json, MAX_RELEASE_METADATA_SIZE)),
            Some(CERTIFICATE_PATH) => Some((&mut certificate, MAX_CERTIFICATE_SIZE)),
            Some(UPGRADE_KEY_ID_PATH) => Some((&mut upgrade_key_id, MAX_UPGRADE_KEY_ID_SIZE)),
            Some(UPGRADE_PUBLIC_KEY_PATH) => {
                Some((&mut upgrade_public_key, MAX_UPGRADE_PUBLIC_KEY_SIZE))
            }
            _ => None,
        };
        if let Some((slot, file_limit)) = selected {
            if slot.is_some() || size > max_selected_file_size.min(file_limit) {
                return Err(UpgradeError::DebInspection(
                    "DEB 关键文件重复或超过大小上限".into(),
                ));
            }
            *slot = Some(read_exact_vec(&mut reader, size)?);
        } else {
            skip_exact(&mut reader, size)?;
        }
        skip_tar_padding(&mut reader, size)?;
    }

    Ok(InspectedArchive {
        expanded_size: total_size,
        files,
        release_json: release_json
            .ok_or_else(|| UpgradeError::DebInspection("缺少 release.json".into()))?,
        certificate: certificate
            .ok_or_else(|| UpgradeError::DebInspection("缺少 TLS 证书".into()))?,
        upgrade_key_id: upgrade_key_id
            .ok_or_else(|| UpgradeError::DebInspection("缺少升级公钥标识".into()))?,
        upgrade_public_key: upgrade_public_key
            .ok_or_else(|| UpgradeError::DebInspection("缺少升级验签公钥".into()))?,
    })
}

/// 计算只包含一张 PEM 证书时的 DER SHA-256 指纹。
pub fn certificate_sha256(pem: &[u8]) -> Result<String, UpgradeError> {
    let certificate_der = decode_single_pem_certificate(pem)?;
    Ok(hex::encode(Sha256::digest(certificate_der)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseMetadata {
    format_version: u32,
    product: String,
    version: SystemVersion,
    architecture: String,
    supported_schema_min: u32,
    supported_schema_max: u32,
    tls_cert_sha256: String,
    upgrade_signing_key_id: String,
}

fn validate_release_metadata(metadata: &ReleaseMetadata) -> Result<(), UpgradeError> {
    if metadata.format_version != 1
        || metadata.product != "usb-control"
        || metadata.architecture != "arm64"
        || metadata.supported_schema_min > metadata.supported_schema_max
        || !is_lower_hex_64(&metadata.tls_cert_sha256)
        || !is_key_id(&metadata.upgrade_signing_key_id)
    {
        return Err(UpgradeError::DebInspection(
            "release.json 业务字段非法".into(),
        ));
    }
    Ok(())
}

fn validate_required_files(files: &BTreeSet<PathBuf>) -> Result<(), UpgradeError> {
    const REQUIRED: &[&str] = &[
        "opt/usb-control/bin/usb-control",
        "opt/usb-control/bin/usb-control-updater",
        "opt/usb-control/bin/usb-control-db-migrate",
        "lib/systemd/system/usb-control.service",
        "lib/systemd/system/usb-control-updater.service",
        RELEASE_METADATA_PATH,
        UPGRADE_KEY_ID_PATH,
        UPGRADE_PUBLIC_KEY_PATH,
        CERTIFICATE_PATH,
    ];
    if REQUIRED.iter().any(|path| !files.contains(Path::new(path))) {
        return Err(UpgradeError::DebInspection("DEB 缺少正式发布文件".into()));
    }
    Ok(())
}

fn validate_migrations(files: &BTreeSet<PathBuf>) -> Result<u32, UpgradeError> {
    validate_sql_sequence(files, Path::new("opt/usb-control/db/migrations"), "迁移")
}

fn validate_seeds(files: &BTreeSet<PathBuf>) -> Result<u32, UpgradeError> {
    validate_sql_sequence(files, Path::new("opt/usb-control/db/seeds"), "种子")
}

fn validate_sql_sequence(
    files: &BTreeSet<PathBuf>,
    directory: &Path,
    label: &str,
) -> Result<u32, UpgradeError> {
    let mut versions = BTreeSet::new();
    for path in files.iter().filter(|path| path.starts_with(directory)) {
        if path.parent() != Some(directory) {
            return Err(UpgradeError::DebInspection(format!(
                "{label}文件不得嵌套目录"
            )));
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| UpgradeError::DebInspection(format!("{label}文件名不是 UTF-8")))?;
        let (prefix, suffix) = name
            .split_once('_')
            .ok_or_else(|| UpgradeError::DebInspection(format!("{label}文件名格式非法")))?;
        if prefix.len() != 4
            || !prefix.bytes().all(|byte| byte.is_ascii_digit())
            || suffix.is_empty()
            || !suffix.ends_with(".sql")
        {
            return Err(UpgradeError::DebInspection(format!(
                "{label}文件名格式非法"
            )));
        }
        let version: u32 = prefix
            .parse()
            .map_err(|_| UpgradeError::DebInspection(format!("{label}版本非法")))?;
        if version == 0 || !versions.insert(version) {
            return Err(UpgradeError::DebInspection(format!(
                "{label}版本重复或为零"
            )));
        }
    }
    let maximum = versions
        .last()
        .copied()
        .ok_or_else(|| UpgradeError::DebInspection(format!("DEB 缺少数据库{label}")))?;
    if versions.len() != maximum as usize
        || !(1..=maximum).all(|version| versions.contains(&version))
    {
        return Err(UpgradeError::DebInspection(format!("{label}版本存在缺口")));
    }
    Ok(maximum)
}

fn validate_upgrade_trust_root(
    key_id: &[u8],
    public_key: &[u8],
    metadata_key_id: &str,
) -> Result<(), UpgradeError> {
    let key_id = strict_single_line(key_id, "升级公钥标识")?;
    let public_key = strict_single_line(public_key, "升级公钥")?;
    if !is_key_id(key_id)
        || key_id != metadata_key_id
        || public_key.len() != 128
        || !public_key.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !valid_sm2_public_key(public_key)
    {
        return Err(UpgradeError::DebInspection("目标升级信任根非法".into()));
    }
    Ok(())
}

fn valid_sm2_public_key(public_key: &str) -> bool {
    sm2::pubkey_valid(public_key)
}

fn strict_single_line<'a>(bytes: &'a [u8], label: &str) -> Result<&'a str, UpgradeError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| UpgradeError::DebInspection(format!("{label} 不是 UTF-8")))?;
    let line = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text);
    if line.is_empty() || line.contains(['\r', '\n']) {
        return Err(UpgradeError::DebInspection(format!("{label} 必须为单行")));
    }
    Ok(line)
}

fn is_allowed_release_file(path: &Path) -> bool {
    const EXACT: &[&str] = &[
        "opt/usb-control/bin/usb-control",
        "opt/usb-control/bin/usb-control-updater",
        "opt/usb-control/bin/usb-control-db-migrate",
        RELEASE_METADATA_PATH,
        "lib/systemd/system/usb-control.service",
        "lib/systemd/system/usb-control-updater.service",
        "opt/usb-control/defaults/etc/usb-control/keys/license_verify.pub",
        "opt/usb-control/defaults/etc/usb-control/keys/sm4_policy.key",
        "opt/usb-control/defaults/etc/usb-control/keys/sm2_policy.key",
        "opt/usb-control/defaults/etc/usb-control/keys/sm2_policy.pub",
        UPGRADE_KEY_ID_PATH,
        UPGRADE_PUBLIC_KEY_PATH,
        CERTIFICATE_PATH,
        "opt/usb-control/defaults/etc/usb-control/tls/server.key",
        "opt/usb-control/defaults/etc/usb-control/tls/server.crt.sha256",
        "opt/usb-control/defaults/etc/usb-control/usb-control.toml",
    ];
    if EXACT.iter().any(|allowed| path == Path::new(allowed)) {
        return true;
    }
    if matches!(
        path.parent(),
        Some(parent)
            if parent == Path::new("opt/usb-control/db/migrations")
                || parent == Path::new("opt/usb-control/db/seeds")
    ) {
        return path.extension().is_some_and(|extension| extension == "sql");
    }
    false
}

fn is_allowed_release_directory(path: &Path) -> bool {
    const EXACT: &[&str] = &[
        ".",
        "opt",
        "opt/usb-control",
        "opt/usb-control/bin",
        "opt/usb-control/install-meta",
        "opt/usb-control/defaults",
        "opt/usb-control/defaults/etc",
        "opt/usb-control/defaults/etc/usb-control",
        "opt/usb-control/defaults/etc/usb-control/keys",
        "opt/usb-control/defaults/etc/usb-control/tls",
        "opt/usb-control/db",
        "opt/usb-control/db/migrations",
        "opt/usb-control/db/seeds",
        "lib",
        "lib/systemd",
        "lib/systemd/system",
    ];
    EXACT.iter().any(|allowed| path == Path::new(allowed))
}

fn read_tar_block<R: Read>(reader: &mut R) -> Result<[u8; 512], UpgradeError> {
    let mut block = [0u8; 512];
    reader
        .read_exact(&mut block)
        .map_err(|error| UpgradeError::DebInspection(format!("DEB tar 截断: {error}")))?;
    Ok(block)
}

fn validate_tar_checksum(header: &[u8; 512]) -> Result<(), UpgradeError> {
    let expected = parse_tar_octal(&header[148..156], "checksum")?;
    let actual: u64 = header
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            if (148..156).contains(&index) {
                b' ' as u64
            } else {
                *byte as u64
            }
        })
        .sum();
    if expected != actual {
        return Err(UpgradeError::DebInspection("DEB tar 校验和无效".into()));
    }
    Ok(())
}

fn parse_tar_octal(field: &[u8], label: &str) -> Result<u64, UpgradeError> {
    if field.first().is_some_and(|byte| byte & 0x80 != 0) {
        return Err(UpgradeError::DebInspection(format!(
            "DEB tar {label} 禁止 base-256 编码"
        )));
    }
    let text = std::str::from_utf8(field)
        .map_err(|_| UpgradeError::DebInspection(format!("DEB tar {label} 非法")))?
        .trim_matches(['\0', ' ']);
    if text.is_empty() {
        return Ok(0);
    }
    if !text.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) {
        return Err(UpgradeError::DebInspection(format!(
            "DEB tar {label} 非八进制"
        )));
    }
    u64::from_str_radix(text, 8)
        .map_err(|_| UpgradeError::DebInspection(format!("DEB tar {label} 溢出")))
}

fn parse_ustar_path(header: &[u8; 512]) -> Result<PathBuf, UpgradeError> {
    if !header[257..263].starts_with(b"ustar") {
        return Err(UpgradeError::DebInspection(
            "DEB tar 不是 ustar 格式".into(),
        ));
    }
    let name = tar_text_field(&header[..100])?;
    let prefix = tar_text_field(&header[345..500])?;
    let combined = if prefix.is_empty() {
        name
    } else {
        format!("{prefix}/{name}")
    };
    normalize_deb_path(Path::new(&combined))
}

fn tar_text_field(field: &[u8]) -> Result<String, UpgradeError> {
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    if field[end..].iter().any(|byte| *byte != 0) {
        return Err(UpgradeError::DebInspection("DEB tar 文本字段非法".into()));
    }
    std::str::from_utf8(&field[..end])
        .map(str::to_string)
        .map_err(|_| UpgradeError::DebInspection("DEB tar 路径不是 UTF-8".into()))
}

fn read_exact_vec<R: Read>(reader: &mut R, size: u64) -> Result<Vec<u8>, UpgradeError> {
    let length = usize::try_from(size)
        .map_err(|_| UpgradeError::DebInspection("DEB 关键文件大小溢出".into()))?;
    let mut bytes = vec![0u8; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| UpgradeError::DebInspection(format!("DEB tar 截断: {error}")))?;
    Ok(bytes)
}

fn skip_exact<R: Read>(reader: &mut R, mut size: u64) -> Result<(), UpgradeError> {
    let mut buffer = [0u8; 8192];
    while size > 0 {
        let requested =
            usize::try_from(size.min(buffer.len() as u64)).expect("bounded read size fits usize");
        reader
            .read_exact(&mut buffer[..requested])
            .map_err(|error| UpgradeError::DebInspection(format!("DEB tar 截断: {error}")))?;
        size -= requested as u64;
    }
    Ok(())
}

fn skip_tar_padding<R: Read>(reader: &mut R, size: u64) -> Result<(), UpgradeError> {
    let padding = (512 - size % 512) % 512;
    skip_exact(reader, padding)
}

fn reject_nonzero_tar_tail<R: Read>(reader: &mut R) -> Result<(), UpgradeError> {
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| UpgradeError::DebInspection(error.to_string()))?;
        if read == 0 {
            return Ok(());
        }
        if buffer[..read].iter().any(|byte| *byte != 0) {
            return Err(UpgradeError::DebInspection("DEB tar 包含尾随数据".into()));
        }
    }
}

fn normalize_deb_path(path: &Path) -> Result<PathBuf, UpgradeError> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => normalized.push(value),
            _ => return Err(UpgradeError::DebInspection("DEB 文件路径非法".into())),
        }
    }
    if normalized.as_os_str().is_empty() && path.components().all(|part| part == Component::CurDir)
    {
        return Ok(PathBuf::from("."));
    }
    if normalized.as_os_str().is_empty() {
        return Err(UpgradeError::DebInspection("DEB 文件路径为空".into()));
    }
    Ok(normalized)
}

fn read_bounded<R: Read>(reader: &mut R, maximum_size: u64) -> Result<Vec<u8>, UpgradeError> {
    let mut bytes = Vec::new();
    reader
        .take(maximum_size + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| UpgradeError::DebInspection(error.to_string()))?;
    if bytes.len() as u64 > maximum_size {
        return Err(UpgradeError::DebInspection("读取内容超过大小上限".into()));
    }
    Ok(bytes)
}

fn decode_single_pem_certificate(pem: &[u8]) -> Result<Vec<u8>, UpgradeError> {
    let text = std::str::from_utf8(pem)
        .map_err(|_| UpgradeError::DebInspection("TLS 证书不是 UTF-8 PEM".into()))?;
    let mut inside = false;
    let mut ended = false;
    let mut encoded = String::new();
    for line in text.lines() {
        match line.trim() {
            "-----BEGIN CERTIFICATE-----" if !inside && !ended => inside = true,
            "-----END CERTIFICATE-----" if inside => {
                inside = false;
                ended = true;
            }
            value if inside => encoded.push_str(value),
            value if !value.is_empty() => {
                return Err(UpgradeError::DebInspection("TLS PEM 包含额外内容".into()));
            }
            _ => {}
        }
    }
    if inside || !ended || encoded.is_empty() {
        return Err(UpgradeError::DebInspection("TLS PEM 结构无效".into()));
    }
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| UpgradeError::DebInspection("TLS PEM Base64 无效".into()))
}

fn terminate_and_wait(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
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
