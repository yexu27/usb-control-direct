use license_upgrade::license::LicenseValidator;
use license_upgrade::production_license::ProductionLicenseValidator;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pk_path = args.get(1).map(|s| s.as_str()).unwrap_or("/etc/usb-control/keys/license_verify.pub");
    let license_path = args.get(2).map(|s| s.as_str()).unwrap_or("/tmp/license_smcrypto.txt");
    let mc = args.get(3).map(|s| s.as_str()).unwrap_or("test-machine-code-001");

    let pk = fs::read(pk_path).expect("read pk");
    let license = fs::read(license_path).expect("read license");
    let validator = ProductionLicenseValidator::new(pk);

    // 解析授权文件看 machine_code
    if license.len() >= 5 {
        let sig_len = u32::from_be_bytes(license[..4].try_into().unwrap()) as usize;
        if license.len() > 4 + sig_len {
            let payload = String::from_utf8_lossy(&license[4 + sig_len..]);
            eprintln!("license payload: {}", payload);
        }
    }
    eprintln!("validator machine_code: {}", mc);

    match validator.validate(&license, mc) {
        Ok(info) => println!("OK expire={}", info.expire_time),
        Err(e) => println!("FAIL: {}", e),
    }
}
