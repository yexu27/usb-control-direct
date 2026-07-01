use file_access::exfat::dir_entry::{build_file_entry_set, ATTR_ARCHIVE, ATTR_DIRECTORY};
use file_access::exfat::dir_snapshot::DirectorySnapshot;
use file_access::exfat::directory_parser::parse_entry_sets;

#[test]
fn parser_preserves_zero_length_file_and_directory() {
    let mut data = Vec::new();
    data.extend(build_file_entry_set("empty.txt", false, 0, 0, false));
    data.extend(build_file_entry_set("empty_dir", true, 100, 0, false));
    data.resize(4096, 0);

    let entries = parse_entry_sets(&data).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "empty.txt");
    assert!(!entries[0].is_dir);
    assert_eq!(entries[0].data_length, 0);
    assert_eq!(entries[0].valid_data_length, 0);
    assert_eq!(entries[0].attributes, ATTR_ARCHIVE);
    assert_eq!(entries[1].name, "empty_dir");
    assert!(entries[1].is_dir);
    assert_eq!(entries[1].attributes, ATTR_DIRECTORY);
}

#[test]
fn parser_preserves_entry_offsets_for_diff() {
    let mut data = Vec::new();
    data.extend(build_file_entry_set("a.txt", false, 10, 1, false));
    data.extend(build_file_entry_set("b.txt", false, 11, 2, false));
    data.resize(4096, 0);

    let entries = parse_entry_sets(&data).unwrap();
    assert!(entries[0].entry_offset < entries[1].entry_offset);
    assert_eq!(entries[0].set_len, 96);
}

#[test]
fn directory_snapshot_indexes_entries_by_name_and_offset() {
    let mut data = Vec::new();
    data.extend(build_file_entry_set("a.txt", false, 10, 1, false));
    data.extend(build_file_entry_set("b.txt", false, 11, 2, false));
    data.resize(4096, 0);

    let snapshot = DirectorySnapshot::parse("/", &data).unwrap();
    let a = snapshot.get_by_name("a.txt").unwrap();
    assert_eq!(a.first_cluster, 10);
    assert_eq!(
        snapshot.get_by_offset(a.entry_offset).unwrap().name,
        "a.txt"
    );
}
