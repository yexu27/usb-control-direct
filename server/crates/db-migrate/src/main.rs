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
    println!(
        "usb-control-db-migrate: database ready schema_version={}",
        report.current_version
    );
    Ok(())
}
