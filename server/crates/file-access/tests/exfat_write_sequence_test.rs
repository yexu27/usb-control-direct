use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::layout::{PARTITION_OFFSET_SECTORS, SECTOR_SIZE};
use file_access::exfat::runtime_state::ExfatRuntimeState;
use file_access::exfat::sector_owner::SectorOwner;
use file_access::exfat::transaction::{PendingTransaction, TransactionWrite};
use file_access::types::{ControlledEntry, PolicySnapshot};

fn snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn file(path: PathBuf, name: &str, size: u64) -> ControlledEntry {
    ControlledEntry {
        real_path: path,
        virtual_name: name.to_string(),
        file_size: size,
        is_dir: false,
        is_virus: false,
        exec_type: None,
        extension: name
            .rsplit_once('.')
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .unwrap_or_default(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![],
    }
}

fn dir(path: PathBuf, name: &str, children: Vec<ControlledEntry>) -> ControlledEntry {
    ControlledEntry {
        real_path: path,
        virtual_name: name.to_string(),
        file_size: 0,
        is_dir: true,
        is_virus: false,
        exec_type: None,
        extension: String::new(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children,
    }
}

fn first_sector_matching(
    state: &ExfatRuntimeState,
    predicate: impl Fn(SectorOwner) -> bool,
) -> u64 {
    (0..state.total_sectors())
        .find(|sector| predicate(state.sector_owner(*sector)))
        .expect("expected matching sector")
}

#[test]
fn write_interpreter_records_real_fat_bitmap_directory_and_data_sectors() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("1/2")).unwrap();
    std::fs::write(tmp.path().join("1/2/existing.txt"), b"existing").unwrap();
    let tree = vec![dir(
        tmp.path().join("1"),
        "1",
        vec![dir(
            tmp.path().join("1/2"),
            "2",
            vec![file(tmp.path().join("1/2/existing.txt"), "existing.txt", 8)],
        )],
    )];
    let state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let mut tx = PendingTransaction::new(1);
    let data = vec![0x5a; SECTOR_SIZE as usize];

    let fat_sector = first_sector_matching(&state, |owner| matches!(owner, SectorOwner::Fat));
    let bitmap_sector =
        first_sector_matching(&state, |owner| matches!(owner, SectorOwner::AllocationBitmap));
    let directory_sector = first_sector_matching(&state, |owner| {
        matches!(owner, SectorOwner::RootDirectory | SectorOwner::DirectoryData { .. })
    });
    let file_data_sector =
        first_sector_matching(&state, |owner| matches!(owner, SectorOwner::FileData { .. }));
    let free_sector =
        first_sector_matching(&state, |owner| matches!(owner, SectorOwner::FreeCluster { .. }));

    for sector in [
        fat_sector,
        bitmap_sector,
        directory_sector,
        file_data_sector,
        free_sector,
    ] {
        state.record_write(&mut tx, sector, &data).unwrap();
    }

    assert!(tx
        .writes()
        .iter()
        .any(|write| matches!(write, TransactionWrite::Fat { .. })));
    assert!(tx
        .writes()
        .iter()
        .any(|write| matches!(write, TransactionWrite::Bitmap { .. })));
    assert!(tx
        .writes()
        .iter()
        .any(|write| matches!(write, TransactionWrite::Directory { .. })));
    assert!(tx
        .writes()
        .iter()
        .any(|write| matches!(write, TransactionWrite::FileData { .. })));
    assert!(tx
        .writes()
        .iter()
        .any(|write| matches!(write, TransactionWrite::FreeCluster { .. })));

    let boot_err = state
        .record_write(&mut tx, PARTITION_OFFSET_SECTORS, &data)
        .unwrap_err();
    assert_eq!(boot_err.kind(), std::io::ErrorKind::PermissionDenied);

    let out_of_range_err = state
        .record_write(&mut tx, state.total_sectors() + 1, &data)
        .unwrap_err();
    assert_eq!(out_of_range_err.kind(), std::io::ErrorKind::PermissionDenied);
}
