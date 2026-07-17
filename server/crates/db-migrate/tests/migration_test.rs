mod support;

use support::{business_table_names, scalar_i64, scalar_string, table_exists, Fixture};
use usb_control_db_migrate::{
    compare_and_set_system_version, read_upgrade_database_state, run_migrations, set_system_version,
};

const BUSINESS_TABLES: &[&str] = &[
    "exec_type",
    "file_access_policy",
    "file_type_blacklist",
    "log_retention_event",
    "malware_log",
    "operation_log",
    "role_permission",
    "system_config",
    "usb_audit_log",
    "usb_whitelist",
    "users",
];

#[test]
fn fresh_database_initializes_only_the_eleven_business_tables_and_defaults() {
    let fixture = Fixture::new();

    let report = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();

    assert_eq!(report.current_version, 1);
    assert_eq!(report.applied_versions, vec![1]);
    assert_eq!(
        business_table_names(&conn),
        BUSINESS_TABLES
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>()
    );
    assert!(!table_exists(&conn, "schema_migrations"));
    assert_eq!(scalar_i64(&conn, "PRAGMA user_version"), 1);
    assert_eq!(scalar_i64(&conn, "SELECT COUNT(*) FROM users"), 3);
    assert_eq!(
        scalar_i64(
            &conn,
            "SELECT COUNT(*) FROM users
             WHERE is_builtin=1 AND username IN ('admin','operator','audit')"
        ),
        3
    );
    assert_eq!(scalar_i64(&conn, "SELECT COUNT(*) FROM role_permission"), 6);
}

#[test]
fn failed_fresh_initialization_leaves_a_retryable_empty_database() {
    let fixture = Fixture::new();
    fixture.replace_seed("INSERT INTO users(no_such_column) VALUES (1);");

    assert!(run_migrations(&fixture.database, &fixture.sql_root).is_err());
    let conn = fixture.connection();
    assert!(business_table_names(&conn).is_empty());
    assert_eq!(scalar_i64(&conn, "PRAGMA user_version"), 0);
    drop(conn);

    fixture.restore_seed();
    let report = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    assert_eq!(report.current_version, 1);
    assert_eq!(business_table_names(&fixture.connection()).len(), 11);
}

#[test]
fn existing_version_one_database_is_preserved_and_rerun_is_noop() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();
    conn.execute(
        "UPDATE system_config SET config_value='production-device', updated_at=10
         WHERE config_key='device_description'",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE system_config SET config_value='authorized', updated_at=11
         WHERE config_key='auth_status'",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO usb_whitelist
         (serial_number, device_name, device_type, permission, add_method, created_at)
         VALUES ('SN-PRESERVE', '现场设备', 'storage', 1, 0, 12)",
        [],
    )
    .unwrap();
    drop(conn);

    let report = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();

    assert_eq!(report.current_version, 1);
    assert!(report.applied_versions.is_empty());
    assert_eq!(
        scalar_string(
            &conn,
            "SELECT config_value FROM system_config WHERE config_key='device_description'"
        ),
        "production-device"
    );
    assert_eq!(
        scalar_string(
            &conn,
            "SELECT config_value FROM system_config WHERE config_key='auth_status'"
        ),
        "authorized"
    );
    assert_eq!(
        scalar_i64(
            &conn,
            "SELECT COUNT(*) FROM usb_whitelist WHERE serial_number='SN-PRESERVE'"
        ),
        1
    );
}

#[test]
fn existing_nonempty_zero_version_database_is_rejected_without_changes() {
    let fixture = Fixture::new();
    let conn = fixture.connection();
    conn.execute_batch(
        "CREATE TABLE site_marker(value TEXT NOT NULL);
         INSERT INTO site_marker(value) VALUES ('keep-me');",
    )
    .unwrap();
    drop(conn);

    let error = run_migrations(&fixture.database, &fixture.sql_root).unwrap_err();
    assert!(
        error.contains("not initialized"),
        "unexpected error: {error}"
    );
    let conn = fixture.connection();
    assert_eq!(
        scalar_string(&conn, "SELECT value FROM site_marker"),
        "keep-me"
    );
    assert!(!table_exists(&conn, "system_config"));
    assert_eq!(scalar_i64(&conn, "PRAGMA user_version"), 0);
}

#[test]
fn applies_future_migrations_in_order_and_updates_user_version() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    fixture.write_migration(
        "0002_add_source.sql",
        "ALTER TABLE system_config ADD COLUMN source TEXT;",
    );
    fixture.write_migration(
        "0003_index_source.sql",
        "CREATE INDEX idx_system_config_source ON system_config(source);",
    );

    let report = run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    let conn = fixture.connection();

    assert_eq!(report.current_version, 3);
    assert_eq!(report.applied_versions, vec![2, 3]);
    assert_eq!(scalar_i64(&conn, "PRAGMA user_version"), 3);
    assert_eq!(
        scalar_i64(
            &conn,
            "SELECT COUNT(*) FROM pragma_table_info('system_config') WHERE name='source'"
        ),
        1
    );
    assert_eq!(
        scalar_i64(
            &conn,
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type='index' AND name='idx_system_config_source'"
        ),
        1
    );
}

#[test]
fn failed_future_migration_rolls_back_schema_and_user_version() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    fixture.write_migration(
        "0002_broken.sql",
        "CREATE TABLE must_rollback(id INTEGER); INVALID SQL;",
    );

    assert!(run_migrations(&fixture.database, &fixture.sql_root).is_err());
    let conn = fixture.connection();
    assert!(!table_exists(&conn, "must_rollback"));
    assert_eq!(scalar_i64(&conn, "PRAGMA user_version"), 1);
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
fn reads_system_version_and_user_version_together() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();

    let state = read_upgrade_database_state(&fixture.database).unwrap();

    assert_eq!(state.system_version, "1.0.0");
    assert_eq!(state.schema_version, 1);
}

#[test]
fn read_rejects_missing_system_version() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();
    fixture
        .connection()
        .execute(
            "DELETE FROM system_config WHERE config_key='system_version'",
            [],
        )
        .unwrap();

    let error = read_upgrade_database_state(&fixture.database).unwrap_err();

    assert!(
        error.contains("system_version"),
        "unexpected error: {error}"
    );
}

#[test]
fn compare_and_set_updates_matching_source_once() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();

    compare_and_set_system_version(&fixture.database, "1.0.0", "3.0.2", 1234).unwrap();

    let conn = fixture.connection();
    assert_eq!(
        scalar_string(
            &conn,
            "SELECT config_value FROM system_config WHERE config_key='system_version'"
        ),
        "3.0.2"
    );
    assert_eq!(
        scalar_i64(
            &conn,
            "SELECT updated_at FROM system_config WHERE config_key='system_version'"
        ),
        1234
    );
}

#[test]
fn compare_and_set_rejects_changed_source_without_writing() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();

    let error =
        compare_and_set_system_version(&fixture.database, "3.0.1", "3.0.2", 1234).unwrap_err();

    assert!(error.contains("source"), "unexpected error: {error}");
    let conn = fixture.connection();
    assert_eq!(
        scalar_string(
            &conn,
            "SELECT config_value FROM system_config WHERE config_key='system_version'"
        ),
        "1.0.0"
    );
}

#[test]
fn set_version_rejects_prefixed_or_non_three_part_version() {
    let fixture = Fixture::new();
    run_migrations(&fixture.database, &fixture.sql_root).unwrap();

    for invalid in [
        "V3.0.2",
        "3.0",
        "3.0.2.1",
        "3..2",
        "3.0.x",
        "03.0.2",
        "3.00.2",
        "18446744073709551616.0.0",
    ] {
        assert!(
            set_system_version(&fixture.database, invalid, 1234).is_err(),
            "invalid version accepted: {invalid}"
        );
    }

    let conn = fixture.connection();
    assert_eq!(
        scalar_string(
            &conn,
            "SELECT config_value FROM system_config WHERE config_key='system_version'"
        ),
        "1.0.0"
    );
}
