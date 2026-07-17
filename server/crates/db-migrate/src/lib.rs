use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Migration {
    version: u32,
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub current_version: u32,
    pub applied_versions: Vec<u32>,
}

/// 在线升级执行前读取的数据库真实状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeDatabaseState {
    pub system_version: String,
    pub schema_version: u32,
}

/// 从同一数据库连接读取业务版本和 Schema 版本。
pub fn read_upgrade_database_state(database_path: &Path) -> Result<UpgradeDatabaseState, String> {
    let conn = open_connection(database_path)?;
    let system_version = conn
        .query_row(
            "SELECT config_value FROM system_config WHERE config_key='system_version'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(|error| format!("read system_version failed: {error}"))?
        .flatten()
        .ok_or_else(|| "system_version is missing".to_string())?;
    validate_system_version(&system_version)?;
    Ok(UpgradeDatabaseState {
        system_version,
        schema_version: read_user_version(&conn)?,
    })
}

/// 仅当数据库仍处于预期源版本时提交目标业务版本。
pub fn compare_and_set_system_version(
    database_path: &Path,
    expected_source: &str,
    target: &str,
    updated_at: i64,
) -> Result<(), String> {
    validate_system_version(expected_source)?;
    validate_system_version(target)?;
    let mut conn = open_connection(database_path)?;
    let transaction = begin_immediate(&mut conn)?;
    let changed = transaction
        .execute(
            "UPDATE system_config
             SET config_value = ?1, updated_at = ?2
             WHERE config_key = 'system_version' AND config_value = ?3",
            params![target, updated_at, expected_source],
        )
        .map_err(|error| format!("compare-and-set system_version failed: {error}"))?;
    if changed != 1 {
        return Err("system_version source does not match".into());
    }
    transaction
        .commit()
        .map_err(|error| format!("commit system_version failed: {error}"))
}

/// 直接安装健康成功后设置已安装业务版本。
pub fn set_system_version(
    database_path: &Path,
    target: &str,
    updated_at: i64,
) -> Result<(), String> {
    validate_system_version(target)?;
    let mut conn = open_connection(database_path)?;
    let transaction = begin_immediate(&mut conn)?;
    let changed = transaction
        .execute(
            "UPDATE system_config
             SET config_value = ?1, updated_at = ?2
             WHERE config_key = 'system_version'",
            params![target, updated_at],
        )
        .map_err(|error| format!("set system_version failed: {error}"))?;
    if changed != 1 {
        return Err("system_version is missing".into());
    }
    transaction
        .commit()
        .map_err(|error| format!("commit system_version failed: {error}"))
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

    let mut applied_versions = Vec::new();
    if current_version == 0 {
        if database_has_user_objects(&conn)? {
            return Err("existing database is not initialized: user_version is 0".into());
        }
        initialize_fresh_database(&mut conn, sql_root, &migrations[0])?;
        applied_versions.push(1);
    }

    let starting_version = read_user_version(&conn)?;
    for migration in migrations
        .iter()
        .filter(|migration| migration.version > starting_version)
    {
        apply_migration(&mut conn, migration)?;
        applied_versions.push(migration.version);
    }

    Ok(MigrationReport {
        current_version: read_user_version(&conn)?,
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
        let (prefix, name) = stem.split_once('_').ok_or_else(|| {
            format!(
                "migration filename must be <version>_<name>.sql: {}",
                path.display()
            )
        })?;
        if name.is_empty() {
            return Err(format!(
                "migration filename must include a name: {}",
                path.display()
            ));
        }
        let version = prefix
            .parse::<u32>()
            .map_err(|_| format!("invalid migration version in {}", path.display()))?;
        if version == 0 {
            return Err(format!(
                "migration version must be positive: {}",
                path.display()
            ));
        }
        let migration = Migration { version, path };
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

fn database_has_user_objects(conn: &Connection) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(
             SELECT 1
             FROM sqlite_master
             WHERE type IN ('table', 'index', 'trigger', 'view')
               AND name NOT LIKE 'sqlite_%'
         )",
        [],
        |row| row.get(0),
    )
    .map_err(|error| format!("inspect existing database objects failed: {error}"))
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
        .execute_batch(&schema)
        .map_err(|error| format!("execute SQL {} failed: {error}", initial.path.display()))?;
    transaction
        .execute_batch(&seeds)
        .map_err(|error| format!("execute SQL {} failed: {error}", seed_path.display()))?;
    set_user_version(&transaction, initial.version)?;
    check_foreign_keys(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("commit initial migration failed: {error}"))
}

fn apply_migration(conn: &mut Connection, migration: &Migration) -> Result<(), String> {
    let sql = read_sql(&migration.path)?;
    let transaction = begin_immediate(conn)?;
    transaction
        .execute_batch(&sql)
        .map_err(|error| format!("execute SQL {} failed: {error}", migration.path.display()))?;
    set_user_version(&transaction, migration.version)?;
    check_foreign_keys(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("commit migration {} failed: {error}", migration.version))
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

fn validate_system_version(version: &str) -> Result<(), String> {
    let mut parts = version.split('.');
    let valid = (0..3).all(|_| {
        parts.next().is_some_and(|part| {
            !(part.is_empty() || (part.len() > 1 && part.starts_with('0')))
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && part.parse::<u64>().is_ok()
        })
    }) && parts.next().is_none();
    if valid {
        Ok(())
    } else {
        Err(format!("invalid system version: {version}"))
    }
}

fn set_user_version(transaction: &Transaction<'_>, version: u32) -> Result<(), String> {
    transaction
        .pragma_update(None, "user_version", version)
        .map_err(|error| format!("set user_version to {version} failed: {error}"))
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
