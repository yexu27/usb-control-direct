use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

const SUPPORTED_SCHEMA_VERSION: i32 = 1;

fn main() {
    if let Err(err) = run() {
        eprintln!("usb-control-db-migrate: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err(format!(
            "usage: {} <database-path> <sql-root> <version-file>",
            args.first()
                .map(String::as_str)
                .unwrap_or("usb-control-db-migrate")
        ));
    }

    let database_path = PathBuf::from(&args[1]);
    let sql_root = PathBuf::from(&args[2]);
    let version_file = PathBuf::from(&args[3]);

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
