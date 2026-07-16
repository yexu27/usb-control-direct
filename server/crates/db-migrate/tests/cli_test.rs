use std::process::Command;

#[test]
fn version_flag_prints_the_shared_release_version() {
    for flag in ["--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_usb-control-db-migrate"))
            .arg(flag)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "flag={flag}, stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{}\n", release_info::display_version())
        );
        assert!(output.stderr.is_empty());
    }
}
