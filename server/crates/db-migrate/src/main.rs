use std::env;
use std::path::PathBuf;

use rusqlite::Connection;

fn main() {
    if let Err(err) = run() {
        eprintln!("usb-control-db-migrate: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if matches!(args.as_slice(), [_, flag] if matches!(flag.as_str(), "--version" | "-V")) {
        println!("{}", release_info::display_version());
        return Ok(());
    }
    if args.len() != 3 {
        return Err(format!(
            "usage: {} <database-path> <sql-root>",
            args.first()
                .map(String::as_str)
                .unwrap_or("usb-control-db-migrate")
        ));
    }

    let database_path = PathBuf::from(&args[1]);
    let report = usb_control_db_migrate::run_migrations(&database_path, &PathBuf::from(&args[2]))?;
    let conn = Connection::open(&database_path)
        .map_err(|error| format!("reopen database for runtime status sync failed: {error}"))?;
    usb_control_db_migrate::sync_virus_db_package_version(&conn, "v0.0.0")?;
    let clamav_status = clamav_status::read_clamav_status("/usr/bin/clamscan")
        .map_err(|error| format!("read ClamAV virus database status failed: {error}"))?;
    usb_control_db_migrate::sync_virus_db_status(&conn, &clamav_status)?;
    println!(
        "usb-control-db-migrate: database ready schema_version={}",
        report.current_version
    );
    Ok(())
}
