use std::path::{Path, PathBuf};

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
    usb_control_db_migrate::run_migrations(path, &sql_root).expect("migrate test database");
}

fn server_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve server root")
}
