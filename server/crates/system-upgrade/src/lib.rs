//! 系统升级领域服务。

mod deb;
mod error;
mod package;
mod readiness;
mod release;
mod result;
mod signing;
mod staging;
mod state;
mod verifier;
mod version;

mod coordinator;

pub use coordinator::{
    PrepareUpgradeRequest, PreparedUpgrade, UpgradeCoordinator, UpgradeEnvironment,
    UpgradePreflight, UpgradePreflightRequest, UpgradeScheduler,
};
pub use deb::{certificate_sha256, DebInspector, DebMetadata, DpkgDebInspector};
pub use error::{UpgradeError, UpgradePreflightFailure};
pub use package::{StagedPackage, UpgradeManifest};
pub use readiness::ServiceReady;
pub use release::{
    read_installed_release, ActiveCommitError, ActiveRelease, ActiveReleaseStore, InstalledRelease,
};
pub use result::{UpgradeResult, UpgradeResultStore};
pub use signing::upgrade_signing_digest;
pub use staging::PackageStager;
pub use state::{UpgradeStateLock, UpgradeStatus, UpgradeTask, UpgradeTaskStore};
pub use verifier::{PackageVerifier, VerificationContext, VerifiedPackage};
pub use version::SystemVersion;
