//! 系统升级领域服务。

mod deb;
mod error;
mod package;
mod readiness;
mod result;
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
pub use result::{
    read_active_release, read_last_known_good, read_upgrade_result, ActiveCommitError,
    ActiveRelease, DirectorySync, LastKnownGoodRelease, ReleaseStateStore, UpgradeResult,
};
pub use staging::PackageStager;
pub use state::{UpgradeStatus, UpgradeTask, UpgradeTaskStore};
pub use verifier::{PackageVerifier, VerificationContext, VerifiedPackage};
pub use version::SystemVersion;
