//! 编译 `proto/usb_control.proto` 至 `OUT_DIR/usb_control.rs`。
//!
//! 由 `src/proto.rs` 通过 `include!` 引入。proto 改动后 `cargo build` 自动
//! 重新生成。

use std::path::PathBuf;

fn main() {
    let proto_root: PathBuf = PathBuf::from("../../../proto");
    let proto_file = proto_root.join("usb_control.proto");

    println!("cargo:rerun-if-changed={}", proto_file.display());

    prost_build::Config::new()
        .compile_protos(&[proto_file], &[proto_root])
        .expect("failed to compile usb_control.proto");
}
