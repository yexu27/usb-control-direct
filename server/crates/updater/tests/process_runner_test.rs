use std::io::{self, Write};
use std::time::Duration;

use usb_control_updater::{CommandRunner, CommandSpec, ProcessCommandRunner, UpdaterError};

#[test]
#[ignore]
fn large_output_helper() {
    let block = vec![b'x'; 256 * 1024];
    io::stdout().write_all(&block).unwrap();
    io::stderr().write_all(&block).unwrap();
}

#[test]
#[ignore]
fn sleep_helper() {
    std::thread::sleep(Duration::from_secs(10));
}

fn helper(name: &str, timeout: Duration) -> CommandSpec {
    CommandSpec {
        stage: "runner_test".into(),
        program: std::env::current_exe().unwrap(),
        args: vec![
            "--ignored".into(),
            "--exact".into(),
            name.into(),
            "--nocapture".into(),
        ],
        timeout,
    }
}

#[test]
fn large_stdout_and_stderr_are_drained_without_deadlock_and_bounded() {
    let output = ProcessCommandRunner
        .run(&helper("large_output_helper", Duration::from_secs(5)))
        .unwrap();
    assert!(output.success);
    assert!(output.stdout.len() <= 64 * 1024);
    assert!(output.stderr.len() <= 64 * 1024);
    assert!(output.stdout_truncated);
    assert!(output.stderr_truncated);
}

#[test]
fn timeout_kills_reaps_and_joins_output_readers() {
    let error = ProcessCommandRunner
        .run(&helper("sleep_helper", Duration::from_millis(50)))
        .unwrap_err();
    assert!(matches!(error, UpdaterError::CommandTimeout { .. }));
}
