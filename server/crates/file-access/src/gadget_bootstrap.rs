//! USB gadget startup bootstrap for the controlled host side.

use std::fs;
use std::path::{Path, PathBuf};

use hid_access::hid_gadget::{ensure_hid_functions_under, HidFunctionNames};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::gadget::{GadgetRuntime, MassStorageHandle};

#[derive(Debug, Clone)]
pub struct GadgetBootstrapConfig {
    pub configfs_root: PathBuf,
    pub udc_root: PathBuf,
    pub gadget_name: String,
    pub config_name: String,
    pub udc: Option<String>,
    pub keep_adb: bool,
    pub storage_function: String,
    pub storage_lun: u8,
    pub keyboard_function: String,
    pub mouse_function: String,
}

#[derive(Debug, Error)]
pub enum GadgetBootstrapError {
    #[error("gadget I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("HID function 配置失败: {0}")]
    Hid(String),
    #[error("no available UDC found under: {0}")]
    UdcNotFound(String),
    #[error("invalid function name: {0}")]
    InvalidFunctionName(String),
    #[error("键盘和鼠标 HID function 不能相同: {0}")]
    DuplicateHidFunction(String),
}

pub struct GadgetBootstrap;

impl GadgetBootstrap {
    pub fn prepare(config: GadgetBootstrapConfig) -> Result<GadgetRuntime, GadgetBootstrapError> {
        validate_function_name(&config.storage_function)?;
        validate_function_name(&config.keyboard_function)?;
        validate_function_name(&config.mouse_function)?;
        if config.keyboard_function == config.mouse_function {
            return Err(GadgetBootstrapError::DuplicateHidFunction(
                config.keyboard_function,
            ));
        }

        let gadget_dir = config.configfs_root.join(&config.gadget_name);
        let functions_dir = gadget_dir.join("functions");
        let config_dir = gadget_dir.join("configs").join(&config.config_name);
        fs::create_dir_all(&functions_dir)?;
        fs::create_dir_all(config_dir.join("strings/0x409"))?;
        fs::create_dir_all(gadget_dir.join("strings/0x409"))?;

        let storage_dir = functions_dir.join(&config.storage_function);
        let lun_dir = storage_dir.join(format!("lun.{}", config.storage_lun));
        fs::create_dir_all(&lun_dir)?;

        let storage_link = config_dir.join(&config.storage_function);
        let keyboard_link = config_dir.join(&config.keyboard_function);
        let mouse_link = config_dir.join(&config.mouse_function);
        let should_change_links = !storage_link.exists()
            || !keyboard_link.exists()
            || !mouse_link.exists()
            || (!config.keep_adb && has_adb_links(&config_dir)?);

        let udc_file = gadget_dir.join("UDC");
        if !udc_file.exists() {
            write_attr(&udc_file, "\n")?;
        }
        let previous_udc = fs::read_to_string(&udc_file).unwrap_or_default();
        let was_bound = !previous_udc.trim().is_empty();
        if should_change_links && was_bound {
            write_attr(&udc_file, "")?;
            info!(
                udc = %previous_udc.trim(),
                "USB gadget bootstrap: temporarily unbound UDC for function changes"
            );
        }

        let result = (|| {
            configure_lun(&lun_dir)?;

            let hid_names = HidFunctionNames {
                keyboard: config.keyboard_function.clone(),
                mouse: config.mouse_function.clone(),
            };
            ensure_hid_functions_under(&gadget_dir, &hid_names)
                .map_err(|e| GadgetBootstrapError::Hid(e.to_string()))?;

            if !config.keep_adb {
                remove_adb_links(&config_dir)?;
            }
            ensure_function_link(&storage_dir, &storage_link)?;
            ensure_function_link(
                &functions_dir.join(&config.keyboard_function),
                &keyboard_link,
            )?;
            ensure_function_link(&functions_dir.join(&config.mouse_function), &mouse_link)?;

            Ok::<(), GadgetBootstrapError>(())
        })();

        if let Err(err) = result {
            if was_bound {
                let _ = write_attr(&udc_file, previous_udc);
            }
            return Err(err);
        }

        let udc_name = match config.udc {
            Some(value) => value,
            None => first_udc_name(&config.udc_root)?,
        };
        if fs::read_to_string(&udc_file)
            .unwrap_or_default()
            .trim()
            .is_empty()
        {
            write_attr(&udc_file, format!("{udc_name}\n"))?;
            info!(udc = %udc_name, "USB gadget bootstrap: bound UDC");
        } else {
            debug!("USB gadget bootstrap: UDC already bound");
        }

        Ok(GadgetRuntime::from_handle(
            MassStorageHandle::new(
                config.gadget_name,
                config.storage_function,
                gadget_dir,
                lun_dir,
            ),
            config.udc_root,
        ))
    }
}

fn configure_lun(lun_dir: &Path) -> Result<(), GadgetBootstrapError> {
    let stale = fs::read_to_string(lun_dir.join("file")).unwrap_or_default();
    if !stale.trim().is_empty() {
        warn!(
            backing = %stale.trim(),
            "启动恢复: 清理旧 mass storage LUN backing"
        );
    }
    write_attr(lun_dir.join("file"), "\n")?;
    write_attr(lun_dir.join("ro"), "0\n")?;
    write_attr(lun_dir.join("removable"), "1\n")?;
    write_attr(lun_dir.join("nofua"), "1\n")?;
    write_attr(lun_dir.join("cdrom"), "0\n")?;
    Ok(())
}

fn validate_function_name(name: &str) -> Result<(), GadgetBootstrapError> {
    if name.is_empty() || name.contains('/') || name.contains("..") {
        return Err(GadgetBootstrapError::InvalidFunctionName(name.to_string()));
    }
    Ok(())
}

fn write_attr(path: impl AsRef<Path>, value: impl AsRef<[u8]>) -> Result<(), std::io::Error> {
    fs::write(path, value)
}

fn ensure_function_link(target: &Path, link: &Path) -> Result<(), std::io::Error> {
    if link.exists() {
        return Ok(());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link)?;
    #[cfg(not(unix))]
    fs::create_dir_all(link)?;
    Ok(())
}

fn has_adb_links(config_dir: &Path) -> Result<bool, std::io::Error> {
    if !config_dir.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(config_dir)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.contains("adb") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn remove_adb_links(config_dir: &Path) -> Result<(), std::io::Error> {
    if !config_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(config_dir)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.contains("adb") {
            let _ = fs::remove_file(&path);
            let _ = fs::remove_dir_all(&path);
        }
    }
    Ok(())
}

fn first_udc_name(udc_root: &Path) -> Result<String, GadgetBootstrapError> {
    let mut names = Vec::new();
    for entry in fs::read_dir(udc_root)? {
        let name = entry?.file_name().to_string_lossy().to_string();
        if !name.is_empty() {
            names.push(name);
        }
    }
    names.sort();
    names
        .into_iter()
        .next()
        .ok_or_else(|| GadgetBootstrapError::UdcNotFound(udc_root.display().to_string()))
}
