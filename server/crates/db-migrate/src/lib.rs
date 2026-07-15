use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

const MIGRATION_LEDGER_SQL: &str = "
CREATE TABLE schema_migrations (
    version    INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    checksum   TEXT    NOT NULL,
    applied_at INTEGER NOT NULL
);";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub path: PathBuf,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub current_version: u32,
    pub applied_versions: Vec<u32>,
}

pub fn run_migrations(database_path: &Path, sql_root: &Path) -> Result<MigrationReport, String> {
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create db dir {} failed: {error}", parent.display()))?;
    }

    let migrations = discover_migrations(&sql_root.join("migrations"))?;
    let supported_max = migrations
        .last()
        .map(|migration| migration.version)
        .ok_or_else(|| "no database migrations found".to_string())?;
    let mut conn = open_connection(database_path)?;
    let current_version = read_user_version(&conn)?;
    if current_version > supported_max {
        return Err(format!(
            "unsupported database user_version {current_version}, supported maximum is {supported_max}"
        ));
    }

    let ledger_exists = table_exists(&conn, "schema_migrations")?;
    if current_version == 1 && !ledger_exists {
        baseline_legacy_v1(&mut conn, &migrations)?;
    } else if current_version == 0 && !ledger_exists {
        initialize_fresh_database(&mut conn, sql_root, &migrations[0])?;
    } else if current_version == 0 {
        return Err(
            "database user_version 0 unexpectedly has a schema_migrations ledger".to_string(),
        );
    } else if !ledger_exists {
        return Err(format!(
            "database user_version {current_version} has no schema_migrations ledger"
        ));
    }

    verify_applied_migrations(&conn, &migrations)?;
    let mut applied_versions = if current_version == 0 {
        vec![migrations[0].version]
    } else {
        Vec::new()
    };
    let mut version = read_user_version(&conn)?;
    let starting_version = version;
    for migration in migrations
        .iter()
        .filter(|migration| migration.version > starting_version)
    {
        apply_migration(&mut conn, migration)?;
        version = migration.version;
        applied_versions.push(version);
    }

    Ok(MigrationReport {
        current_version: version,
        applied_versions,
    })
}

fn open_connection(database_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(database_path)
        .map_err(|error| format!("open database {} failed: {error}", database_path.display()))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|error| format!("set busy timeout failed: {error}"))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| format!("enable foreign_keys failed: {error}"))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| format!("enable WAL journal mode failed: {error}"))?;
    Ok(conn)
}

fn discover_migrations(directory: &Path) -> Result<Vec<Migration>, String> {
    let entries = fs::read_dir(directory).map_err(|error| {
        format!(
            "read migration directory {} failed: {error}",
            directory.display()
        )
    })?;
    let mut migrations = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("read migration entry failed: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("sql") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("migration filename is not valid UTF-8: {}", path.display()))?;
        let (prefix, _) = stem.split_once('_').ok_or_else(|| {
            format!(
                "migration filename must be <version>_<name>.sql: {}",
                path.display()
            )
        })?;
        let version = prefix
            .parse::<u32>()
            .map_err(|_| format!("invalid migration version in {}", path.display()))?;
        if version == 0 {
            return Err(format!(
                "migration version must be positive: {}",
                path.display()
            ));
        }
        let contents = fs::read(&path)
            .map_err(|error| format!("read migration {} failed: {error}", path.display()))?;
        let migration = Migration {
            version,
            name: stem.to_string(),
            path,
            checksum: sha256_hex(&contents),
        };
        if migrations.insert(version, migration).is_some() {
            return Err(format!("duplicate migration version {version}"));
        }
    }

    let migrations: Vec<_> = migrations.into_values().collect();
    for (index, migration) in migrations.iter().enumerate() {
        let expected = u32::try_from(index + 1).map_err(|_| "too many migrations".to_string())?;
        if migration.version != expected {
            return Err(format!(
                "migration sequence gap: expected version {expected}, found {}",
                migration.version
            ));
        }
    }
    Ok(migrations)
}

fn sha256_hex(contents: &[u8]) -> String {
    Sha256::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn initialize_fresh_database(
    conn: &mut Connection,
    sql_root: &Path,
    initial: &Migration,
) -> Result<(), String> {
    if initial.version != 1 {
        return Err("first migration must be version 1".to_string());
    }
    let schema = read_sql(&initial.path)?;
    let seed_path = sql_root.join("seeds/0001_default_data.sql");
    let seeds = read_sql(&seed_path)?;
    let transaction = begin_immediate(conn)?;
    transaction
        .execute_batch(MIGRATION_LEDGER_SQL)
        .map_err(|error| format!("create migration ledger failed: {error}"))?;
    transaction
        .execute_batch(&schema)
        .map_err(|error| format!("execute SQL {} failed: {error}", initial.path.display()))?;
    transaction
        .execute_batch(&seeds)
        .map_err(|error| format!("execute SQL {} failed: {error}", seed_path.display()))?;
    record_migration(&transaction, initial)?;
    set_user_version(&transaction, initial.version)?;
    check_foreign_keys(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("commit initial migration failed: {error}"))
}

fn baseline_legacy_v1(conn: &mut Connection, migrations: &[Migration]) -> Result<(), String> {
    let initial = migrations
        .first()
        .filter(|migration| migration.version == 1)
        .ok_or_else(|| "version 1 migration is missing".to_string())?;
    let transaction = begin_immediate(conn)?;
    validate_legacy_v1(&transaction)?;
    transaction
        .execute_batch(MIGRATION_LEDGER_SQL)
        .map_err(|error| format!("create legacy migration ledger failed: {error}"))?;
    record_migration(&transaction, initial)?;
    check_foreign_keys(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("commit legacy v1 baseline failed: {error}"))
}

fn apply_migration(conn: &mut Connection, migration: &Migration) -> Result<(), String> {
    let sql = read_sql(&migration.path)?;
    let transaction = begin_immediate(conn)?;
    transaction
        .execute_batch(&sql)
        .map_err(|error| format!("execute SQL {} failed: {error}", migration.path.display()))?;
    record_migration(&transaction, migration)?;
    set_user_version(&transaction, migration.version)?;
    check_foreign_keys(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("commit migration {} failed: {error}", migration.version))
}

fn verify_applied_migrations(conn: &Connection, migrations: &[Migration]) -> Result<(), String> {
    let expected: BTreeMap<_, _> = migrations
        .iter()
        .map(|migration| (migration.version, migration))
        .collect();
    let mut statement = conn
        .prepare("SELECT version, name, checksum FROM schema_migrations ORDER BY version")
        .map_err(|error| format!("read migration ledger failed: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| format!("query migration ledger failed: {error}"))?;
    let mut highest = 0;
    for row in rows {
        let (version, name, checksum) =
            row.map_err(|error| format!("decode migration ledger failed: {error}"))?;
        let migration = expected.get(&version).ok_or_else(|| {
            format!("applied migration {version} is not supported by this release")
        })?;
        if name != migration.name || checksum != migration.checksum {
            return Err(format!(
                "migration {version} checksum or name differs from the applied migration"
            ));
        }
        highest = version;
    }
    let user_version = read_user_version(conn)?;
    if highest != user_version {
        return Err(format!(
            "migration ledger version {highest} does not match user_version {user_version}"
        ));
    }
    Ok(())
}

fn validate_legacy_v1(transaction: &Transaction<'_>) -> Result<(), String> {
    const TABLES: &[&str] = &[
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
    const INDEXES: &[&str] = &[
        "idx_usb_whitelist_sn",
        "idx_file_type_blacklist_ext",
        "idx_usb_audit_log_time",
        "idx_usb_audit_log_sn",
        "idx_usb_audit_log_type",
        "idx_usb_audit_log_time_type",
        "idx_malware_log_time",
        "idx_malware_log_sn",
        "idx_malware_log_result",
        "idx_malware_log_virus",
        "idx_users_username",
        "idx_users_status",
        "idx_operation_log_time",
        "idx_operation_log_user",
        "idx_operation_log_type",
        "idx_operation_log_reqid",
        "idx_log_retention_time",
        "idx_log_retention_cat",
    ];
    for table in TABLES {
        if !object_exists(transaction, "table", table)? {
            return Err(format!(
                "legacy v1 validation failed: missing table {table}"
            ));
        }
    }
    for index in INDEXES {
        if !object_exists(transaction, "index", index)? {
            return Err(format!(
                "legacy v1 validation failed: missing index {index}"
            ));
        }
    }
    validate_columns(
        transaction,
        "users",
        &["username", "password_hash", "role", "status", "is_builtin"],
    )?;
    validate_columns(
        transaction,
        "operation_log",
        &["op_time", "username", "log_type", "result", "request_id"],
    )?;
    validate_columns(
        transaction,
        "system_config",
        &["config_key", "config_value", "updated_at"],
    )?;

    for (username, role) in [("admin", 0), ("operator", 1), ("audit", 2)] {
        let found: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM users WHERE username=?1 AND role=?2 AND is_builtin=1)",
                params![username, role],
                |row| row.get(0),
            )
            .map_err(|error| format!("legacy v1 validation failed for user {username}: {error}"))?;
        if !found {
            return Err(format!(
                "legacy v1 validation failed: builtin user {username} is missing or invalid"
            ));
        }
    }
    for (role, page) in [
        (0, "system_management"),
        (0, "user_management"),
        (1, "file_access_control"),
        (1, "usb_device_control"),
        (1, "policy_management"),
        (2, "log_management"),
    ] {
        let found: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM role_permission WHERE role=?1 AND page_key=?2)",
                params![role, page],
                |row| row.get(0),
            )
            .map_err(|error| format!("legacy v1 permission validation failed: {error}"))?;
        if !found {
            return Err(format!(
                "legacy v1 validation failed: permission ({role}, {page}) is missing"
            ));
        }
    }
    Ok(())
}

fn validate_columns(
    transaction: &Transaction<'_>,
    table: &str,
    required: &[&str],
) -> Result<(), String> {
    let mut statement = transaction
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| format!("legacy v1 read columns for {table} failed: {error}"))?;
    let columns: BTreeSet<String> = statement
        .query_map([], |row| row.get(1))
        .map_err(|error| format!("legacy v1 query columns for {table} failed: {error}"))?
        .collect::<Result<_, _>>()
        .map_err(|error| format!("legacy v1 decode columns for {table} failed: {error}"))?;
    for column in required {
        if !columns.contains(*column) {
            return Err(format!(
                "legacy v1 validation failed: table {table} is missing column {column}"
            ));
        }
    }
    Ok(())
}

fn object_exists(conn: &Connection, object_type: &str, name: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type=?1 AND name=?2)",
        params![object_type, name],
        |row| row.get(0),
    )
    .map_err(|error| format!("check {object_type} {name} failed: {error}"))
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool, String> {
    object_exists(conn, "table", table)
}

fn begin_immediate(conn: &mut Connection) -> Result<Transaction<'_>, String> {
    conn.transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("begin immediate transaction failed: {error}"))
}

fn read_sql(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("read SQL {} failed: {error}", path.display()))
}

fn read_user_version(conn: &Connection) -> Result<u32, String> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| format!("read user_version failed: {error}"))
}

fn set_user_version(transaction: &Transaction<'_>, version: u32) -> Result<(), String> {
    transaction
        .pragma_update(None, "user_version", version)
        .map_err(|error| format!("set user_version to {version} failed: {error}"))
}

fn record_migration(transaction: &Transaction<'_>, migration: &Migration) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO schema_migrations(version, name, checksum, applied_at)
             VALUES (?1, ?2, ?3, strftime('%s','now'))",
            params![migration.version, migration.name, migration.checksum],
        )
        .map_err(|error| format!("record migration {} failed: {error}", migration.version))?;
    Ok(())
}

fn check_foreign_keys(transaction: &Transaction<'_>) -> Result<(), String> {
    let violation = transaction
        .query_row("PRAGMA foreign_key_check", [], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .optional()
        .map_err(|error| format!("foreign_key_check failed: {error}"))?;
    if let Some((table, row_id)) = violation {
        return Err(format!(
            "foreign_key_check found violation in table {table}, row {row_id}"
        ));
    }
    Ok(())
}

pub fn sync_virus_db_package_version(conn: &Connection, version: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO system_config (config_key, config_value, updated_at)
         VALUES ('virus_db_package_version', ?1, strftime('%s','now'))
         ON CONFLICT(config_key) DO NOTHING",
        params![version],
    )
    .map_err(|error| format!("sync virus_db_package_version failed: {error}"))?;
    Ok(())
}

pub fn sync_virus_db_status(
    conn: &Connection,
    status: &clamav_status::ClamavStatus,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO system_config (config_key, config_value, updated_at)
         VALUES ('virus_db_version', ?1, strftime('%s','now'))
         ON CONFLICT(config_key)
         DO UPDATE SET config_value = ?1, updated_at = strftime('%s','now')",
        params![status.virus_db_version],
    )
    .map_err(|error| format!("sync virus_db_version failed: {error}"))?;
    conn.execute(
        "INSERT INTO system_config (config_key, config_value, updated_at)
         VALUES ('virus_db_updated_at', ?1, strftime('%s','now'))
         ON CONFLICT(config_key)
         DO UPDATE SET config_value = ?1, updated_at = strftime('%s','now')",
        params![status.virus_db_updated_at.to_string()],
    )
    .map_err(|error| format!("sync virus_db_updated_at failed: {error}"))?;
    Ok(())
}

use rusqlite::OptionalExtension;
