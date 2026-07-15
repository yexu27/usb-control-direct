//! 升级容器的受控落盘与流式解析。

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{StagedPackage, UpgradeError, UpgradeManifest};

const MAX_MANIFEST_SIZE: u64 = 64 * 1024;
const MAX_SIGNATURE_SIZE: u64 = 16 * 1024;

/// 把上传字节原子落盘并解析为受控 staging 文件。
pub struct PackageStager {
    root_dir: PathBuf,
    max_package_size: u64,
}

impl PackageStager {
    pub fn new(root_dir: PathBuf, max_package_size: u64) -> Self {
        Self {
            root_dir,
            max_package_size,
        }
    }

    pub fn stage(
        &self,
        upgrade_id: &str,
        package_bytes: &[u8],
    ) -> Result<StagedPackage, UpgradeError> {
        validate_upgrade_id(upgrade_id)?;
        if package_bytes.len() as u64 > self.max_package_size {
            return Err(UpgradeError::Format("升级包超过大小上限".into()));
        }

        let staging_dir = self.root_dir.join("staging");
        create_private_dir_all(&staging_dir)?;
        let temporary = staging_dir.join(format!("{upgrade_id}.tmp"));
        let final_dir = staging_dir.join(upgrade_id);
        if final_dir.exists() || temporary.exists() {
            return Err(UpgradeError::State("升级 staging 已存在".into()));
        }
        create_private_dir(&temporary)?;

        let result = self.stage_in(&temporary, &final_dir, package_bytes);
        if result.is_err() {
            let _ = fs::remove_dir_all(&temporary);
        }
        result
    }

    /// 重新打开既有 staging，并重新解析原始容器及绑定预期包摘要。
    pub fn reopen(
        &self,
        upgrade_id: &str,
        expected_package_sha256: &str,
    ) -> Result<StagedPackage, UpgradeError> {
        validate_upgrade_id(upgrade_id)?;
        if !is_lower_hex_64(expected_package_sha256) {
            return Err(UpgradeError::DigestMismatch);
        }
        let root = self.root_dir.join("staging").join(upgrade_id);
        validate_staging_layout(&root)?;
        let package_path = root.join("package.bin");
        let package_size = fs::metadata(&package_path)?.len();
        if package_size > self.max_package_size
            || sha256_file(&package_path)? != expected_package_sha256
        {
            return Err(UpgradeError::DigestMismatch);
        }
        let parsed = parse_archive(&package_path, None, self.max_package_size)?;
        Ok(StagedPackage {
            manifest_path: root.join("manifest.json"),
            deb_path: root.join("payload.deb"),
            signature_path: root.join("signature.sm2"),
            root,
            package_path,
            manifest_raw: parsed.manifest_raw,
            manifest: parsed.manifest,
            signature: parsed.signature,
        })
    }

    fn stage_in(
        &self,
        temporary: &Path,
        final_dir: &Path,
        package_bytes: &[u8],
    ) -> Result<StagedPackage, UpgradeError> {
        let package_path = temporary.join("package.bin");
        write_new_file(&package_path, package_bytes)?;

        let parsed = parse_archive(
            &package_path,
            Some(&temporary.join("payload.deb")),
            self.max_package_size,
        )?;

        write_new_file(&temporary.join("manifest.json"), &parsed.manifest_raw)?;
        write_new_file(&temporary.join("signature.sm2"), &parsed.signature)?;
        sync_dir(temporary)?;
        commit_staging(temporary, final_dir)?;

        Ok(StagedPackage {
            root: final_dir.to_path_buf(),
            package_path: final_dir.join("package.bin"),
            manifest_path: final_dir.join("manifest.json"),
            deb_path: final_dir.join("payload.deb"),
            signature_path: final_dir.join("signature.sm2"),
            manifest_raw: parsed.manifest_raw,
            manifest: parsed.manifest,
            signature: parsed.signature,
        })
    }
}

struct ParsedArchive {
    manifest_raw: Vec<u8>,
    manifest: UpgradeManifest,
    signature: Vec<u8>,
}

fn parse_archive(
    package_path: &Path,
    deb_target: Option<&Path>,
    max_package_size: u64,
) -> Result<ParsedArchive, UpgradeError> {
    let mut archive_file = File::open(package_path)?;
    let mut names = BTreeSet::new();
    let mut manifest_raw = None;
    let mut signature = None;
    let mut deb_name = None;
    let mut deb_size = None;
    {
        let mut archive = tar::Archive::new(&mut archive_file);
        for entry in archive.entries().map_err(format_error)? {
            let mut entry = entry.map_err(format_error)?;
            if !entry.header().entry_type().is_file() {
                return Err(UpgradeError::Format("容器只允许普通文件".into()));
            }
            let path = entry.path().map_err(format_error)?.into_owned();
            validate_archive_path(&path)?;
            let name = path
                .to_str()
                .ok_or_else(|| UpgradeError::Format("容器文件名不是 UTF-8".into()))?
                .to_string();
            if !names.insert(name.clone()) {
                return Err(UpgradeError::Format(format!("容器条目重复: {name}")));
            }
            let declared_size = entry.size();
            match name.as_str() {
                "manifest.json" => {
                    manifest_raw = Some(read_limited(
                        &mut entry,
                        declared_size,
                        MAX_MANIFEST_SIZE,
                        "manifest",
                    )?);
                }
                "signature.sm2" => {
                    signature = Some(read_limited(
                        &mut entry,
                        declared_size,
                        MAX_SIGNATURE_SIZE,
                        "signature",
                    )?);
                }
                _ if name.ends_with(".deb") => {
                    if deb_name.is_some() || declared_size > max_package_size {
                        return Err(UpgradeError::Format("DEB 数量或大小非法".into()));
                    }
                    let copied = if let Some(target) = deb_target {
                        let mut output = new_private_file(target)?;
                        let copied = io::copy(&mut entry, &mut output)?;
                        output.sync_all()?;
                        copied
                    } else {
                        io::copy(&mut entry, &mut io::sink())?
                    };
                    if copied != declared_size {
                        return Err(UpgradeError::Format("DEB 条目长度不完整".into()));
                    }
                    deb_name = Some(name);
                    deb_size = Some(copied);
                }
                _ => return Err(UpgradeError::Format(format!("容器包含额外条目: {name}"))),
            }
        }
    }
    reject_nonzero_trailing_data(&mut archive_file)?;
    let manifest_raw =
        manifest_raw.ok_or_else(|| UpgradeError::Format("缺少 manifest.json".into()))?;
    let signature = signature.ok_or_else(|| UpgradeError::Format("缺少 signature.sm2".into()))?;
    let deb_name = deb_name.ok_or_else(|| UpgradeError::Format("缺少 DEB".into()))?;
    let deb_size = deb_size.expect("DEB name and size are assigned together");
    if names.len() != 3 {
        return Err(UpgradeError::Format("容器必须恰好包含三个文件".into()));
    }
    let manifest: UpgradeManifest = serde_json::from_slice(&manifest_raw)
        .map_err(|error| UpgradeError::Format(format!("manifest 无效: {error}")))?;
    if manifest.deb_file != deb_name || manifest.deb_size != deb_size {
        return Err(UpgradeError::Format(
            "manifest 的 DEB 文件名或大小不匹配".into(),
        ));
    }
    let expected_name = format!("usb-control_V{}_arm64.deb", manifest.package_version);
    if manifest.deb_file != expected_name {
        return Err(UpgradeError::Format("DEB 文件名不符合发布格式".into()));
    }
    Ok(ParsedArchive {
        manifest_raw,
        manifest,
        signature,
    })
}

fn format_error(error: impl std::fmt::Display) -> UpgradeError {
    UpgradeError::Format(error.to_string())
}

fn validate_staging_layout(root: &Path) -> Result<(), UpgradeError> {
    if !fs::symlink_metadata(root)?.file_type().is_dir() {
        return Err(UpgradeError::Format("staging 根目录类型非法".into()));
    }
    let expected = BTreeSet::from([
        "manifest.json".to_string(),
        "package.bin".to_string(),
        "payload.deb".to_string(),
        "signature.sm2".to_string(),
    ]);
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(UpgradeError::Format("staging 只允许普通文件".into()));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| UpgradeError::Format("staging 文件名不是 UTF-8".into()))?;
        actual.insert(name);
    }
    if actual != expected {
        return Err(UpgradeError::Format("staging 文件布局非法".into()));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, UpgradeError> {
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
    Ok(hex::encode(hasher.finalize()))
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_upgrade_id(upgrade_id: &str) -> Result<(), UpgradeError> {
    let valid = !upgrade_id.is_empty()
        && upgrade_id.len() <= 128
        && upgrade_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(UpgradeError::Format("升级任务标识非法".into()))
    }
}

fn validate_archive_path(path: &Path) -> Result<(), UpgradeError> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(UpgradeError::Format("容器条目路径非法".into()));
    }
    Ok(())
}

fn read_limited<R: Read>(
    reader: &mut R,
    declared_size: u64,
    limit: u64,
    label: &str,
) -> Result<Vec<u8>, UpgradeError> {
    if declared_size > limit {
        return Err(UpgradeError::Format(format!("{label} 超过大小上限")));
    }
    let mut bytes = Vec::with_capacity(declared_size as usize);
    reader.read_to_end(&mut bytes)?;
    if bytes.len() as u64 != declared_size {
        return Err(UpgradeError::Format(format!("{label} 长度不完整")));
    }
    Ok(bytes)
}

fn reject_nonzero_trailing_data(file: &mut File) -> Result<(), UpgradeError> {
    let position = file.stream_position()?;
    let length = file.metadata()?.len();
    if position > length {
        return Err(UpgradeError::Format("容器长度非法".into()));
    }
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        if buffer[..read].iter().any(|byte| *byte != 0) {
            return Err(UpgradeError::Format("容器末尾包含附加数据".into()));
        }
    }
    Ok(())
}

fn create_private_dir_all(path: &Path) -> io::Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::DirBuilder::new().mode(0o700).create(path)
}

fn new_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

fn write_new_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = new_private_file(path)?;
    io::copy(&mut &*bytes, &mut file)?;
    file.sync_all()
}

fn sync_dir(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

fn commit_staging(temporary: &Path, final_dir: &Path) -> Result<(), UpgradeError> {
    let parent = final_dir
        .parent()
        .ok_or_else(|| UpgradeError::State("staging 缺少父目录".into()))?;
    fs::rename(temporary, final_dir)?;
    if let Err(error) = sync_dir(parent) {
        let _ = fs::remove_dir_all(final_dir);
        let _ = sync_dir(parent);
        return Err(UpgradeError::Io(error));
    }
    Ok(())
}
