use std::fs;
use std::path::PathBuf;

use file_access::exfat::layout::{SECTOR_SIZE, PARTITION_OFFSET_SECTORS, EXFAT_SIGNATURE};
use file_access::exfat::volume::VirtualVolume;
use file_access::file_tree::build_file_tree;
use file_access::policy::{evaluate_access, load_policy_snapshot};
use file_access::types::{AccessDecision, PolicySnapshot, SectorContent};

fn make_snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: std::collections::HashSet::new(),
        permission: 1,
    }
}

#[test]
fn volume_mbr_has_boot_signature() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("test.txt"), b"Hello world").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let snapshot = make_snapshot();
    let volume = VirtualVolume::build(&tree, &snapshot);

    let content = volume.read_sector(0);
    match content {
        SectorContent::Metadata(data) => {
            assert_eq!(data.len(), SECTOR_SIZE as usize);
            assert_eq!(data[510], 0x55);
            assert_eq!(data[511], 0xAA);
        }
        _ => panic!("MBR should be Metadata"),
    }
}

#[test]
fn volume_boot_sector_has_exfat_signature() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("test.txt"), b"Hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let snapshot = make_snapshot();
    let volume = VirtualVolume::build(&tree, &snapshot);

    let content = volume.read_sector(PARTITION_OFFSET_SECTORS);
    match content {
        SectorContent::Metadata(data) => {
            assert_eq!(&data[3..11], EXFAT_SIGNATURE);
        }
        _ => panic!("Boot sector should be Metadata"),
    }
}

#[test]
fn volume_gap_sectors_are_zero() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("test.txt"), b"Hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let snapshot = make_snapshot();
    let volume = VirtualVolume::build(&tree, &snapshot);

    let content = volume.read_sector(1);
    assert!(matches!(content, SectorContent::Zero));
}

#[test]
fn volume_file_data_maps_to_real_path() {
    let tmp = tempfile::tempdir().unwrap();
    let test_data = b"This is test file content for volume test";
    fs::write(tmp.path().join("data.bin"), test_data).unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let snapshot = make_snapshot();
    let volume = VirtualVolume::build(&tree, &snapshot);

    // 查找 data.bin 的数据扇区
    let file_sectors = volume.find_file_data_sectors("data.bin");
    assert!(!file_sectors.is_empty(), "Should have data sectors for data.bin");

    let first_sector = file_sectors[0];
    let content = volume.read_sector(first_sector);
    match content {
        SectorContent::FileData { real_path, offset, valid_bytes, blocked } => {
            assert_eq!(real_path, tmp.path().join("data.bin"));
            assert_eq!(offset, 0);
            assert!(!blocked);
        }
        _ => panic!("File data sector should be FileData"),
    }
}

#[test]
fn volume_total_sectors() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("test.txt"), b"Hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let snapshot = make_snapshot();
    let volume = VirtualVolume::build(&tree, &snapshot);

    assert!(volume.total_sectors() > PARTITION_OFFSET_SECTORS);
}

#[test]
fn volume_beyond_total_returns_zero() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("test.txt"), b"Hello").unwrap();

    let tree = build_file_tree(tmp.path(), &[]);
    let snapshot = make_snapshot();
    let volume = VirtualVolume::build(&tree, &snapshot);

    let content = volume.read_sector(volume.total_sectors() + 100);
    assert!(matches!(content, SectorContent::Zero));
}
