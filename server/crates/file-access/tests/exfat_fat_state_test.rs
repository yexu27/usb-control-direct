use file_access::exfat::fat::FatBuilder;
use file_access::exfat::layout::{FAT_END_OF_CHAIN, FAT_MEDIA_TYPE};

#[test]
fn fat_builder_writes_reserved_entries() {
    let builder = FatBuilder::new(10);
    let data = builder.build(1);
    let e0 = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let e1 = u32::from_le_bytes(data[4..8].try_into().unwrap());

    assert_eq!(e0, FAT_MEDIA_TYPE);
    assert_eq!(e1, FAT_END_OF_CHAIN);
}

#[test]
fn fat_builder_writes_single_and_chained_clusters() {
    let mut builder = FatBuilder::new(10);
    builder.set_single(2);
    builder.set_chain(3, 3);
    let data = builder.build(1);

    assert_eq!(
        u32::from_le_bytes(data[8..12].try_into().unwrap()),
        FAT_END_OF_CHAIN
    );
    assert_eq!(u32::from_le_bytes(data[12..16].try_into().unwrap()), 4);
    assert_eq!(u32::from_le_bytes(data[16..20].try_into().unwrap()), 5);
    assert_eq!(
        u32::from_le_bytes(data[20..24].try_into().unwrap()),
        FAT_END_OF_CHAIN
    );
}
