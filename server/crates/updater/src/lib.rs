//! USB Control 短周期系统升级执行器。

mod error;
mod executor;
mod health;
mod migration;
mod rollback;

pub use error::UpdaterError;
pub use executor::{
    ActiveReleasePublisher, Clock, CommandOutput, CommandRunner, CommandSpec, ExecutionDisposition,
    PackageRevalidator, ProcessCommandRunner, RevalidatedPackage, SharedPackageRevalidator,
    SystemClock, UpgradeExecutor, UpgradePaths,
};
pub use health::{
    certificate_sha256, validate_health_snapshot, HealthExpectation, ServiceSnapshot,
};
pub use rollback::{FileLkgRepository, LastKnownGoodRelease, LkgRepository};
