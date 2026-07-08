use file_access::exfat::dir_entry::*;

fn stream_entry(entry_set: &[u8]) -> &[u8] {
    &entry_set[32..64]
}

fn stream_flags(entry_set: &[u8]) -> u8 {
    stream_entry(entry_set)[1]
}

fn stream_valid_data_length(entry_set: &[u8]) -> u64 {
    u64::from_le_bytes(stream_entry(entry_set)[8..16].try_into().unwrap())
}

fn stream_data_length(entry_set: &[u8]) -> u64 {
    u64::from_le_bytes(stream_entry(entry_set)[24..32].try_into().unwrap())
}

#[test]
fn file_entry_set_basic_structure() {
    let data = build_file_entry_set("readme.txt", false, 5, 1024, false);

    // File Entry + Stream Extension + 1 FileName Entry = 3 * 32 = 96 bytes
    assert_eq!(data.len(), 96);

    // File Entry type
    assert_eq!(data[0], ENTRY_TYPE_FILE);
    // SecondaryCount = 2 (stream + 1 name)
    assert_eq!(data[1], 2);

    // Stream Extension type
    assert_eq!(data[32], ENTRY_TYPE_STREAM);

    // FileName Entry type
    assert_eq!(data[64], ENTRY_TYPE_FILE_NAME);
}

#[test]
fn file_entry_set_data_length() {
    let data = build_file_entry_set("test.bin", false, 5, 0x1_0000_0000, false);

    // Stream Extension DataLength at offset 32+24
    let data_len = u64::from_le_bytes(data[56..64].try_into().unwrap());
    assert_eq!(data_len, 0x1_0000_0000); // > 4GiB
}

#[test]
fn read_only_file_preserves_data_length_and_cluster() {
    let data = build_file_entry_set("[病毒禁止访问]virus.exe", false, 5, 1024, true);

    // Stream Extension DataLength
    let data_len = u64::from_le_bytes(data[56..64].try_into().unwrap());
    assert_eq!(data_len, 1024);

    // Stream Extension FirstCluster
    let first_cluster = u32::from_le_bytes(data[52..56].try_into().unwrap());
    assert_eq!(first_cluster, 5);

    let attrs = u16::from_le_bytes(data[4..6].try_into().unwrap());
    assert_eq!(attrs & ATTR_READ_ONLY, ATTR_READ_ONLY);
}

#[test]
fn directory_entry_has_dir_attribute() {
    let data = build_file_entry_set("docs", true, 3, 0, false);

    // FileAttributes at offset 4
    let attrs = u16::from_le_bytes(data[4..6].try_into().unwrap());
    assert_eq!(attrs & ATTR_DIRECTORY, ATTR_DIRECTORY);
}

#[test]
fn directory_entry_with_allocated_cluster_has_directory_stream_length() {
    let entry_set = build_file_entry_set("t2", true, 42, 4096, false);

    assert_eq!(stream_valid_data_length(&entry_set), 4096);
    assert_eq!(stream_data_length(&entry_set), 4096);
    assert_eq!(stream_flags(&entry_set) & 0x01, 0x01);
}

#[test]
fn directory_entry_without_allocated_cluster_has_zero_stream_length() {
    let entry_set = build_file_entry_set("empty", true, 0, 0, false);

    assert_eq!(stream_valid_data_length(&entry_set), 0);
    assert_eq!(stream_data_length(&entry_set), 0);
    assert_eq!(stream_flags(&entry_set) & 0x01, 0x01);
}

#[test]
fn long_filename_uses_multiple_name_entries() {
    let long_name = "这是一个很长的中文文件名测试用例.txt";
    let utf16_len = long_name.encode_utf16().count();
    let name_entries = (utf16_len + 14) / 15; // ceil division
    let expected_size = (2 + name_entries) * 32; // File + Stream + Name entries

    let data = build_file_entry_set(long_name, false, 5, 100, false);
    assert_eq!(data.len(), expected_size);

    // SecondaryCount
    assert_eq!(data[1] as usize, 1 + name_entries);
}

#[test]
fn set_checksum_is_nonzero() {
    let data = build_file_entry_set("test.txt", false, 5, 100, false);
    let checksum = u16::from_le_bytes(data[2..4].try_into().unwrap());
    assert_ne!(checksum, 0);
}

#[test]
fn volume_label_entry() {
    let entry = build_volume_label_entry("USB_DRIVE");
    assert_eq!(entry.len(), 32);
    assert_eq!(entry[0], ENTRY_TYPE_VOLUME_LABEL);
    assert_eq!(entry[1], 9); // "USB_DRIVE" = 9 chars
}

#[test]
fn bitmap_entry() {
    let entry = build_bitmap_entry(3, 128);
    assert_eq!(entry[0], ENTRY_TYPE_BITMAP);
    let cluster = u32::from_le_bytes(entry[20..24].try_into().unwrap());
    assert_eq!(cluster, 3);
    let length = u64::from_le_bytes(entry[24..32].try_into().unwrap());
    assert_eq!(length, 128);
}

#[test]
fn upcase_entry() {
    let entry = build_upcase_entry(4, 262144, 0xDEADBEEF);
    assert_eq!(entry[0], ENTRY_TYPE_UPCASE);
    let checksum = u32::from_le_bytes(entry[4..8].try_into().unwrap());
    assert_eq!(checksum, 0xDEADBEEF);
}
