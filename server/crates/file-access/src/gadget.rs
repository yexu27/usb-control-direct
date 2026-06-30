//! RK configfs mass storage runtime.
//!
//! The RK3568 BSP already provides a composite gadget under configfs.  The
//! server must reuse that gadget and only attach/detach the mass storage LUN
//! backing file for U disk mapping.

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing::{debug, info};

const CONFIGFS_ROOT: &str = "/sys/kernel/config/usb_gadget";
const UDC_CLASS_ROOT: &str = "/sys/class/udc";

#[derive(Debug, Error)]
pub enum GadgetError {
    #[error("configfs path does not exist: {0}")]
    ConfigfsMissing(String),
    #[error("no usable mass_storage LUN found")]
    LunNotFound,
    #[error("gadget I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("backing path is not UTF-8: {0}")]
    BackingPathInvalid(String),
    #[error("no available UDC found under: {0}")]
    UdcNotFound(String),
}

#[derive(Debug, Clone)]
pub struct MassStorageLun {
    gadget_name: String,
    function_name: String,
    gadget_dir: PathBuf,
    lun_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GadgetRuntime {
    lun: MassStorageLun,
    udc_root: PathBuf,
}

impl GadgetRuntime {
    pub fn discover() -> Result<Self, GadgetError> {
        Self::discover_under(CONFIGFS_ROOT)
    }

    pub fn discover_under(root: impl AsRef<Path>) -> Result<Self, GadgetError> {
        Self::discover_under_with_udc_root(root, UDC_CLASS_ROOT)
    }

    pub fn discover_under_with_udc_root(
        root: impl AsRef<Path>,
        udc_root: impl AsRef<Path>,
    ) -> Result<Self, GadgetError> {
        let root = root.as_ref();
        if !root.exists() {
            return Err(GadgetError::ConfigfsMissing(root.display().to_string()));
        }

        let mut candidates = Vec::new();
        for gadget_entry in fs::read_dir(root)? {
            let gadget_dir = gadget_entry?.path();
            if !gadget_dir.is_dir() {
                continue;
            }

            let Some(gadget_name) = gadget_dir
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
            else {
                continue;
            };

            let functions_dir = gadget_dir.join("functions");
            if !functions_dir.is_dir() {
                continue;
            }

            for function_entry in fs::read_dir(&functions_dir)? {
                let function_dir = function_entry?.path();
                let Some(function_name) = function_dir
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                else {
                    continue;
                };

                if !function_name.starts_with("mass_storage") {
                    continue;
                }

                let lun_dir = function_dir.join("lun.0");
                if !lun_dir.is_dir() {
                    continue;
                }

                let linked_to_active_config =
                    gadget_dir.join("configs/b.1").join(&function_name).exists();
                candidates.push((
                    !linked_to_active_config,
                    gadget_name.clone(),
                    function_name,
                    gadget_dir.clone(),
                    lun_dir,
                ));
            }
        }

        candidates.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(left.1.cmp(&right.1))
                .then(left.2.cmp(&right.2))
        });

        let Some((_, gadget_name, function_name, gadget_dir, lun_dir)) =
            candidates.into_iter().next()
        else {
            return Err(GadgetError::LunNotFound);
        };

        info!(
            gadget = %gadget_name,
            function = %function_name,
            lun = %lun_dir.display(),
            "discovered RK mass storage LUN"
        );

        Ok(Self {
            lun: MassStorageLun {
                gadget_name,
                function_name,
                gadget_dir,
                lun_dir,
            },
            udc_root: udc_root.as_ref().to_path_buf(),
        })
    }

    pub fn gadget_name(&self) -> &str {
        &self.lun.gadget_name
    }

    pub fn function_name(&self) -> &str {
        &self.lun.function_name
    }

    pub fn lun_dir(&self) -> &Path {
        &self.lun.lun_dir
    }

    fn udc_path(&self) -> PathBuf {
        self.lun.gadget_dir.join("UDC")
    }

    fn unbind_udc(&self) -> Result<Option<String>, GadgetError> {
        let path = self.udc_path();
        if !path.exists() {
            return Ok(None);
        }

        let current = fs::read_to_string(&path).unwrap_or_default();
        let current = current.trim().to_string();
        if current.is_empty() {
            return Ok(None);
        }

        fs::write(&path, "\n")?;
        info!(udc = %current, "unbound UDC for mass storage LUN update");
        Ok(Some(current))
    }

    fn bind_udc(&self, previous: Option<String>) -> Result<(), GadgetError> {
        let udc = match previous {
            Some(udc) => udc,
            None => first_udc_name(&self.udc_root)?,
        };

        fs::write(self.udc_path(), format!("{udc}\n"))?;
        info!(udc = %udc, "rebound UDC after mass storage LUN update");
        Ok(())
    }

    pub fn bind_udc_if_empty(&self) -> Result<(), GadgetError> {
        self.bind_udc_if_empty_under(&self.udc_root)
    }

    pub fn bind_udc_if_empty_under(&self, udc_root: impl AsRef<Path>) -> Result<(), GadgetError> {
        let path = self.udc_path();
        if !path.exists() {
            return Ok(());
        }

        let current = fs::read_to_string(&path).unwrap_or_default();
        if !current.trim().is_empty() {
            debug!(udc = %current.trim(), "UDC already bound");
            return Ok(());
        }

        let udc = first_udc_name(udc_root.as_ref())?;
        fs::write(&path, format!("{udc}\n"))?;
        info!(udc = %udc, "bound UDC for USB gadget startup");
        Ok(())
    }

    pub fn current_backing(&self) -> Result<String, GadgetError> {
        let value = fs::read_to_string(self.lun.lun_dir.join("file"))?;
        Ok(value.trim_end_matches('\n').to_string())
    }

    pub fn prepare_empty_lun(&self) -> Result<(), GadgetError> {
        let previous = self.unbind_udc()?;
        fs::write(self.lun.lun_dir.join("file"), "\n")?;
        let _ = fs::write(self.lun.lun_dir.join("removable"), "1\n");
        let _ = fs::write(self.lun.lun_dir.join("nofua"), "1\n");
        let _ = fs::write(self.lun.lun_dir.join("cdrom"), "0\n");
        self.bind_udc(previous)?;
        Ok(())
    }

    pub fn attach_mass_storage(&self, backing: &Path, readonly: bool) -> Result<(), GadgetError> {
        let backing_str = backing
            .to_str()
            .ok_or_else(|| GadgetError::BackingPathInvalid(backing.display().to_string()))?;

        let previous = self.unbind_udc()?;
        let _ = fs::write(self.lun.lun_dir.join("file"), "\n");
        let _ = fs::write(
            self.lun.lun_dir.join("ro"),
            if readonly { "1\n" } else { "0\n" },
        );
        let _ = fs::write(self.lun.lun_dir.join("removable"), "1\n");
        let _ = fs::write(self.lun.lun_dir.join("nofua"), "1\n");
        let _ = fs::write(self.lun.lun_dir.join("cdrom"), "0\n");
        fs::write(self.lun.lun_dir.join("file"), backing_str)?;
        self.bind_udc(previous)?;

        info!(
            backing = %backing_str,
            readonly,
            "attached NBD backing to mass storage LUN"
        );
        Ok(())
    }

    pub fn detach_mass_storage(&self) -> Result<(), GadgetError> {
        let previous = self.unbind_udc()?;
        let current = self.current_backing().unwrap_or_default();
        fs::write(self.lun.lun_dir.join("file"), "\n")?;
        if current.is_empty() {
            debug!("mass storage LUN backing already empty");
        } else {
            info!(backing = %current, "cleared mass storage LUN backing");
        }
        self.bind_udc(previous)?;
        Ok(())
    }
}

fn first_udc_name(udc_root: &Path) -> Result<String, GadgetError> {
    let mut names = Vec::new();
    for entry in fs::read_dir(udc_root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.is_empty() {
            names.push(name);
        }
    }
    names.sort();
    names
        .into_iter()
        .next()
        .ok_or_else(|| GadgetError::UdcNotFound(udc_root.display().to_string()))
}
