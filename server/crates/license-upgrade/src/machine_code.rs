//! 机器码生成。
//!
//! 读取 CPU 序列号和 MAC 地址，经 SM3 哈希后 Base64 编码生成机器码，
//! 并将机器码渲染为二维码 PNG 图片。

use std::io::Cursor;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::DynamicImage;
use image::Luma;
use qrcode::QrCode;
use smcrypto::sm3;

use crate::error::LicenseUpgradeError;

/// 机器码生成结果。
pub struct MachineCodeResult {
    /// Base64 编码的机器码。
    pub machine_code: String,
    /// 二维码 PNG 图片字节。
    pub qrcode_png: Vec<u8>,
}

/// 生成装置机器码及对应二维码 PNG。
///
/// 读取 CPU 序列号和 MAC 地址，拼接后经 SM3 哈希生成唯一标识，
/// 再将标识编码为 Base64 并渲染二维码。
///
/// 返回:
/// - 成功时返回 [`MachineCodeResult`]；失败时返回 [`LicenseUpgradeError`]。
pub fn generate_machine_code() -> Result<MachineCodeResult, LicenseUpgradeError> {
    let cpu_serial = read_cpu_serial()?;
    let mac_address = read_mac_address()?;

    let raw_input = format!("{cpu_serial}:{mac_address}");
    let hash_hex = sm3::sm3_hash(raw_input.as_bytes());
    let hash_bytes = hex_decode(&hash_hex).map_err(|msg| {
        LicenseUpgradeError::MachineCodeError(format!("SM3 哈希解码失败: {msg}"))
    })?;
    let machine_code = STANDARD.encode(hash_bytes);

    let qrcode_png = generate_qrcode_png(&machine_code)?;

    Ok(MachineCodeResult {
        machine_code,
        qrcode_png,
    })
}

/// 生成二维码 PNG 图片。
///
/// 参数:
/// - `data`: 二维码内容字符串。
///
/// 返回:
/// - 成功时返回 PNG 字节序列；失败时返回 [`LicenseUpgradeError`]。
pub fn generate_qrcode_png(data: &str) -> Result<Vec<u8>, LicenseUpgradeError> {
    let code = QrCode::new(data.as_bytes()).map_err(|e| {
        LicenseUpgradeError::Internal(format!("二维码生成失败: {e}"))
    })?;

    let image_buf = code.render::<Luma<u8>>().build();
    let dynamic_image = DynamicImage::ImageLuma8(image_buf);

    let mut png_bytes: Vec<u8> = Vec::new();
    dynamic_image
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| {
            LicenseUpgradeError::Internal(format!("PNG 编码失败: {e}"))
        })?;

    Ok(png_bytes)
}

/// 读取 CPU 序列号。
///
/// 从 `/proc/cpuinfo` 解析 "Serial" 行获取 CPU 序列号。
///
/// 返回:
/// - 成功时返回序列号字符串；失败时返回 [`LicenseUpgradeError::MachineCodeError`]。
fn read_cpu_serial() -> Result<String, LicenseUpgradeError> {
    let content = std::fs::read_to_string("/proc/cpuinfo").map_err(|e| {
        LicenseUpgradeError::MachineCodeError(format!("读取 /proc/cpuinfo 失败: {e}"))
    })?;

    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Serial") {
            let serial = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
            if !serial.is_empty() {
                return Ok(serial.to_string());
            }
        }
    }

    Err(LicenseUpgradeError::MachineCodeError(
        "未找到 CPU Serial 信息".into(),
    ))
}

/// 读取网卡 MAC 地址。
///
/// 从 `/sys/class/net/*/address` 读取 MAC 地址，跳过 lo、docker、veth 接口，
/// 按接口名排序后取第一个有效（非全零）MAC 地址。
///
/// 返回:
/// - 成功时返回 MAC 地址字符串；失败时返回 [`LicenseUpgradeError::MachineCodeError`]。
fn read_mac_address() -> Result<String, LicenseUpgradeError> {
    let net_dir = std::path::Path::new("/sys/class/net");
    let mut entries: Vec<String> = Vec::new();

    let dir_entries = std::fs::read_dir(net_dir).map_err(|e| {
        LicenseUpgradeError::MachineCodeError(format!("读取 /sys/class/net 失败: {e}"))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            LicenseUpgradeError::MachineCodeError(format!("遍历网卡目录失败: {e}"))
        })?;

        let iface_name = entry.file_name().to_string_lossy().to_string();

        // 跳过环回和虚拟接口
        if iface_name == "lo"
            || iface_name.starts_with("docker")
            || iface_name.starts_with("veth")
        {
            continue;
        }

        entries.push(iface_name);
    }

    entries.sort();

    for iface_name in &entries {
        let addr_path = net_dir.join(iface_name).join("address");
        if let Ok(mac) = std::fs::read_to_string(&addr_path) {
            let mac = mac.trim().to_string();
            // 跳过全零 MAC
            if !mac.is_empty() && mac != "00:00:00:00:00:00" {
                return Ok(mac);
            }
        }
    }

    Err(LicenseUpgradeError::MachineCodeError(
        "未找到有效的 MAC 地址".into(),
    ))
}

/// 将 hex 字符串解码为字节数组。
///
/// 参数:
/// - `hex`: 偶数长度的十六进制字符串。
///
/// 返回:
/// - 成功时返回解码后的字节序列；失败时返回错误描述字符串。
fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err(format!(
            "hex 字符串长度必须为偶数，实际长度: {}",
            hex.len()
        ));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| format!("非法十六进制字符: {:?}", &hex[i..i + 2]))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_decode_valid_string() {
        let result = hex_decode("48656c6c6f").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn hex_decode_odd_length_returns_error() {
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn hex_decode_empty_string() {
        let result = hex_decode("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn generate_qrcode_png_produces_valid_png() {
        let png = generate_qrcode_png("test-machine-code").unwrap();
        // PNG 文件以 0x89504E47 开头
        assert!(png.len() > 8);
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }
}
