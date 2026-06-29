use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

const SUPPORTED_SCHEMA_VERSION: i32 = 1;

pub fn run_migration(
    database_path: PathBuf,
    sql_root: PathBuf,
    version_file: PathBuf,
) -> Result<(), String> {
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create db dir {} failed: {e}", parent.display()))?;
    }

    let version = fs::read_to_string(&version_file)
        .map_err(|e| format!("read version file {} failed: {e}", version_file.display()))?
        .trim()
        .to_string();
    if version.is_empty() {
        return Err(format!("version file {} is empty", version_file.display()));
    }

    let mut conn = Connection::open(&database_path)
        .map_err(|e| format!("open database {} failed: {e}", database_path.display()))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| format!("enable foreign_keys failed: {e}"))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| format!("set busy timeout failed: {e}"))?;

    let current_version = read_user_version(&conn)?;
    if current_version == 0 {
        execute_sql(&mut conn, &sql_root.join("migrations/0001_init.sql"))?;
        execute_sql(&mut conn, &sql_root.join("seeds/0001_default_data.sql"))?;
    } else if current_version > SUPPORTED_SCHEMA_VERSION {
        return Err(format!(
            "unsupported database user_version {current_version}, supported maximum is {SUPPORTED_SCHEMA_VERSION}"
        ));
    }

    sync_system_version(&conn, &version)?;
    sync_virus_db_package_version(&conn, "v0.0.0")?;
    let clamav_status = clamav_status::read_clamav_status("/usr/bin/clamscan")
        .map_err(|e| format!("read ClamAV virus database status failed: {e}"))?;
    sync_virus_db_status(&conn, &clamav_status)?;
    println!("usb-control-db-migrate: database ready version={version}");
    Ok(())
}

fn read_user_version(conn: &Connection) -> Result<i32, String> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| format!("read user_version failed: {e}"))
}

fn execute_sql(conn: &mut Connection, path: &Path) -> Result<(), String> {
    let sql =
        fs::read_to_string(path).map_err(|e| format!("read sql {} failed: {e}", path.display()))?;
    conn.execute_batch(&sql)
        .map_err(|e| format!("execute sql {} failed: {e}", path.display()))
}

fn sync_system_version(conn: &Connection, version: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO system_config (config_key, config_value, updated_at)
         VALUES ('system_version', ?1, strftime('%s','now'))
         ON CONFLICT(config_key)
         DO UPDATE SET config_value = ?1, updated_at = strftime('%s','now')",
        params![version],
    )
    .map_err(|e| format!("sync system_version failed: {e}"))?;
    Ok(())
}

pub fn sync_virus_db_package_version(conn: &Connection, version: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO system_config (config_key, config_value, updated_at)
         VALUES ('virus_db_package_version', ?1, strftime('%s','now'))
         ON CONFLICT(config_key) DO NOTHING",
        params![version],
    )
    .map_err(|e| format!("sync virus_db_package_version failed: {e}"))?;
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
    .map_err(|e| format!("sync virus_db_version failed: {e}"))?;
    conn.execute(
        "INSERT INTO system_config (config_key, config_value, updated_at)
         VALUES ('virus_db_updated_at', ?1, strftime('%s','now'))
         ON CONFLICT(config_key)
         DO UPDATE SET config_value = ?1, updated_at = strftime('%s','now')",
        params![status.virus_db_updated_at.to_string()],
    )
    .map_err(|e| format!("sync virus_db_updated_at failed: {e}"))?;
    Ok(())
}
