use rusqlite::Connection;

use crate::error::StorageError;

const SUPPORTED_SCHEMA_VERSION: i32 = 1;

const REQUIRED_TABLES: &[&str] = &[
    "usb_whitelist",
    "file_type_blacklist",
    "file_access_policy",
    "exec_type",
    "usb_audit_log",
    "malware_log",
    "system_config",
    "users",
    "role_permission",
    "operation_log",
    "log_retention_event",
];

const REQUIRED_CONFIGS: &[&str] = &[
    "device_description",
    "auth_status",
    "auth_expire_time",
    "machine_code",
    "system_version",
    "virus_db_package_version",
    "virus_db_version",
    "virus_db_updated_at",
];

pub(crate) fn verify_database_ready(conn: &Connection) -> Result<(), StorageError> {
    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version == 0 {
        return Err(StorageError::DatabaseNotInitialized(
            "PRAGMA user_version is 0; run usb-control-db-migrate.sh before starting service"
                .into(),
        ));
    }
    if version > SUPPORTED_SCHEMA_VERSION {
        return Err(StorageError::SchemaVersionUnsupported {
            current: version,
            supported: SUPPORTED_SCHEMA_VERSION,
        });
    }

    for table in REQUIRED_TABLES {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table],
            |row| row.get(0),
        )?;
        if exists != 1 {
            return Err(StorageError::DatabaseNotInitialized(format!(
                "required table missing: {table}"
            )));
        }
    }

    for key in REQUIRED_CONFIGS {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM system_config WHERE config_key=?1",
            [key],
            |row| row.get(0),
        )?;
        if exists != 1 {
            return Err(StorageError::DatabaseNotInitialized(format!(
                "required system_config missing: {key}"
            )));
        }
    }

    Ok(())
}
