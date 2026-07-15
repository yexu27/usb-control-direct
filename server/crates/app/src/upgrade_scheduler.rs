//! systemd 短周期 updater 调度适配器。

use std::process::Command;

use system_upgrade::{UpgradeError, UpgradeScheduler};

pub const SYSTEMCTL_PROGRAM: &str = "systemctl";
pub const SYSTEMCTL_START_ARGS: [&str; 3] = ["start", "--no-block", "usb-control-updater.service"];

pub struct SystemdUpgradeScheduler;

impl UpgradeScheduler for SystemdUpgradeScheduler {
    fn start(&self, _upgrade_id: &str) -> Result<(), UpgradeError> {
        let status = Command::new(SYSTEMCTL_PROGRAM)
            .args(SYSTEMCTL_START_ARGS)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(UpgradeError::State(format!(
                "systemd 调度 updater 失败: {status}"
            )))
        }
    }
}
