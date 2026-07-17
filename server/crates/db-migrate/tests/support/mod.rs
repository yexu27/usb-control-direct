use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tempfile::TempDir;

pub struct Fixture {
    _temp: TempDir,
    pub database: PathBuf,
    pub sql_root: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let sql_root = temp.path().join("db");
        fs::create_dir_all(sql_root.join("migrations")).unwrap();
        fs::create_dir_all(sql_root.join("seeds")).unwrap();
        fs::copy(
            deploy_db_root().join("migrations/0001_init.sql"),
            sql_root.join("migrations/0001_init.sql"),
        )
        .unwrap();
        fs::copy(
            deploy_db_root().join("seeds/0001_default_data.sql"),
            sql_root.join("seeds/0001_default_data.sql"),
        )
        .unwrap();
        let database = temp.path().join("device.db");
        Self {
            _temp: temp,
            database,
            sql_root,
        }
    }

    pub fn connection(&self) -> Connection {
        Connection::open(&self.database).unwrap()
    }

    pub fn write_migration(&self, name: &str, sql: &str) {
        fs::write(self.sql_root.join("migrations").join(name), sql).unwrap();
    }

    pub fn replace_seed(&self, sql: &str) {
        fs::write(self.sql_root.join("seeds/0001_default_data.sql"), sql).unwrap();
    }

    pub fn restore_seed(&self) {
        fs::copy(
            deploy_db_root().join("seeds/0001_default_data.sql"),
            self.sql_root.join("seeds/0001_default_data.sql"),
        )
        .unwrap();
    }
}

fn deploy_db_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/db")
}

pub fn scalar_i64(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).unwrap()
}

pub fn scalar_string(conn: &Connection, sql: &str) -> String {
    conn.query_row(sql, [], |row| row.get(0)).unwrap()
}

pub fn table_exists(conn: &Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [table],
        |row| row.get(0),
    )
    .unwrap()
}

pub fn business_table_names(conn: &Connection) -> Vec<String> {
    let mut statement = conn
        .prepare(
            "SELECT name
             FROM sqlite_master
             WHERE type='table' AND name NOT LIKE 'sqlite_%'
             ORDER BY name",
        )
        .unwrap();
    statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}
