use file_access::exfat::bitmap::generate_bitmap;

#[test]
fn allocation_bitmap_marks_no_clusters_when_empty() {
    let bitmap = generate_bitmap(16, 0);

    assert_eq!(bitmap.len(), 2);
    assert_eq!(bitmap[0], 0x00);
}

#[test]
fn allocation_bitmap_marks_full_and_partial_bytes() {
    let first_eight = generate_bitmap(16, 8);
    assert_eq!(first_eight[0], 0xff);
    assert_eq!(first_eight[1], 0x00);

    let partial = generate_bitmap(16, 3);
    assert_eq!(partial[0], 0x07);
}
