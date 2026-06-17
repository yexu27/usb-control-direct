//! Upcase Table 生成。
//!
//! exFAT 大小写转换表，用于文件名比较。使用压缩格式减少体积。

/// 生成 Upcase Table 数据。
///
/// 返回:
///   - (table_data, checksum): 表数据和 UpcaseTableChecksum。
pub fn generate_upcase_table() -> (Vec<u8>, u32) {
    let mut table = Vec::with_capacity(2 * 65536);

    // 完整映射 0x0000 - 0xFFFF
    for code in 0u16..=0xFFFFu16 {
        let upper = to_upper(code);
        table.push(upper as u8);
        table.push((upper >> 8) as u8);

        if code == 0xFFFF {
            break;
        }
    }

    let checksum = compute_upcase_checksum(&table);
    (table, checksum)
}

/// 简单大小写转换。
fn to_upper(ch: u16) -> u16 {
    // ASCII 范围
    if (0x0061..=0x007A).contains(&ch) {
        return ch - 0x0020;
    }
    // Latin-1 Supplement
    if (0x00E0..=0x00F6).contains(&ch) || (0x00F8..=0x00FE).contains(&ch) {
        return ch - 0x0020;
    }
    ch
}

/// 计算 Upcase Table Checksum。
pub fn compute_upcase_checksum(data: &[u8]) -> u32 {
    let mut checksum: u32 = 0;
    for &byte in data {
        checksum = if checksum & 1 != 0 {
            0x80000000u32.wrapping_add(checksum >> 1).wrapping_add(byte as u32)
        } else {
            (checksum >> 1).wrapping_add(byte as u32)
        };
    }
    checksum
}
