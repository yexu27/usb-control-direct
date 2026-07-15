use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use usb_control_app::upgrade_preflight::{
    available_bytes, platform_matches, record_locks_available, BoundedCommandRunner,
};

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

#[test]
#[ignore]
fn lock_holder_helper() {
    let lock_path = std::env::var_os("USB_CONTROL_TEST_LOCK").unwrap();
    let ready_path = std::env::var_os("USB_CONTROL_TEST_READY").unwrap();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .unwrap();
    let mut lock = libc::flock {
        l_type: libc::F_WRLCK as i16,
        l_whence: libc::SEEK_SET as i16,
        l_start: 0,
        l_len: 0,
        l_pid: 0,
    };
    let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLK, &mut lock) };
    assert_eq!(result, 0);
    fs::write(ready_path, b"ready").unwrap();
    std::thread::sleep(Duration::from_secs(10));
}

fn helper_args(name: &str) -> Vec<String> {
    vec![
        "--ignored".into(),
        "--exact".into(),
        name.into(),
        "--nocapture".into(),
    ]
}

#[test]
fn stdout_and_stderr_are_drained_without_deadlock_and_bounded() {
    let executable = std::env::current_exe().unwrap();
    let args = helper_args("large_output_helper");
    let refs = args.iter().map(OsStr::new).collect::<Vec<_>>();
    let output = BoundedCommandRunner::new(Duration::from_secs(5), 64 * 1024)
        .run(&executable, &refs)
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 64 * 1024);
    assert_eq!(output.stderr.len(), 64 * 1024);
}

#[test]
fn timeout_kills_and_reaps_child() {
    let executable = std::env::current_exe().unwrap();
    let args = helper_args("sleep_helper");
    let refs = args.iter().map(OsStr::new).collect::<Vec<_>>();
    let started = Instant::now();
    let error = BoundedCommandRunner::new(Duration::from_millis(50), 64 * 1024)
        .run(&executable, &refs)
        .unwrap_err();
    assert!(error.contains("超时"));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn second_process_holding_record_lock_is_reported_busy() {
    let temp = tempfile::tempdir().unwrap();
    let lock = temp.path().join("dpkg.lock");
    let ready = temp.path().join("ready");
    fs::write(&lock, b"").unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(helper_args("lock_holder_helper"))
        .env("USB_CONTROL_TEST_LOCK", &lock)
        .env("USB_CONTROL_TEST_READY", &ready)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !ready.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(ready.exists(), "lock holder did not become ready");
    assert!(!record_locks_available(std::slice::from_ref(&lock)).unwrap());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(record_locks_available(&[lock]).unwrap());
}

#[test]
fn statvfs_reports_available_bytes_for_directory() {
    let temp = tempfile::tempdir().unwrap();
    assert!(available_bytes(temp.path()).unwrap() > 0);
}

#[test]
fn platform_parser_accepts_only_target_platform() {
    assert!(platform_matches(
        "ID=ubuntu\nVERSION_ID=\"22.04\"\n",
        "aarch64\n",
        "4.19.232\n"
    ));
    assert!(!platform_matches(
        "ID=ubuntu\nVERSION_ID=\"24.04\"\n",
        "aarch64\n",
        "4.19.232\n"
    ));
    assert!(!platform_matches(
        "ID=ubuntu\nVERSION_ID=\"22.04\"\n",
        "x86_64\n",
        "4.19.232\n"
    ));
    assert!(!platform_matches(
        "ID=ubuntu\nVERSION_ID=\"22.04\"\n",
        "aarch64\n",
        "5.15.0\n"
    ));
}
