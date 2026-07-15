mod support;

use rusqlite::Connection;
use support::{scalar_i64, scalar_string, table_exists, Fixture};
use usb_control_db_migrate::{run_migrations, sync_virus_db_package_version, sync_virus_db_status};

#[test]
fn records_migration_checksum_atomically() {
    let fixture = Fixture::new();
    let report = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();

    assert_eq!(report.current_version, 1);
    assert_eq!(report.applied_versions, vec![1]);
    assert_eq!(
        scalar_i64(&conn, "SELECT COUNT(*) FROM schema_migrations"),
        1
    );
    assert_eq!(
        scalar_string(&conn, "SELECT name FROM schema_migrations WHERE version=1"),
        "0001_init"
    );
    assert_eq!(
        scalar_string(
            &conn,
            "SELECT checksum FROM schema_migrations WHERE version=1"
        )
        .len(),
        64
    );
}

#[test]
fn rerun_is_idempotent() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let second = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();

    assert!(second.applied_versions.is_empty());
    assert_eq!(scalar_i64(&conn, "SELECT COUNT(*) FROM users"), 3);
    assert_eq!(
        scalar_i64(&conn, "SELECT COUNT(*) FROM schema_migrations"),
        1
    );
}

#[test]
fn changed_applied_checksum_is_rejected() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let init = std::fs::read_to_string(fixture.sql_root.join("migrations/0001_init.sql")).unwrap();
    fixture.replace_init(&format!("{init}\n-- changed after release\n"));

    let error = run_migrations(&fixture.database, &fixture.sql_root).unwrap_err();
    assert!(error.contains("checksum"), "unexpected error: {error}");
}

#[test]
fn failed_single_migration_rolls_back_its_schema_and_metadata() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    fixture.write_migration(
        "0002_broken.sql",
        "CREATE TABLE must_rollback(id INTEGER); INVALID SQL;",
    );

    assert!(run_migrations(&fixture.database, &fixture.sql_root).is_err());
    let conn = fixture.connection();
    assert!(!table_exists(&conn, "must_rollback"));
    assert_eq!(
        scalar_i64(&conn, "SELECT COUNT(*) FROM schema_migrations"),
        1
    );
    assert_eq!(scalar_i64(&conn, "PRAGMA user_version"), 1);
}

#[test]
fn migration_does_not_write_system_version() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();
    conn.execute(
        "UPDATE system_config SET config_value='9.8.7' WHERE config_key='system_version'",
        [],
    )
    .unwrap();
    drop(conn);

    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();
    assert_eq!(
        scalar_string(
            &conn,
            "SELECT config_value FROM system_config WHERE config_key='system_version'"
        ),
        "9.8.7"
    );
}

#[test]
fn rejects_database_newer_than_supported_max() {
    let fixture = Fixture::new();
    let conn = fixture.connection();
    conn.pragma_update(None, "user_version", 2).unwrap();
    drop(conn);

    let error = run_migrations(&fixture.database, &fixture.sql_root).unwrap_err();
    assert!(error.contains("supported maximum is 1"), "{error}");
}

#[test]
fn rejects_zero_version_database_with_existing_migration_ledger() {
    let fixture = Fixture::new();
    let conn = fixture.connection();
    conn.execute_batch(
        "CREATE TABLE schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        );",
    )
    .unwrap();
    drop(conn);

    let error = run_migrations(&fixture.database, &fixture.sql_root).unwrap_err();
    assert!(
        error.contains("user_version 0"),
        "unexpected error: {error}"
    );
}

#[test]
fn fresh_database_initializes_schema_seeds_and_builtin_users_atomically() {
    let valid = Fixture::new();
    run_migrations(&valid.database, &valid.sql_root).unwrap();
    let conn = valid.connection();
    assert_eq!(scalar_i64(&conn, "SELECT COUNT(*) FROM users"), 3);
    assert_eq!(
        scalar_i64(
            &conn,
            "SELECT COUNT(*) FROM users WHERE is_builtin=1 AND username IN ('admin','operator','audit')"
        ),
        3
    );
    assert_eq!(scalar_i64(&conn, "SELECT COUNT(*) FROM role_permission"), 6);

    let fixture = Fixture::new();
    fixture.replace_seed("INSERT INTO users(no_such_column) VALUES (1);");

    assert!(run_migrations(&fixture.database, &fixture.sql_root).is_err());
    let conn = fixture.connection();
    assert!(!table_exists(&conn, "users"));
    assert!(!table_exists(&conn, "schema_migrations"));
    assert_eq!(scalar_i64(&conn, "PRAGMA user_version"), 0);
}

#[test]
fn valid_legacy_user_version_one_is_baselined_once() {
    let fixture = Fixture::new();
    fixture.initialize_legacy_v1();

    let first = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let second = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();
    assert!(first.applied_versions.is_empty());
    assert!(second.applied_versions.is_empty());
    assert_eq!(
        scalar_i64(&conn, "SELECT COUNT(*) FROM schema_migrations"),
        1
    );
}

#[test]
fn malformed_legacy_user_version_one_is_rejected_without_baseline() {
    let fixture = Fixture::new();
    let conn = fixture.connection();
    conn.execute_batch("CREATE TABLE users(id INTEGER PRIMARY KEY);")
        .unwrap();
    conn.pragma_update(None, "user_version", 1).unwrap();
    drop(conn);

    let error = run_migrations(&fixture.database, &fixture.sql_root).unwrap_err();
    assert!(error.contains("legacy"), "unexpected error: {error}");
    let conn = fixture.connection();
    assert!(!table_exists(&conn, "schema_migrations"));
}

#[test]
fn runtime_clamav_status_sync_remains_after_schema_success() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = Connection::open(&fixture.database).unwrap();
    sync_virus_db_package_version(&conn, "v0.0.0").unwrap();
    sync_virus_db_status(
        &conn,
        &clamav_status::ClamavStatus {
            engine_version: "1.4.4".into(),
            virus_db_version: "28045".into(),
            virus_db_updated_at: 1_782_656_776,
            raw_output: String::new(),
        },
    )
    .unwrap();

    assert_eq!(
        scalar_string(
            &conn,
            "SELECT config_value FROM system_config WHERE config_key='virus_db_version'"
        ),
        "28045"
    );
}
