//! 授权与病毒库维护服务。
//!
//! 包含机器码生成、授权校验和病毒库升级。

pub mod error;
pub mod license;
pub mod machine_code;
pub mod production_license;
pub mod virusdb_upgrade;

pub use error::LicenseUpgradeError;
pub use license::{LicenseInfo, LicenseValidator};
pub use machine_code::{generate_machine_code, MachineCodeResult};
pub use production_license::ProductionLicenseValidator;
pub use virusdb_upgrade::VirusdbUpgradeManager;
