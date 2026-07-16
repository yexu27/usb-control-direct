use std::process::Command;

use usb_control_updater::{parse_command, UpdaterCommand};

#[test]
fn version_flag_prints_the_shared_release_version() {
    for flag in ["--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_usb-control-updater"))
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

#[test]
fn cli_accepts_only_run_or_finalize_install() {
    assert_eq!(
        parse_command(["usb-control-updater", "run", "--root", "/upgrade"]).unwrap(),
        UpdaterCommand::Run {
            root: "/upgrade".into()
        }
    );
    assert_eq!(
        parse_command(["usb-control-updater", "finalize-install"]).unwrap(),
        UpdaterCommand::FinalizeInstall
    );
    assert_eq!(
        parse_command(["usb-control-updater", "--version"]).unwrap(),
        UpdaterCommand::Version
    );
    assert_eq!(
        parse_command(["usb-control-updater", "-V"]).unwrap(),
        UpdaterCommand::Version
    );

    for args in [
        vec!["usb-control-updater"],
        vec!["usb-control-updater", "run"],
        vec!["usb-control-updater", "run", "--root", ""],
        vec!["usb-control-updater", "finalize-install", "extra"],
        vec!["usb-control-updater", "unknown"],
    ] {
        assert!(parse_command(args).is_err());
    }
}
