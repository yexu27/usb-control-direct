use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use smcrypto::{sm2, sm3};
use usb_control_release_tool::generate_key;

#[test]
fn generated_key_material_is_valid_private_and_non_secret_in_output() {
    let dir = tempfile::tempdir().unwrap();

    generate_key("upgrade-test-01", dir.path()).unwrap();

    let private = read_trimmed(&dir.path().join("upgrade_sign.key"));
    let public = read_trimmed(&dir.path().join("upgrade_verify.pub"));
    let key_id = read_trimmed(&dir.path().join("upgrade_verify.id"));
    assert_eq!(private.len(), 64);
    assert!(private.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(public.len(), 128);
    assert!(public.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(key_id, "upgrade-test-01");
    assert_eq!(
        fs::metadata(dir.path().join("upgrade_sign.key"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    for public_file in ["upgrade_verify.pub", "upgrade_verify.id"] {
        assert_eq!(
            fs::metadata(dir.path().join(public_file))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o644
        );
    }

    let digest = hex::decode(sm3::sm3_hash(b"release-tool-key-self-test")).unwrap();
    let signature = sm2::Sign::new(&private).sign(&digest);
    assert!(sm2::Verify::new(&public).verify(&digest, &signature));

    let cli_dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_usb-control-release-tool"))
        .args(["generate-key", "--key-id", "upgrade-test-02", "--key-dir"])
        .arg(cli_dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let cli_private = read_trimmed(&cli_dir.path().join("upgrade_sign.key"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&cli_private));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(&cli_private));
}

#[test]
fn generation_rejects_existing_target_without_overwriting_any_file() {
    for existing in [
        "upgrade_sign.key",
        "upgrade_verify.pub",
        "upgrade_verify.id",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(existing);
        fs::write(&path, b"keep-existing").unwrap();

        assert!(generate_key("upgrade-test-01", dir.path()).is_err());

        assert_eq!(fs::read(&path).unwrap(), b"keep-existing");
        for other in [
            "upgrade_sign.key",
            "upgrade_verify.pub",
            "upgrade_verify.id",
        ] {
            if other != existing {
                assert!(!dir.path().join(other).exists());
            }
        }
    }
}

#[test]
fn generation_rejects_key_id_outside_runtime_contract() {
    for invalid in [
        "",
        "UPPER",
        "under_score",
        ".leading",
        "-leading",
        &"a".repeat(65),
    ] {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            generate_key(invalid, dir.path()).is_err(),
            "accepted {invalid}"
        );
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}

fn read_trimmed(path: &std::path::Path) -> String {
    fs::read_to_string(path).unwrap().trim().to_string()
}
