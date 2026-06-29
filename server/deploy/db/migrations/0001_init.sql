PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;

CREATE TABLE IF NOT EXISTS usb_whitelist (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    serial_number   TEXT    NOT NULL UNIQUE,
    vid             TEXT,
    pid             TEXT,
    device_name     TEXT,
    capacity_bytes  INTEGER,
    device_type     TEXT    NOT NULL DEFAULT 'storage',
    description     TEXT,
    permission      INTEGER NOT NULL DEFAULT 0,
    add_method      INTEGER NOT NULL,
    created_at      INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_usb_whitelist_sn ON usb_whitelist(serial_number);

CREATE TABLE IF NOT EXISTS file_type_blacklist (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    extension   TEXT    NOT NULL UNIQUE,
    description TEXT,
    is_default  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_file_type_blacklist_ext ON file_type_blacklist(extension);

CREATE TABLE IF NOT EXISTS file_access_policy (
    policy_key  TEXT    PRIMARY KEY,
    enabled     INTEGER NOT NULL DEFAULT 0,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS exec_type (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    type_name   TEXT    NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS usb_audit_log (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    event_time          INTEGER NOT NULL,
    device_type         TEXT,
    interface_type      TEXT,
    interface_class     INTEGER,
    interface_subclass  INTEGER,
    interface_protocol  INTEGER,
    device_name         TEXT,
    device_sn           TEXT,
    vid                 TEXT,
    pid                 TEXT,
    event_type          TEXT    NOT NULL,
    permission          INTEGER,
    capacity_bytes      INTEGER,
    detail              TEXT
);
CREATE INDEX IF NOT EXISTS idx_usb_audit_log_time ON usb_audit_log(event_time);
CREATE INDEX IF NOT EXISTS idx_usb_audit_log_sn ON usb_audit_log(device_sn);
CREATE INDEX IF NOT EXISTS idx_usb_audit_log_type ON usb_audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_usb_audit_log_time_type ON usb_audit_log(event_time, event_type);

CREATE TABLE IF NOT EXISTS malware_log (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_time        INTEGER NOT NULL,
    device_sn        TEXT,
    device_name      TEXT,
    file_path        TEXT,
    scan_result      INTEGER NOT NULL,
    virus_name       TEXT,
    virus_db_version TEXT,
    process_result   INTEGER,
    fail_reason      TEXT,
    detail           TEXT
);
CREATE INDEX IF NOT EXISTS idx_malware_log_time ON malware_log(scan_time);
CREATE INDEX IF NOT EXISTS idx_malware_log_sn ON malware_log(device_sn);
CREATE INDEX IF NOT EXISTS idx_malware_log_result ON malware_log(scan_result);
CREATE INDEX IF NOT EXISTS idx_malware_log_virus ON malware_log(virus_name);

CREATE TABLE IF NOT EXISTS system_config (
    config_key   TEXT    PRIMARY KEY,
    config_value TEXT,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    username            TEXT    NOT NULL UNIQUE,
    password_hash       TEXT    NOT NULL,
    role                INTEGER NOT NULL,
    status              INTEGER NOT NULL DEFAULT 0,
    is_builtin          INTEGER NOT NULL DEFAULT 0,
    login_fail_count    INTEGER NOT NULL DEFAULT 0,
    lock_until          INTEGER,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    password_changed_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

CREATE TABLE IF NOT EXISTS role_permission (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    role      INTEGER NOT NULL,
    page_key  TEXT    NOT NULL,
    UNIQUE(role, page_key)
);

CREATE TABLE IF NOT EXISTS operation_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    op_time         INTEGER NOT NULL,
    username        TEXT    NOT NULL,
    role            INTEGER NOT NULL,
    log_type        TEXT    NOT NULL,
    action_type     TEXT,
    target          TEXT,
    before_value    TEXT,
    after_value     TEXT,
    related_file    TEXT,
    related_version TEXT,
    result          INTEGER NOT NULL,
    fail_reason     TEXT,
    source_ip       TEXT,
    app_version     TEXT,
    session_id      TEXT,
    request_id      TEXT,
    detail          TEXT
);
CREATE INDEX IF NOT EXISTS idx_operation_log_time ON operation_log(op_time);
CREATE INDEX IF NOT EXISTS idx_operation_log_user ON operation_log(username);
CREATE INDEX IF NOT EXISTS idx_operation_log_type ON operation_log(log_type);
CREATE INDEX IF NOT EXISTS idx_operation_log_reqid ON operation_log(request_id);

CREATE TABLE IF NOT EXISTS log_retention_event (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    trigger_time            INTEGER NOT NULL,
    log_category            TEXT    NOT NULL,
    storage_usage_percent   INTEGER NOT NULL,
    covered_from_time       INTEGER,
    covered_to_time         INTEGER,
    covered_count           INTEGER NOT NULL DEFAULT 0,
    result                  INTEGER NOT NULL,
    fail_reason             TEXT
);
CREATE INDEX IF NOT EXISTS idx_log_retention_time ON log_retention_event(trigger_time);
CREATE INDEX IF NOT EXISTS idx_log_retention_cat ON log_retention_event(log_category);

PRAGMA user_version = 1;
