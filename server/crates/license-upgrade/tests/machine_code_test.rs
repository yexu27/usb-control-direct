//! 机器码模块集成测试。
//!
//! 完整的 `generate_machine_code()` 依赖 `/proc/cpuinfo` 和 `/sys/class/net`，
//! 仅在 Linux 装置上可用。此处仅测试 QR PNG 生成等不依赖系统文件的功能。

use license_upgrade::machine_code::generate_qrcode_png;

#[test]
fn qrcode_png_is_valid_png_format() {
    let png = generate_qrcode_png("ABCDEF123456").unwrap();
    // PNG 魔数: 89 50 4E 47 0D 0A 1A 0A
    assert!(png.len() > 8, "PNG 数据过短");
    assert_eq!(
        &png[0..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        "PNG 魔数不匹配"
    );
}

#[test]
fn qrcode_png_different_input_produces_different_output() {
    let png1 = generate_qrcode_png("input-a").unwrap();
    let png2 = generate_qrcode_png("input-b").unwrap();
    assert_ne!(png1, png2, "不同输入应产生不同的二维码");
}

#[test]
fn qrcode_png_empty_input_still_works() {
    // 空字符串也应能生成有效二维码
    let png = generate_qrcode_png("").unwrap();
    assert!(png.len() > 8);
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}
