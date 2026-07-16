mod support;

use support::{write_deb_fixture, DebFixtureOptions};
use system_upgrade::{DebInspector, DpkgDebInspector, SystemVersion};

#[test]
fn inspects_required_files_and_strict_release_metadata() {
    let fixture = write_deb_fixture(DebFixtureOptions::default());
    let metadata = DpkgDebInspector::default()
        .inspect(&fixture.path)
        .expect("valid bounded DEB must be inspected");

    assert_eq!(metadata.package, "usb-control");
    assert_eq!(
        metadata.version,
        SystemVersion::parse("3.1.0").expect("valid expected version")
    );
    assert_eq!(metadata.architecture, "arm64");
    assert!(metadata.expanded_size > 0);
    assert!(metadata.expanded_size <= 512 * 1024 * 1024);
    assert_eq!(metadata.supported_schema_min, 1);
    assert_eq!(metadata.supported_schema_max, 2);
    assert_eq!(metadata.migration_schema_to, 2);
    assert_eq!(metadata.tls_cert_sha256, fixture.tls_sha256);
    assert_eq!(metadata.upgrade_signing_key_id, "upgrade-prod-01");
}

#[test]
fn rejects_control_fields_that_disagree_with_release_metadata() {
    for options in [
        DebFixtureOptions {
            control_package: "another-package",
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            control_version: "3.2.0",
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            control_architecture: "amd64",
            ..DebFixtureOptions::default()
        },
    ] {
        let fixture = write_deb_fixture(options);
        assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
    }
}

#[test]
fn rejects_missing_required_file_or_migration() {
    for omitted in [
        "opt/usb-control/bin/usb-control",
        "opt/usb-control/bin/usb-control-updater",
        "opt/usb-control/bin/usb-control-db-migrate",
        "opt/usb-control/install-meta/release.json",
        "lib/systemd/system/usb-control.service",
        "lib/systemd/system/usb-control-updater.service",
        "opt/usb-control/db/migrations/0001_init.sql",
        "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.id",
        "opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.pub",
        "opt/usb-control/defaults/etc/usb-control/tls/server.crt",
    ] {
        let fixture = write_deb_fixture(DebFixtureOptions {
            omit_path: Some(omitted),
            ..DebFixtureOptions::default()
        });
        assert!(
            DpkgDebInspector::default().inspect(&fixture.path).is_err(),
            "missing {omitted} must be rejected"
        );
    }
}

#[test]
fn rejects_certificate_fingerprint_mismatch_and_unknown_metadata_field() {
    let bad_fingerprint = write_deb_fixture(DebFixtureOptions {
        release_tls_sha256: Some("00".repeat(32)),
        ..DebFixtureOptions::default()
    });
    assert!(DpkgDebInspector::default()
        .inspect(&bad_fingerprint.path)
        .is_err());

    let unknown_field = write_deb_fixture(DebFixtureOptions {
        release_extra_field: true,
        ..DebFixtureOptions::default()
    });
    assert!(DpkgDebInspector::default()
        .inspect(&unknown_field.path)
        .is_err());
}

#[test]
fn rejects_every_forbidden_release_content_class() {
    let forbidden = [
        "opt/usb-control/install-meta/VERSION",
        "opt/usb-control/install-meta/component-lock.txt",
        "opt/usb-control/defaults/etc/usb-control/keys/upgrade_sign.key",
        "opt/usb-control/tests/case.json",
        "opt/usb-control/bin/smoke.sh",
        "opt/usb-control/testdata/sample.bin",
        "opt/usb-control/src/main.rs",
        "lib/modules/usb_control.ko",
        "var/lib/clamav/daily.cvd",
        "var/lib/usb-control/usb-control.db",
        "var/log/usb-control/server.log",
        "opt/usb-control/bundle/nested.deb",
        "opt/usb-control/bin/integration-test",
        "opt/usb-control/bin/usb-control-otg-init.sh",
        "opt/usb-control/defaults/etc/usb-control/keys/sign.key",
    ];
    for path in forbidden {
        let fixture = write_deb_fixture(DebFixtureOptions {
            extra_paths: vec![path],
            ..DebFixtureOptions::default()
        });
        assert!(
            DpkgDebInspector::default().inspect(&fixture.path).is_err(),
            "forbidden release content must be rejected: {path}"
        );
    }
}

#[test]
fn rejects_symlink_and_hardlink_even_when_the_path_is_allowlisted() {
    for entry_type in [b'2', b'1'] {
        let fixture = write_deb_fixture(DebFixtureOptions {
            special_entry: Some(("opt/usb-control/bin/usb-control", entry_type)),
            ..DebFixtureOptions::default()
        });
        assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
    }
}

#[test]
fn rejects_directory_outside_allowed_parent_set() {
    let fixture = write_deb_fixture(DebFixtureOptions {
        special_entry: Some(("opt/usb-control/rogue", b'5')),
        ..DebFixtureOptions::default()
    });
    assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
}

#[test]
fn validates_target_upgrade_trust_root_and_allows_key_rotation() {
    let rotated = write_deb_fixture(DebFixtureOptions {
        release_key_id: "upgrade-prod-02",
        upgrade_key_id: "upgrade-prod-02",
        ..DebFixtureOptions::default()
    });
    DpkgDebInspector::default()
        .inspect(&rotated.path)
        .expect("target DEB may carry the next signing key id");

    for options in [
        DebFixtureOptions {
            upgrade_key_id: "INVALID_ID",
            release_key_id: "INVALID_ID",
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            upgrade_key_id: "upgrade-prod-02",
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            upgrade_public_key: Some("abcd"),
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            upgrade_public_key: Some("gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg"),
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            upgrade_public_key: Some("11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111"),
            ..DebFixtureOptions::default()
        },
    ] {
        let fixture = write_deb_fixture(options);
        assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
    }
}

#[test]
fn rejects_migration_format_duplicates_and_gaps() {
    for paths in [
        vec![
            "opt/usb-control/db/migrations/0001_init.sql",
            "opt/usb-control/db/migrations/0003_gap.sql",
        ],
        vec![
            "opt/usb-control/db/migrations/0001_init.sql",
            "opt/usb-control/db/migrations/0001_duplicate.sql",
            "opt/usb-control/db/migrations/0002_upgrade.sql",
        ],
        vec![
            "opt/usb-control/db/migrations/1_bad.sql",
            "opt/usb-control/db/migrations/0002_upgrade.sql",
        ],
    ] {
        let fixture = write_deb_fixture(DebFixtureOptions {
            migration_paths: paths,
            ..DebFixtureOptions::default()
        });
        assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
    }
}

#[test]
fn rejects_seed_format_duplicates_and_gaps() {
    for paths in [
        vec![
            "opt/usb-control/db/seeds/0001_default.sql",
            "opt/usb-control/db/seeds/0003_gap.sql",
        ],
        vec![
            "opt/usb-control/db/seeds/0001_default.sql",
            "opt/usb-control/db/seeds/0001_duplicate.sql",
        ],
        vec!["opt/usb-control/db/seeds/1_bad.sql"],
    ] {
        let fixture = write_deb_fixture(DebFixtureOptions {
            seed_paths: paths,
            ..DebFixtureOptions::default()
        });
        assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
    }
}

#[test]
fn rejects_gnu_longname_and_pax_extensions_before_materialization() {
    for entry_type in [b'L', b'K', b'x', b'g'] {
        let fixture = write_deb_fixture(DebFixtureOptions {
            tar_extension: Some(entry_type),
            ..DebFixtureOptions::default()
        });
        assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
    }
}

#[test]
fn rejects_entry_count_expanded_size_and_selected_file_limits() {
    for options in [
        DebFixtureOptions {
            exceed_entry_count: true,
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            exceed_expanded_size: true,
            ..DebFixtureOptions::default()
        },
        DebFixtureOptions {
            exceed_selected_file_size: true,
            ..DebFixtureOptions::default()
        },
    ] {
        let fixture = write_deb_fixture(options);
        assert!(DpkgDebInspector::default().inspect(&fixture.path).is_err());
    }
}
