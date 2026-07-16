fn main() {
    println!("cargo:rerun-if-env-changed=USB_CONTROL_RELEASE_VERSION");

    if let Ok(version) = std::env::var("USB_CONTROL_RELEASE_VERSION") {
        if !is_strict_version(&version) {
            panic!(
                "USB_CONTROL_RELEASE_VERSION must be major.minor.patch without prefixes or leading zeros"
            );
        }
    }
}

fn is_strict_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let valid_part = |part: &str| {
        !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && (part == "0" || !part.starts_with('0'))
    };

    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(major), Some(minor), Some(patch), None)
            if valid_part(major) && valid_part(minor) && valid_part(patch)
    )
}
