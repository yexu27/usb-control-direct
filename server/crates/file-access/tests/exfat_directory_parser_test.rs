use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::directory_parser::parse_entry_sets;

#[test]
fn parser_reads_file_entry_set_name_size_and_cluster() {
    let bytes = build_file_entry_set("hello.txt", false, 10, 1234, false);
    let entries = parse_entry_sets(&bytes).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "hello.txt");
    assert_eq!(entries[0].first_cluster, 10);
    assert_eq!(entries[0].data_length, 1234);
    assert!(!entries[0].is_dir);
    assert!(!entries[0].is_deleted);
}

#[test]
fn parser_ignores_zero_padding() {
    let bytes = vec![0u8; 512];
    let entries = parse_entry_sets(&bytes).unwrap();
    assert!(entries.is_empty());
}
