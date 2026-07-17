//! updater 使用的数据库升级状态端口与 SQLite 适配器。

use std::path::PathBuf;

use usb_control_db_migrate::UpgradeDatabaseState;

use crate::UpdaterError;

/// updater 读取升级源状态和提交安装状态所依赖的数据库能力。
pub trait UpgradeDatabase: Send + Sync {
    fn read_state(&self) -> Result<UpgradeDatabaseState, UpdaterError>;

    fn compare_and_commit_online_install_state(
        &self,
        expected_source: &str,
        target: &str,
        virus_db_version: &str,
        virus_db_updated_at: i64,
        committed_at: i64,
    ) -> Result<(), UpdaterError>;

    fn commit_direct_install_state(
        &self,
        target: &str,
        virus_db_version: &str,
        virus_db_updated_at: i64,
        committed_at: i64,
    ) -> Result<(), UpdaterError>;
}

/// 将 updater 数据库端口委托给 db-migrate 基础设施库。
pub struct SqliteUpgradeDatabase {
    database_path: PathBuf,
}

impl SqliteUpgradeDatabase {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

impl UpgradeDatabase for SqliteUpgradeDatabase {
    fn read_state(&self) -> Result<UpgradeDatabaseState, UpdaterError> {
        usb_control_db_migrate::read_upgrade_database_state(&self.database_path)
            .map_err(UpdaterError::TaskInvalid)
    }

    fn compare_and_commit_online_install_state(
        &self,
        expected_source: &str,
        target: &str,
        virus_db_version: &str,
        virus_db_updated_at: i64,
        committed_at: i64,
    ) -> Result<(), UpdaterError> {
        usb_control_db_migrate::compare_and_commit_online_install_state(
            &self.database_path,
            expected_source,
            target,
            virus_db_version,
            virus_db_updated_at,
            committed_at,
        )
        .map_err(UpdaterError::TaskInvalid)
    }

    fn commit_direct_install_state(
        &self,
        target: &str,
        virus_db_version: &str,
        virus_db_updated_at: i64,
        committed_at: i64,
    ) -> Result<(), UpdaterError> {
        usb_control_db_migrate::commit_direct_install_state(
            &self.database_path,
            target,
            virus_db_version,
            virus_db_updated_at,
            committed_at,
        )
        .map_err(UpdaterError::TaskInvalid)
    }
}
