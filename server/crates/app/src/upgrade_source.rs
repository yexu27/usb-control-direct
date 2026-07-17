//! 数据库存储到系统升级领域源状态端口的适配。

use std::sync::Arc;

use storage::Storage;
use system_upgrade::{SystemVersion, UpgradeError, UpgradeSourceReader, UpgradeSourceState};

pub struct StorageUpgradeSourceReader {
    storage: Arc<Storage>,
}

impl StorageUpgradeSourceReader {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
}

impl UpgradeSourceReader for StorageUpgradeSourceReader {
    fn read(&self) -> Result<UpgradeSourceState, UpgradeError> {
        let version = self
            .storage
            .system_version()
            .map_err(|error| UpgradeError::State(format!("读取数据库系统版本失败: {error}")))?;
        Ok(UpgradeSourceState {
            current_version: SystemVersion::parse(&version)?,
            current_schema: self
                .storage
                .schema_version()
                .map_err(|error| UpgradeError::State(format!("读取数据库 Schema 失败: {error}")))?,
        })
    }
}
