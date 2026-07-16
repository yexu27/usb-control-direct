#[test]
fn release_version_is_strict_three_part_numeric() {
    assert_eq!(release_info::VERSION.split('.').count(), 3);
    assert!(release_info::VERSION.split('.').all(|part| {
        !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && (part == "0" || !part.starts_with('0'))
    }));
}

#[test]
fn display_version_adds_only_the_ui_prefix() {
    assert_eq!(
        release_info::display_version(),
        format!("V{}", release_info::VERSION)
    );
}
