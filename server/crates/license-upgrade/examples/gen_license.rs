//! 授权文件签发工具（用于测试环境模拟授权服务）。
//!
//! 用法: gen_license <私钥hex> <机器码>
//!
//! 示例:
//!   cargo run --example gen_license "ab12...sk" "bY7DKfz..."
//!
//! 输出: /tmp/license.txt 授权文件

use smcrypto::sm2::Sign;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sk_hex = args.get(1).expect("缺少私钥参数（64 位 hex）");
    let machine_code = args.get(2).map(|s| s.as_str()).unwrap_or("test-machine-code-001");

    let expire_time = chrono::Utc::now().timestamp() + 10 * 365 * 86400;
    let payload = format!(
        r#"{{"machine_code":"{}","expire_time":{}}}"#,
        machine_code, expire_time
    );
    let payload_bytes = payload.as_bytes();

    let signer = Sign::new(sk_hex);
    let sig = signer.sign_raw(payload_bytes);

    // 组装授权文件: [4字节大端 sig_len][signature][payload_json]
    let mut license = Vec::new();
    let sig_len = sig.len() as u32;
    license.extend_from_slice(&sig_len.to_be_bytes());
    license.extend_from_slice(&sig);
    license.extend_from_slice(payload_bytes);

    fs::write("/tmp/license.txt", &license).expect("write license");
    println!("授权文件: /tmp/license.txt ({} bytes)", license.len());
}
