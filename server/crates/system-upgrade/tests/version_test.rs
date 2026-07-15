use system_upgrade::SystemVersion;

#[test]
fn parses_strict_three_part_version() {
    assert_eq!(SystemVersion::parse("3.1.0").unwrap().to_string(), "3.1.0");
}

#[test]
fn rejects_prefix_leading_zero_and_missing_part() {
    let invalid_versions = [
        "V3.1.0",
        "3.01.0",
        "3.1",
        "3.1.0.1",
        "３.1.0",
        "3.1.18446744073709551616",
    ];

    for value in invalid_versions {
        assert!(
            SystemVersion::parse(value).is_err(),
            "version must be rejected: {value}"
        );
    }
}

#[test]
fn compares_versions_numerically() {
    assert!(SystemVersion::parse("3.10.0").unwrap() > SystemVersion::parse("3.9.9").unwrap());
}

#[test]
fn serializes_and_deserializes_as_strict_version_string() {
    let version = SystemVersion::parse("3.1.0").unwrap();

    assert_eq!(serde_json::to_string(&version).unwrap(), "\"3.1.0\"");
    assert_eq!(
        serde_json::from_str::<SystemVersion>("\"3.1.0\"").unwrap(),
        version
    );
    assert!(
        serde_json::from_str::<SystemVersion>("{\"major\":3,\"minor\":1,\"patch\":0}").is_err()
    );
    assert!(serde_json::from_str::<SystemVersion>("\"3.01.0\"").is_err());
}
