//! T01-T11 全表 DDL、索引、初始化数据与 schema 版本管理。
//!
//! 与 `docs/architecture/08-数据库设计.md` 逐字对齐。

use rusqlite::Connection;
use tracing::debug;

use crate::error::StorageError;

/// 当前 schema 版本。
const CURRENT_SCHEMA_VERSION: i32 = 1;

/// 获取数据库当前 schema 版本。
pub fn get_schema_version(conn: &Connection) -> Result<i32, StorageError> {
    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    Ok(version)
}

/// 设置数据库 schema 版本。
fn set_schema_version(conn: &Connection, version: i32) -> Result<(), StorageError> {
    conn.execute_batch(&format!("PRAGMA user_version = {};", version))?;
    Ok(())
}

/// 执行 schema 初始化（建表 + 初始数据 + 设置 user_version）。
///
/// 仅在 user_version == 0 时执行（首次建库）。
pub fn migrate(conn: &Connection) -> Result<(), StorageError> {
    debug!("开始数据库 schema 迁移");

    let version = get_schema_version(conn)?;
    if version >= CURRENT_SCHEMA_VERSION {
        return Ok(());
    }

    if version == 0 {
        create_all_tables(conn)?;
        insert_initial_data(conn)?;
        set_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
    }

    debug!("数据库 schema 迁移完成");
    Ok(())
}

/// 创建 T01-T11 全部表和索引。
fn create_all_tables(conn: &Connection) -> Result<(), StorageError> {
    debug!("创建/检查全部数据表和索引 (T01-T11)");
    conn.execute_batch(
        "
        -- T01 白名单表
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

        -- T02 文件类型黑名单表
        CREATE TABLE IF NOT EXISTS file_type_blacklist (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            extension   TEXT    NOT NULL UNIQUE,
            description TEXT,
            is_default  INTEGER NOT NULL DEFAULT 0,
            created_at  INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_file_type_blacklist_ext ON file_type_blacklist(extension);

        -- T03 文件访问控制开关表
        CREATE TABLE IF NOT EXISTS file_access_policy (
            policy_key  TEXT    PRIMARY KEY,
            enabled     INTEGER NOT NULL DEFAULT 0,
            updated_at  INTEGER NOT NULL
        );

        -- T04 可执行程序控制表
        CREATE TABLE IF NOT EXISTS exec_type (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            type_name   TEXT    NOT NULL UNIQUE,
            description TEXT
        );

        -- T05 USB 审计日志表
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

        -- T06 恶意代码检测日志表
        CREATE TABLE IF NOT EXISTS malware_log (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_time       INTEGER NOT NULL,
            device_sn       TEXT,
            device_name     TEXT,
            file_path       TEXT,
            scan_result     INTEGER NOT NULL,
            virus_name      TEXT,
            virus_db_version TEXT,
            process_result  INTEGER,
            fail_reason     TEXT,
            detail          TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_malware_log_time ON malware_log(scan_time);
        CREATE INDEX IF NOT EXISTS idx_malware_log_sn ON malware_log(device_sn);
        CREATE INDEX IF NOT EXISTS idx_malware_log_result ON malware_log(scan_result);
        CREATE INDEX IF NOT EXISTS idx_malware_log_virus ON malware_log(virus_name);

        -- T07 系统配置表
        CREATE TABLE IF NOT EXISTS system_config (
            config_key   TEXT    PRIMARY KEY,
            config_value TEXT,
            updated_at   INTEGER NOT NULL
        );

        -- T08 用户表
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

        -- T09 角色权限表
        CREATE TABLE IF NOT EXISTS role_permission (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            role      INTEGER NOT NULL,
            page_key  TEXT    NOT NULL,
            UNIQUE(role, page_key)
        );

        -- T10 操作日志表
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

        -- T11 日志覆盖追溯表
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
        ",
    )?;
    Ok(())
}

/// 插入初始化数据。
fn insert_initial_data(conn: &Connection) -> Result<(), StorageError> {
    debug!("插入默认初始数据");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    insert_default_blacklist(conn, now)?;
    insert_default_policies(conn, now)?;
    insert_default_exec_types(conn)?;
    insert_default_system_config(conn, now)?;
    insert_default_users(conn, now)?;
    insert_default_role_permissions(conn)?;
    Ok(())
}

/// T02: 38 种默认文件类型黑名单。
fn insert_default_blacklist(conn: &Connection, now: i64) -> Result<(), StorageError> {
    let extensions = [
        (".jse", "编码后的 JavaScript 脚本"),
        (".vbe", "编码后的 VBScript 脚本"),
        (".vb", "VBScript 文件变体"),
        (".psm1", "PowerShell 模块文件"),
        (".psd1", "PowerShell 模块数据文件"),
        (".cpl", "控制面板扩展项，本质是 DLL"),
        (".msp", "Windows Installer 补丁"),
        (".mst", "Windows Installer 转换文件"),
        (".appref-ms", "ClickOnce 部署引用"),
        (".docm", "含宏的 Word 文档"),
        (".xlsm", "含宏的 Excel 文档"),
        (".pptm", "含宏的 PowerPoint 文档"),
        (".dotm", "含宏的 Word 模板"),
        (".pl", "Perl 脚本"),
        (".rb", "Ruby 脚本"),
        (".php", "PHP 脚本"),
        (".pyc", "Python 编译字节码"),
        (".gadget", "Windows 桌面小工具"),
        (".scr", "屏幕保护程序，本质是 exe"),
        (".msi", "Windows Installer 安装包"),
        (".ps1", "PowerShell 脚本"),
        (".vbs", "VBScript 脚本"),
        (".js", "JavaScript（WSH 宿主环境）"),
        (".bat", "批处理文件"),
        (".cmd", "Windows 命令脚本"),
        (".pif", "程序信息文件"),
        (".com", "DOS 可执行文件"),
        (".wsf", "Windows Script File"),
        (".hta", "HTML Application"),
        (".jar", "Java 程序"),
        (".lnk", "快捷方式"),
        (".reg", "注册表文件"),
        (".sh", "Shell 脚本"),
        (".bin", "二进制可执行文件"),
        (".run", "自解压安装脚本"),
        (".appimage", "Linux 应用打包格式"),
        (".py", "Python 脚本"),
        (".msc", "Microsoft 管理控制台单元"),
    ];

    let mut stmt = conn.prepare(
        "INSERT INTO file_type_blacklist (extension, description, is_default, created_at) VALUES (?1, ?2, 1, ?3)",
    )?;
    for (ext, desc) in &extensions {
        stmt.execute(rusqlite::params![ext, desc, now])?;
    }
    Ok(())
}

/// T03: 三个文件访问控制开关初始值（均默认关闭）。
fn insert_default_policies(conn: &Connection, now: i64) -> Result<(), StorageError> {
    let mut stmt = conn.prepare(
        "INSERT INTO file_access_policy (policy_key, enabled, updated_at) VALUES (?1, 0, ?2)",
    )?;
    for key in &["exec_control", "auto_read_control", "file_type_blacklist_control"] {
        stmt.execute(rusqlite::params![key, now])?;
    }
    Ok(())
}

/// T04: 4 种内置可执行程序类型。
fn insert_default_exec_types(conn: &Connection) -> Result<(), StorageError> {
    let types = [
        ("dll", "动态链接库"),
        ("exe", "Windows 可执行文件"),
        ("PE", "Windows PE 可执行文件格式"),
        ("ELF", "Linux 原生可执行文件格式"),
    ];
    let mut stmt =
        conn.prepare("INSERT INTO exec_type (type_name, description) VALUES (?1, ?2)")?;
    for (name, desc) in &types {
        stmt.execute(rusqlite::params![name, desc])?;
    }
    Ok(())
}

/// T07: 系统配置初始值。
fn insert_default_system_config(conn: &Connection, now: i64) -> Result<(), StorageError> {
    let configs: &[(&str, &str)] = &[
        ("device_description", "(AD USB protection dev)USB Device"),
        ("auth_status", "\"unauthorized\""),
        ("auth_expire_time", "0"),
        ("machine_code", "\"\""),
        ("system_version", "\"1.0.0\""),
        ("virus_db_version", "\"\""),
        ("virus_db_updated_at", "0"),
    ];
    let mut stmt = conn.prepare(
        "INSERT INTO system_config (config_key, config_value, updated_at) VALUES (?1, ?2, ?3)",
    )?;
    for (key, value) in configs {
        stmt.execute(rusqlite::params![key, value, now])?;
    }
    Ok(())
}

/// T08: 三个内置账号。密码 bcrypt hash 使用 cost=12。
fn insert_default_users(conn: &Connection, now: i64) -> Result<(), StorageError> {
    let users: &[(&str, &str, i32)] = &[
        (
            "admin",
            "$2b$12$ZDhWMHU7IE.y3Bwj8iRmrekwJT52DQxDx33mVNz3hbLCZ9g5/NLwO",
            0,
        ),
        (
            "operator",
            "$2b$12$nC77GZiBjNtullz9Zu4YUuDQ0XYMXKxcLWLkrYDDUYQKxBnaVj7X2",
            1,
        ),
        (
            "audit",
            "$2b$12$07S1.9Mzmw26mMpp4zcx.O3gdFN/lMSmOpKo3xG01e2c4R52KW/HK",
            2,
        ),
    ];

    let mut stmt = conn.prepare(
        "INSERT INTO users (username, password_hash, role, status, is_builtin, login_fail_count, created_at, updated_at) \
         VALUES (?1, ?2, ?3, 0, 1, 0, ?4, ?4)",
    )?;
    for (username, hash, role) in users {
        stmt.execute(rusqlite::params![username, hash, role, now])?;
    }
    Ok(())
}

/// T09: 角色权限预置数据（6 行）。
fn insert_default_role_permissions(conn: &Connection) -> Result<(), StorageError> {
    let permissions: &[(i32, &str)] = &[
        (0, "system_management"),
        (0, "user_management"),
        (1, "file_access_control"),
        (1, "usb_device_control"),
        (1, "policy_management"),
        (2, "log_management"),
    ];
    let mut stmt =
        conn.prepare("INSERT INTO role_permission (role, page_key) VALUES (?1, ?2)")?;
    for (role, page_key) in permissions {
        stmt.execute(rusqlite::params![role, page_key])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA auto_vacuum = FULL; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;").unwrap();
        migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn migrate_creates_all_tables() {
        let conn = setup_db();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        let expected = vec![
            "exec_type", "file_access_policy", "file_type_blacklist",
            "log_retention_event", "malware_log", "operation_log",
            "role_permission", "system_config", "usb_audit_log",
            "usb_whitelist", "users",
        ];
        assert_eq!(tables, expected);
    }

    #[test]
    fn migrate_sets_schema_version() {
        let conn = setup_db();
        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn migrate_idempotent() {
        let conn = setup_db();
        migrate(&conn).unwrap();
        assert_eq!(get_schema_version(&conn).unwrap(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn default_blacklist_matches_prd_v1_0_1_entries() {
        let conn = setup_db();
        let actual: Vec<(String, String, i32)> = conn
            .prepare(
                "SELECT extension, description, is_default FROM file_type_blacklist ORDER BY id",
            )
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .map(|row| row.unwrap())
            .collect();
        let expected = vec![
            (".jse", "编码后的 JavaScript 脚本", 1),
            (".vbe", "编码后的 VBScript 脚本", 1),
            (".vb", "VBScript 文件变体", 1),
            (".psm1", "PowerShell 模块文件", 1),
            (".psd1", "PowerShell 模块数据文件", 1),
            (".cpl", "控制面板扩展项，本质是 DLL", 1),
            (".msp", "Windows Installer 补丁", 1),
            (".mst", "Windows Installer 转换文件", 1),
            (".appref-ms", "ClickOnce 部署引用", 1),
            (".docm", "含宏的 Word 文档", 1),
            (".xlsm", "含宏的 Excel 文档", 1),
            (".pptm", "含宏的 PowerPoint 文档", 1),
            (".dotm", "含宏的 Word 模板", 1),
            (".pl", "Perl 脚本", 1),
            (".rb", "Ruby 脚本", 1),
            (".php", "PHP 脚本", 1),
            (".pyc", "Python 编译字节码", 1),
            (".gadget", "Windows 桌面小工具", 1),
            (".scr", "屏幕保护程序，本质是 exe", 1),
            (".msi", "Windows Installer 安装包", 1),
            (".ps1", "PowerShell 脚本", 1),
            (".vbs", "VBScript 脚本", 1),
            (".js", "JavaScript（WSH 宿主环境）", 1),
            (".bat", "批处理文件", 1),
            (".cmd", "Windows 命令脚本", 1),
            (".pif", "程序信息文件", 1),
            (".com", "DOS 可执行文件", 1),
            (".wsf", "Windows Script File", 1),
            (".hta", "HTML Application", 1),
            (".jar", "Java 程序", 1),
            (".lnk", "快捷方式", 1),
            (".reg", "注册表文件", 1),
            (".sh", "Shell 脚本", 1),
            (".bin", "二进制可执行文件", 1),
            (".run", "自解压安装脚本", 1),
            (".appimage", "Linux 应用打包格式", 1),
            (".py", "Python 脚本", 1),
            (".msc", "Microsoft 管理控制台单元", 1),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn default_policies_has_3_entries() {
        let conn = setup_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM file_access_policy", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn default_exec_types_has_4_entries() {
        let conn = setup_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM exec_type", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn default_users_has_3_builtin() {
        let conn = setup_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users WHERE is_builtin = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn default_role_permissions_has_6_entries() {
        let conn = setup_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM role_permission", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 6);
    }

    #[test]
    fn default_system_config_has_7_entries() {
        let conn = setup_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM system_config", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 7);
    }
}
