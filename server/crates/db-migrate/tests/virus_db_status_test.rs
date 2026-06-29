use rusqlite::Connection;
use usb_control_db_migrate::sync_virus_db_status;

#[test]
fn sync_virus_db_status_updates_config_values() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE system_config (
            config_key TEXT PRIMARY KEY,
            config_value TEXT,
            updated_at INTEGER NOT NULL
        );",
    )
    .unwrap();
    let status = clamav_status::ClamavStatus {
        engine_version: "1.4.4".into(),
        virus_db_version: "28045".into(),
        virus_db_updated_at: 1_782_656_776,
        raw_output: "ClamAV 1.4.4/28045/Sun Jun 28 14:26:16 2026".into(),
    };

    sync_virus_db_status(&conn, &status).unwrap();

    let version: String = conn
        .query_row(
            "SELECT config_value FROM system_config WHERE config_key='virus_db_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let updated_at: String = conn
        .query_row(
            "SELECT config_value FROM system_config WHERE config_key='virus_db_updated_at'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "28045");
    assert_eq!(updated_at, "1782656776");
}
