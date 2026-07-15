use std::fs;
use std::path::PathBuf;

use system_upgrade::UpgradeTask;
use usb_control_updater::{
    ProcessCommandRunner, SharedPackageRevalidator, SystemClock, UpdaterError, UpgradeExecutor,
    UpgradePaths,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("usb-control-updater: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), UpdaterError> {
    let root = parse_args(std::env::args())?;
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

fn parse_args<I, S>(args: I) -> Result<PathBuf, UpdaterError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let _program = args.next();
    if args.next().as_deref() != Some("run") || args.next().as_deref() != Some("--root") {
        return Err(UpdaterError::TaskInvalid(
            "usage: usb-control-updater run --root <upgrade-root>".into(),
        ));
    }
    let root = args
        .next()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| UpdaterError::TaskInvalid("--root 缺少路径".into()))?;
    if args.next().is_some() {
        return Err(UpdaterError::TaskInvalid("存在未知命令参数".into()));
    }
    Ok(root)
}
