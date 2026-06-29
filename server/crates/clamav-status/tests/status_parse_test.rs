use clamav_status::parse_clamav_version_output;

#[test]
fn parses_real_clamav_version_output() {
    let status =
        parse_clamav_version_output("ClamAV 1.4.4/28045/Sun Jun 28 14:26:16 2026\n")
            .unwrap();

    assert_eq!(status.engine_version, "1.4.4");
    assert_eq!(status.virus_db_version, "28045");
    assert_eq!(status.virus_db_updated_at, 1_782_656_776);
    assert_eq!(
        status.raw_output,
        "ClamAV 1.4.4/28045/Sun Jun 28 14:26:16 2026"
    );
}

#[test]
fn rejects_missing_signature_version() {
    let err = parse_clamav_version_output("ClamAV 1.4.4").unwrap_err();
    assert!(err.to_string().contains("invalid ClamAV version output"));
}

#[test]
fn rejects_unparseable_update_time() {
    let err = parse_clamav_version_output("ClamAV 1.4.4/28045/not-a-date").unwrap_err();
    assert!(err.to_string().contains("parse ClamAV database time failed"));
}
