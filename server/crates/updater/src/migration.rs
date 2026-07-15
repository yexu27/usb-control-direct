//! 目标/回滚版本数据库迁移命令契约。

use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use crate::executor::command;
use crate::{CommandRunner, UpdaterError};

pub(crate) fn run_migration(
    runner: &dyn CommandRunner,
    migrator: &Path,
    database: &Path,
    sql_root: &Path,
) -> Result<(), UpdaterError> {
    let args = [
        OsString::from(database.as_os_str()),
        OsString::from(sql_root.as_os_str()),
    ];
    runner
        .run(&command(
            "migrating",
            migrator,
            args,
            Duration::from_secs(300),
        ))
        .map_err(|error| UpdaterError::MigrationFailed(error.to_string()))?;
    Ok(())
}
