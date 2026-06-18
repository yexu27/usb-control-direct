//! S09 授权与系统维护服务。
//!
//! 包含机器码生成、授权校验、系统升级和病毒库升级。

pub mod error;
pub mod license;
pub mod machine_code;
pub mod system_upgrade;
pub mod virusdb_upgrade;

pub use error::LicenseUpgradeError;
pub use license::{LicenseInfo, LicenseValidator};
pub use machine_code::{generate_machine_code, MachineCodeResult};
pub use system_upgrade::SystemUpgradeManager;
pub use virusdb_upgrade::VirusdbUpgradeManager;
