//! SM2 密钥对生成工具。
//!
//! 用法: cargo run --example gen_keypair
//!
//! 输出：私钥hex 公钥hex（分行）
//! 公钥可写入 /etc/usb-control/keys/license_verify.pub

use std::fs;

fn main() {
    let (sk, pk) = smcrypto::sm2::gen_keypair();
    let pk_raw = hex::decode(&pk).expect("decode pk");
    let out_dir = std::path::Path::new(".");
    fs::write(out_dir.join("license_verify.pub"), &pk_raw).expect("write pub key");
    println!("{}", sk);
    println!("{}", pk);
    println!("\n公钥已写入: license_verify.pub");
}
