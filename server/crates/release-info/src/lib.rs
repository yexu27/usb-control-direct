//! 构建时发布版本信息。

pub const VERSION: &str = match option_env!("USB_CONTROL_RELEASE_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

pub fn display_version() -> String {
    format!("V{VERSION}")
}
