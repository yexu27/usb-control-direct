use std::fs;

use file_access::exfat::layout::{
    CLUSTER_SIZE, EXFAT_SIGNATURE, FAT_END_OF_CHAIN, MIN_VIRTUAL_VOLUME_BYTES,
    PARTITION_OFFSET_SECTORS, SECTOR_SIZE,
};
use file_access::exfat::volume::VirtualVolume;
use file_access::file_tree::build_file_tree;
use file_access::types::{ControlledEntry, PolicySnapshot, SectorContent};

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
    assert!(
        !file_sectors.is_empty(),
        "Should have data sectors for data.bin"
    );

    let first_sector = file_sectors[0];
    let content = volume.read_sector(first_sector);
    match content {
        SectorContent::FileData {
            real_path,
            offset,
            valid_bytes: _,
            blocked,
        } => {
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

#[test]
fn virtual_volume_uses_minimum_capacity_for_tiny_content() {
    let entry = ControlledEntry {
        real_path: "/mnt/usb_raw/123.txt".into(),
        virtual_name: "123.txt".into(),
        is_dir: false,
        file_size: 3,
        is_virus: false,
        exec_type: None,
        extension: "txt".into(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![],
    };

    let volume = VirtualVolume::build_with_capacity(&[entry], &make_snapshot(), 0);

    assert!(volume.total_sectors() * SECTOR_SIZE as u64 >= MIN_VIRTUAL_VOLUME_BYTES);
}

#[test]
fn virtual_volume_uses_source_capacity_when_larger_than_minimum() {
    let source_size = 64 * 1024 * 1024;
    let volume = VirtualVolume::build_with_capacity(&[], &make_snapshot(), source_size);

    assert!(volume.total_sectors() * SECTOR_SIZE as u64 >= source_size);
}

#[test]
fn allocation_bitmap_chain_covers_large_source_capacity() {
    let source_size = 512 * 1024 * 1024;
    let volume = VirtualVolume::build_with_capacity(&[], &make_snapshot(), source_size);
    let layout = volume.layout();

    let root_sector = layout.cluster_to_sector(2);
    let root = match volume.read_sector(root_sector) {
        SectorContent::Metadata(data) => data,
        other => panic!("root directory sector should be metadata, got {other:?}"),
    };

    let bitmap_entry = &root[32..64];
    assert_eq!(bitmap_entry[0], 0x81);
    let bitmap_cluster = u32::from_le_bytes(bitmap_entry[20..24].try_into().unwrap());
    let bitmap_len = u64::from_le_bytes(bitmap_entry[24..32].try_into().unwrap());
    assert_eq!(bitmap_cluster, 3);
    assert!(bitmap_len > CLUSTER_SIZE as u64);

    let mut chain_clusters = 0u64;
    let mut cluster = bitmap_cluster;
    loop {
        chain_clusters += 1;
        let fat_offset = cluster as u64 * 4;
        let fat_sector =
            PARTITION_OFFSET_SECTORS + layout.fat_offset_sectors + fat_offset / SECTOR_SIZE as u64;
        let fat_sector_data = match volume.read_sector(fat_sector) {
            SectorContent::Metadata(data) => data,
            other => panic!("FAT sector should be metadata, got {other:?}"),
        };
        let offset_in_sector = (fat_offset % SECTOR_SIZE as u64) as usize;
        let next = u32::from_le_bytes(
            fat_sector_data[offset_in_sector..offset_in_sector + 4]
                .try_into()
                .unwrap(),
        );
        if next == FAT_END_OF_CHAIN {
            break;
        }
        cluster = next;
    }

    assert!(
        chain_clusters * CLUSTER_SIZE as u64 >= bitmap_len,
        "bitmap chain has {chain_clusters} clusters for {bitmap_len} bytes"
    );
}
