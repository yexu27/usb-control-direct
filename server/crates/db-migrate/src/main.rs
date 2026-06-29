use std::env;
use std::path::PathBuf;

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

    usb_control_db_migrate::run_migration(
        PathBuf::from(&args[1]),
        PathBuf::from(&args[2]),
        PathBuf::from(&args[3]),
    )
}
