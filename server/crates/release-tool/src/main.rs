use std::path::PathBuf;

use usb_control_release_tool::{
    build_bin, generate_key, verify_bin, BuildBinRequest, ReleaseToolError,
};

fn main() {
    if let Err(error) = run(std::env::args()) {
        eprintln!("usb-control-release-tool: {error}");
        std::process::exit(1);
    }
}

fn run<I, S>(args: I) -> Result<(), ReleaseToolError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    match args.as_slice() {
        [_, command, key_id_option, key_id, key_dir_option, key_dir]
            if command == "generate-key"
                && key_id_option == "--key-id"
                && key_dir_option == "--key-dir" =>
        {
            generate_key(key_id, &PathBuf::from(key_dir))?;
            println!("upgrade key material created");
        }
        [_, command, deb_option, deb, key_dir_option, key_dir, output_option, output]
            if command == "build-bin"
                && deb_option == "--deb"
                && key_dir_option == "--key-dir"
                && output_option == "--output" =>
        {
            let manifest = build_bin(BuildBinRequest {
                deb_path: &PathBuf::from(deb),
                key_dir: &PathBuf::from(key_dir),
                output_path: &PathBuf::from(output),
            })?;
            println!("upgrade BIN created for V{}", manifest.package_version);
        }
        [_, command, bin_option, bin, key_dir_option, key_dir]
            if command == "verify-bin"
                && bin_option == "--bin"
                && key_dir_option == "--key-dir" =>
        {
            let manifest = verify_bin(&PathBuf::from(bin), &PathBuf::from(key_dir))?;
            println!("upgrade BIN verified for V{}", manifest.package_version);
        }
        _ => {
            return Err(ReleaseToolError::InvalidInput(
                "usage: generate-key | build-bin | verify-bin".into(),
            ));
        }
    }
    Ok(())
}
