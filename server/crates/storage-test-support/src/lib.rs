use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tempfile::TempDir;

pub struct TestDb {
    _dir: TempDir,
    path: PathBuf,
}

impl TestDb {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("create temp db dir");
        let path = dir.path().join("device.db");
        initialize_database(&path);
        Self { _dir: dir, path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Default for TestDb {
    fn default() -> Self {
        Self::new()
    }
}

pub fn initialize_database(path: &Path) {
    let sql_root = server_root().join("deploy/db");
    let migration = sql_root.join("migrations/0001_init.sql");
    let seed = sql_root.join("seeds/0001_default_data.sql");

    let conn = Connection::open(path).expect("open test database");
    let migration_sql = fs::read_to_string(&migration).expect("read migration sql");
    conn.execute_batch(&migration_sql)
        .expect("execute migration sql");
    let seed_sql = fs::read_to_string(&seed).expect("read seed sql");
    conn.execute_batch(&seed_sql).expect("execute seed sql");
}

fn server_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve server root")
}
