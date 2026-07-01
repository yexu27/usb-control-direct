//! exFAT 目录项解析。

use crate::exfat::dir_entry::{ENTRY_TYPE_FILE, ENTRY_TYPE_FILE_NAME, ENTRY_TYPE_STREAM};
use crate::exfat::layout::DIR_ENTRY_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub first_cluster: u32,
    pub data_length: u64,
    pub valid_data_length: u64,
    pub attributes: u16,
    pub entry_offset: usize,
    pub set_len: usize,
    pub is_deleted: bool,
}

pub fn parse_entry_sets(data: &[u8]) -> Result<Vec<ParsedDirectoryEntry>, std::io::Error> {
    let mut entries = Vec::new();
    let mut offset = 0usize;
    while offset + DIR_ENTRY_SIZE as usize <= data.len() {
        let entry_type = data[offset];
        if entry_type == 0x00 {
            offset += DIR_ENTRY_SIZE as usize;
            continue;
        }

        let is_deleted = entry_type == (ENTRY_TYPE_FILE & 0x7f);
        if entry_type != ENTRY_TYPE_FILE && !is_deleted {
            offset += DIR_ENTRY_SIZE as usize;
            continue;
        }

        let secondary_count = data[offset + 1] as usize;
        let set_len = (secondary_count + 1) * DIR_ENTRY_SIZE as usize;
        if offset + set_len > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "directory entry set exceeds buffer",
            ));
        }
        if is_deleted {
            offset += set_len;
            continue;
        }

        let file_entry = &data[offset..offset + DIR_ENTRY_SIZE as usize];
        let attributes = u16::from_le_bytes([file_entry[4], file_entry[5]]);
        let is_dir = attributes & 0x10 != 0;
        let stream_offset = offset + DIR_ENTRY_SIZE as usize;
        if data[stream_offset] != ENTRY_TYPE_STREAM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing stream extension",
            ));
        }

        let first_cluster = u32::from_le_bytes(
            data[stream_offset + 20..stream_offset + 24]
                .try_into()
                .unwrap(),
        );
        let valid_data_length = u64::from_le_bytes(
            data[stream_offset + 8..stream_offset + 16]
                .try_into()
                .unwrap(),
        );
        let data_length = u64::from_le_bytes(
            data[stream_offset + 24..stream_offset + 32]
                .try_into()
                .unwrap(),
        );

        let mut utf16 = Vec::new();
        let mut cursor = stream_offset + DIR_ENTRY_SIZE as usize;
        while cursor < offset + set_len {
            if data[cursor] == ENTRY_TYPE_FILE_NAME {
                for i in 0..15 {
                    let p = cursor + 2 + i * 2;
                    let ch = u16::from_le_bytes([data[p], data[p + 1]]);
                    if ch != 0 {
                        utf16.push(ch);
                    }
                }
            }
            cursor += DIR_ENTRY_SIZE as usize;
        }

        let name = String::from_utf16(&utf16).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid UTF-16 filename")
        })?;
        entries.push(ParsedDirectoryEntry {
            name,
            is_dir,
            first_cluster,
            data_length,
            valid_data_length,
            attributes,
            entry_offset: offset,
            set_len,
            is_deleted,
        });
        offset += set_len;
    }
    Ok(entries)
}
