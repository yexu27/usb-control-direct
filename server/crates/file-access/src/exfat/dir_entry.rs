//! exFAT 目录项生成。
//!
//! 每个文件/目录由一组目录项组成：
//! - File Entry (0x85): 属性、时间戳
//! - Stream Extension (0xC0): 数据长度、起始簇、文件名哈希
//! - FileName Entries (0xC1): 文件名 UTF-16LE，每个条目 15 字符

use crate::exfat::layout::DIR_ENTRY_SIZE;

/// 目录项类型。
pub const ENTRY_TYPE_FILE: u8 = 0x85;
pub const ENTRY_TYPE_STREAM: u8 = 0xC0;
pub const ENTRY_TYPE_FILE_NAME: u8 = 0xC1;
pub const ENTRY_TYPE_VOLUME_LABEL: u8 = 0x83;
pub const ENTRY_TYPE_BITMAP: u8 = 0x81;
pub const ENTRY_TYPE_UPCASE: u8 = 0x82;

/// 文件属性。
pub const ATTR_DIRECTORY: u16 = 0x10;
pub const ATTR_ARCHIVE: u16 = 0x20;
pub const ATTR_READ_ONLY: u16 = 0x01;

/// 每个 FileName Entry 的字符容量。
const CHARS_PER_NAME_ENTRY: usize = 15;

/// 生成 Volume Label 目录项。
pub fn build_volume_label_entry(label: &str) -> Vec<u8> {
    let utf16: Vec<u16> = label.encode_utf16().collect();
    let char_count = utf16.len().min(11);

    let mut entry = vec![0u8; DIR_ENTRY_SIZE as usize];
    entry[0] = ENTRY_TYPE_VOLUME_LABEL;
    entry[1] = char_count as u8;

    for (i, &ch) in utf16.iter().take(char_count).enumerate() {
        let offset = 2 + i * 2;
        entry[offset] = ch as u8;
        entry[offset + 1] = (ch >> 8) as u8;
    }

    entry
}

/// 生成 Allocation Bitmap 目录项。
pub fn build_bitmap_entry(start_cluster: u32, data_length: u64) -> Vec<u8> {
    let mut entry = vec![0u8; DIR_ENTRY_SIZE as usize];
    entry[0] = ENTRY_TYPE_BITMAP;
    // BitmapFlags at offset 1: 0 = first bitmap
    entry[1] = 0;
    // FirstCluster at offset 20 (u32 LE)
    entry[20..24].copy_from_slice(&start_cluster.to_le_bytes());
    // DataLength at offset 24 (u64 LE)
    entry[24..32].copy_from_slice(&data_length.to_le_bytes());
    entry
}

/// 生成 Upcase Table 目录项。
pub fn build_upcase_entry(start_cluster: u32, data_length: u64, checksum: u32) -> Vec<u8> {
    let mut entry = vec![0u8; DIR_ENTRY_SIZE as usize];
    entry[0] = ENTRY_TYPE_UPCASE;
    // TableChecksum at offset 4 (u32 LE)
    entry[4..8].copy_from_slice(&checksum.to_le_bytes());
    // FirstCluster at offset 20 (u32 LE)
    entry[20..24].copy_from_slice(&start_cluster.to_le_bytes());
    // DataLength at offset 24 (u64 LE)
    entry[24..32].copy_from_slice(&data_length.to_le_bytes());
    entry
}

/// 生成文件/目录的完整目录项集（File + Stream + FileName entries）。
///
/// 参数:
///   - name: 文件/目录名。
///   - is_dir: 是否为目录。
///   - start_cluster: 起始簇号（0 表示无数据）。
///   - data_length: 数据长度（字节）。
///   - is_virus: 是否为病毒文件（DataLength=0）。
///
/// 返回:
///   - 目录项字节序列。
pub fn build_file_entry_set(
    name: &str,
    is_dir: bool,
    start_cluster: u32,
    data_length: u64,
    is_virus: bool,
) -> Vec<u8> {
    let utf16_name: Vec<u16> = name.encode_utf16().collect();
    let name_entry_count = (utf16_name.len() + CHARS_PER_NAME_ENTRY - 1) / CHARS_PER_NAME_ENTRY;
    let secondary_count = 1 + name_entry_count; // Stream + FileName entries

    let actual_data_length = if is_virus { 0 } else { data_length };
    let actual_start_cluster = if is_virus { 0 } else { start_cluster };

    let mut entries = Vec::new();

    // File Entry (0x85)
    let mut file_entry = vec![0u8; DIR_ENTRY_SIZE as usize];
    file_entry[0] = ENTRY_TYPE_FILE;
    file_entry[1] = secondary_count as u8;
    // SetChecksum at offset 2 (u16 LE) — 稍后填充

    let attrs = if is_dir {
        ATTR_DIRECTORY
    } else if is_virus {
        ATTR_READ_ONLY | ATTR_ARCHIVE
    } else {
        ATTR_ARCHIVE
    };
    file_entry[4..6].copy_from_slice(&attrs.to_le_bytes());

    // 时间戳：使用固定值（2025-01-01 00:00:00）
    // CreateTimestamp at offset 8 (u32)
    let timestamp = encode_exfat_timestamp(2025, 1, 1, 0, 0, 0);
    file_entry[8..12].copy_from_slice(&timestamp.to_le_bytes());
    // LastModifiedTimestamp at offset 12
    file_entry[12..16].copy_from_slice(&timestamp.to_le_bytes());
    // LastAccessedTimestamp at offset 16
    file_entry[16..20].copy_from_slice(&timestamp.to_le_bytes());

    entries.extend_from_slice(&file_entry);

    // Stream Extension (0xC0)
    let name_hash = compute_name_hash(&utf16_name);
    let mut stream_entry = vec![0u8; DIR_ENTRY_SIZE as usize];
    stream_entry[0] = ENTRY_TYPE_STREAM;

    // GeneralSecondaryFlags at offset 1
    // Bit 0: AllocationPossible = 1
    // Bit 1: NoFatChain = 1（连续分配时可设置）
    let flags = if actual_data_length > 0 { 0x03u8 } else { 0x01u8 };
    stream_entry[1] = flags;

    // NameLength at offset 3
    stream_entry[3] = utf16_name.len() as u8;

    // NameHash at offset 4 (u16 LE)
    stream_entry[4..6].copy_from_slice(&name_hash.to_le_bytes());

    // ValidDataLength at offset 8 (u64 LE)
    stream_entry[8..16].copy_from_slice(&actual_data_length.to_le_bytes());

    // FirstCluster at offset 20 (u32 LE)
    stream_entry[20..24].copy_from_slice(&actual_start_cluster.to_le_bytes());

    // DataLength at offset 24 (u64 LE)
    stream_entry[24..32].copy_from_slice(&actual_data_length.to_le_bytes());

    entries.extend_from_slice(&stream_entry);

    // FileName Entries (0xC1)
    for chunk_idx in 0..name_entry_count {
        let mut name_entry = vec![0u8; DIR_ENTRY_SIZE as usize];
        name_entry[0] = ENTRY_TYPE_FILE_NAME;
        // GeneralSecondaryFlags at offset 1: 0
        name_entry[1] = 0;

        let start = chunk_idx * CHARS_PER_NAME_ENTRY;
        let end = (start + CHARS_PER_NAME_ENTRY).min(utf16_name.len());

        for (i, &ch) in utf16_name[start..end].iter().enumerate() {
            let offset = 2 + i * 2;
            name_entry[offset] = ch as u8;
            name_entry[offset + 1] = (ch >> 8) as u8;
        }

        entries.extend_from_slice(&name_entry);
    }

    // 计算 SetChecksum 并回填 File Entry
    let set_checksum = compute_set_checksum(&entries);
    entries[2] = set_checksum as u8;
    entries[3] = (set_checksum >> 8) as u8;

    entries
}

/// 编码 exFAT 时间戳。
fn encode_exfat_timestamp(year: u16, month: u8, day: u8, hour: u8, min: u8, sec: u8) -> u32 {
    let y = ((year - 1980) as u32) << 25;
    let m = (month as u32) << 21;
    let d = (day as u32) << 16;
    let h = (hour as u32) << 11;
    let mi = (min as u32) << 5;
    let s = (sec as u32 / 2) & 0x1F;
    y | m | d | h | mi | s
}

/// 计算文件名哈希（NameHash）。
pub fn compute_name_hash(name_utf16: &[u16]) -> u16 {
    let mut hash: u16 = 0;
    for &ch in name_utf16 {
        // 使用 upcase 版本
        let upper = simple_upcase(ch);
        let lo = upper as u8;
        let hi = (upper >> 8) as u8;
        hash = hash.rotate_right(1).wrapping_add(lo as u16);
        hash = hash.rotate_right(1).wrapping_add(hi as u16);
    }
    hash
}

/// 简单大小写转换（用于哈希）。
fn simple_upcase(ch: u16) -> u16 {
    if (0x0061..=0x007A).contains(&ch) {
        ch - 0x0020
    } else {
        ch
    }
}

/// 计算 SetChecksum。
fn compute_set_checksum(entry_set: &[u8]) -> u16 {
    let mut checksum: u16 = 0;
    for (i, &byte) in entry_set.iter().enumerate() {
        // 跳过 File Entry 的 SetChecksum 字段（offset 2, 3）
        if i == 2 || i == 3 {
            continue;
        }
        checksum = if checksum & 1 != 0 {
            0x8000u16.wrapping_add(checksum >> 1).wrapping_add(byte as u16)
        } else {
            (checksum >> 1).wrapping_add(byte as u16)
        };
    }
    checksum
}
