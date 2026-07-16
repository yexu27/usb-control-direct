//! USB Control 短周期系统升级执行器。

mod database;
mod error;
mod executor;
mod health;
mod install;
mod migration;

pub use database::{SqliteUpgradeDatabase, UpgradeDatabase};
pub use error::UpdaterError;
pub use executor::{
    Clock, CommandOutput, CommandRunner, CommandSpec, PackageRevalidator, ProcessCommandRunner,
    RevalidatedPackage, SharedPackageRevalidator, SystemClock, UpgradeExecutionReport,
    UpgradeExecutor, UpgradePaths,
};
pub use health::{
    certificate_sha256, validate_health_snapshot, HealthExpectation, ServiceSnapshot,
};
pub use install::{parse_command, InstallFinalizer, ManagedInstallGuard, UpdaterCommand};
