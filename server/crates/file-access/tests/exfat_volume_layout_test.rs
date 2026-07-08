use std::collections::HashSet;

use file_access::exfat::layout::SECTOR_SIZE;
use file_access::exfat::volume::VirtualVolume;
use file_access::types::{
    blocked_placeholder_bytes, ControlledEntry, ExecFileType, PolicySnapshot, SectorContent,
};

#[test]
fn blocked_file_uses_placeholder_size_and_file_data_sector() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("tool.bin");
    std::fs::write(&real, vec![1u8; 512]).unwrap();
    let entry = ControlledEntry {
        real_path: real,
        virtual_name: "tool.bin".to_string(),
        file_size: 512,
        is_dir: false,
        is_virus: false,
        exec_type: Some(ExecFileType::Elf),
        extension: "bin".to_string(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![],
    };
    let snapshot = PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    };

    let volume = VirtualVolume::build(&[entry], &snapshot).unwrap();
    let sectors = volume.find_file_data_sectors("tool.bin");
    assert!(!sectors.is_empty());
    match volume.read_sector(sectors[0]).unwrap() {
        SectorContent::FileData { valid_bytes, .. } => {
            assert_eq!(
                valid_bytes as usize,
                blocked_placeholder_bytes().len().min(SECTOR_SIZE as usize)
            );
        }
        other => panic!("expected file data sector, got {other:?}"),
    }
}
