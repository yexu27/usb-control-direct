use std::fs;
use system_upgrade::UpgradeTask;
use usb_control_updater::{
    parse_command, InstallFinalizer, ProcessCommandRunner, SharedPackageRevalidator, SystemClock,
    UpdaterCommand, UpdaterError, UpgradeExecutor, UpgradePaths,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("usb-control-updater: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), UpdaterError> {
    match parse_command(std::env::args())? {
        UpdaterCommand::Version => {
            println!("{}", release_info::display_version());
            Ok(())
        }
        UpdaterCommand::FinalizeInstall => InstallFinalizer::new(
            UpgradePaths::production("/var/lib/usb-control/upgrade".into()),
            ProcessCommandRunner,
            SystemClock,
        )
        .finalize(),
        UpdaterCommand::Run { root } => run_online(root),
    }
}

fn run_online(root: std::path::PathBuf) -> Result<(), UpdaterError> {
    // 这里只取得调度标识；executor 获取全局锁后会重新严格读取并验证完整任务。
    let task: UpgradeTask = serde_json::from_slice(&fs::read(root.join("current.json"))?)?;
    let report = UpgradeExecutor::new(
        UpgradePaths::production(root),
        ProcessCommandRunner,
        SharedPackageRevalidator::production(),
        SystemClock,
    )
    .execute(&task.upgrade_id)?;
    if let Some(warning) = report.post_commit_warning {
        eprintln!("usb-control-updater: 升级已提交，但元数据收敛待重试: {warning}");
    }
    Ok(())
}
