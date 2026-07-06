use std::collections::HashSet;
use std::fs;

use file_access::exfat::layout::PARTITION_OFFSET_SECTORS;
use file_access::exfat::volume::VirtualVolume;
use file_access::file_tree::build_file_tree;
use file_access::types::{PolicySnapshot, SectorContent};

fn make_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn le_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn le_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

#[test]
fn mbr_partition_matches_disk_layout() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let volume = VirtualVolume::build(&tree, &make_snapshot()).unwrap();
    let layout = volume.layout();
    let mbr = match volume.read_sector(0).unwrap() {
        SectorContent::Metadata(data) => data,
        other => panic!("MBR should be metadata, got {other:?}"),
    };

    assert_eq!(le_u32(&mbr, 446 + 8) as u64, PARTITION_OFFSET_SECTORS);
    assert_eq!(le_u32(&mbr, 446 + 12) as u64, layout.volume_length_sectors);
    assert_eq!(
        le_u32(&mbr, 446 + 8) as u64 + le_u32(&mbr, 446 + 12) as u64,
        layout.total_sectors
    );
}

#[test]
fn boot_sector_matches_disk_layout() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let volume = VirtualVolume::build(&tree, &make_snapshot()).unwrap();
    let layout = volume.layout();
    let boot = match volume.read_sector(PARTITION_OFFSET_SECTORS).unwrap() {
        SectorContent::Metadata(data) => data,
        other => panic!("Boot sector should be metadata, got {other:?}"),
    };

    assert_eq!(le_u64(&boot, 64), PARTITION_OFFSET_SECTORS);
    assert_eq!(le_u64(&boot, 72), layout.volume_length_sectors);
    assert_eq!(le_u32(&boot, 80) as u64, layout.fat_offset_sectors);
    assert_eq!(le_u32(&boot, 84) as u64, layout.fat_length_sectors);
    assert_eq!(le_u32(&boot, 88) as u64, layout.cluster_heap_offset_sectors);
    assert_eq!(le_u32(&boot, 92), layout.cluster_count);
}

#[test]
fn volume_rejects_sector_at_total_sectors() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let volume = VirtualVolume::build(&tree, &make_snapshot()).unwrap();

    let err = volume.read_sector(volume.total_sectors()).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn last_legal_sector_is_readable() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let volume = VirtualVolume::build(&tree, &make_snapshot()).unwrap();

    let content = volume.read_sector(volume.total_sectors() - 1).unwrap();
    assert!(matches!(
        content,
        SectorContent::Metadata(_) | SectorContent::FileData { .. } | SectorContent::Zero
    ));
}

#[test]
fn legal_unmapped_sector_returns_zero() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("hello.txt"), b"hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let volume = VirtualVolume::build(&tree, &make_snapshot()).unwrap();

    let gap_sector = 1;
    assert!(gap_sector < volume.total_sectors());
    assert!(matches!(
        volume.read_sector(gap_sector).unwrap(),
        SectorContent::Zero
    ));
}
