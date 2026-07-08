use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::layout::{PARTITION_OFFSET_SECTORS, SECTOR_SIZE};
use file_access::exfat::runtime_state::ExfatRuntimeState;
use file_access::exfat::sector_owner::SectorOwner;
use file_access::exfat::transaction::{PendingTransaction, ResolveStatus, TransactionWrite};
use file_access::types::{ControlledEntry, PolicySnapshot};
use file_access::vfs::mutation::FsMutation;

fn snapshot() -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission: 1,
    }
}

fn write_directory_entry(
    state: &ExfatRuntimeState,
    tx: &mut PendingTransaction,
    directory_sector: u64,
    entry: Vec<u8>,
) {
    write_directory_entries(state, tx, directory_sector, vec![entry]);
}

fn write_directory_entries(
    state: &ExfatRuntimeState,
    tx: &mut PendingTransaction,
    directory_sector: u64,
    entries: Vec<Vec<u8>>,
) {
    let mut sector = vec![0u8; SECTOR_SIZE as usize];
    let mut cursor = 0usize;
    for entry in entries {
        sector[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    state.record_write(tx, directory_sector, &sector).unwrap();
}

fn write_empty_directory_sector(
    state: &ExfatRuntimeState,
    tx: &mut PendingTransaction,
    directory_sector: u64,
) {
    state
        .record_write(tx, directory_sector, &vec![0u8; SECTOR_SIZE as usize])
        .unwrap();
}

fn complete_mutations(status: ResolveStatus) -> Vec<FsMutation> {
    match status {
        ResolveStatus::Complete(resolved) => resolved.mutations,
        ResolveStatus::Incomplete(reason) => {
            panic!("expected complete transaction, got incomplete: {reason:?}")
        }
        ResolveStatus::Invalid(err) => {
            panic!("expected complete transaction, got invalid: {err:?}")
        }
    }
}

fn commit_closed_transaction(
    state: &mut ExfatRuntimeState,
    tx: &PendingTransaction,
) -> Vec<FsMutation> {
    let mutations = complete_mutations(state.try_commit_closed_transaction(tx).unwrap());
    for mutation in mutations.clone() {
        state.commit_mutation(mutation).unwrap();
    }
    mutations
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
    let bitmap_sector = first_sector_matching(&state, |owner| {
        matches!(owner, SectorOwner::AllocationBitmap)
    });
    let directory_sector = first_sector_matching(&state, |owner| {
        matches!(
            owner,
            SectorOwner::RootDirectory | SectorOwner::DirectoryData { .. }
        )
    });
    let file_data_sector = first_sector_matching(&state, |owner| {
        matches!(owner, SectorOwner::FileData { .. })
    });
    let free_sector = first_sector_matching(&state, |owner| {
        matches!(owner, SectorOwner::FreeCluster { .. })
    });

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
    assert_eq!(
        out_of_range_err.kind(),
        std::io::ErrorKind::PermissionDenied
    );
}

#[test]
fn write_at_sequence_creates_empty_dir_and_zero_file_without_flush() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let root_sector = state.cluster_to_sector(
        state
            .directory_store()
            .directory_clusters("/")
            .unwrap()
            .first()
            .copied()
            .unwrap(),
    );
    let free_cluster = match state.sector_owner(first_sector_matching(&state, |owner| {
        matches!(owner, SectorOwner::FreeCluster { .. })
    })) {
        SectorOwner::FreeCluster { cluster } => cluster,
        other => panic!("expected free cluster, got {other:?}"),
    };

    let mut tx = PendingTransaction::new(2);
    write_directory_entry(
        &state,
        &mut tx,
        root_sector,
        build_file_entry_set("closed_empty_dir", true, free_cluster, 0, false),
    );
    let mutations = commit_closed_transaction(&mut state, &tx);
    assert_eq!(mutations.len(), 1);
    assert!(tmp.path().join("closed_empty_dir").is_dir());
    assert!(state.lookup_path("/closed_empty_dir").unwrap().is_dir());

    let mut tx = PendingTransaction::new(3);
    write_directory_entries(
        &state,
        &mut tx,
        root_sector,
        vec![
            build_file_entry_set("closed_empty_dir", true, free_cluster, 0, false),
            build_file_entry_set("closed_zero.txt", false, 0, 0, false),
        ],
    );
    let mutations = commit_closed_transaction(&mut state, &tx);
    assert_eq!(mutations.len(), 1);
    assert_eq!(
        std::fs::metadata(tmp.path().join("closed_zero.txt"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(state.lookup_path("/closed_zero.txt").unwrap().size, 0);
}

#[test]
fn write_at_sequence_creates_file_and_commits_data_to_real_usb() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let root_sector = state.cluster_to_sector(
        state
            .directory_store()
            .directory_clusters("/")
            .unwrap()
            .first()
            .copied()
            .unwrap(),
    );
    let free_sector = first_sector_matching(&state, |owner| {
        matches!(owner, SectorOwner::FreeCluster { .. })
    });
    let file_cluster = match state.sector_owner(free_sector) {
        SectorOwner::FreeCluster { cluster } => cluster,
        other => panic!("expected free cluster, got {other:?}"),
    };

    let mut tx = PendingTransaction::new(4);
    let mut data_sector = vec![0u8; SECTOR_SIZE as usize];
    data_sector[..11].copy_from_slice(b"hello world");
    state
        .record_write(&mut tx, free_sector, &data_sector)
        .unwrap();
    write_directory_entry(
        &state,
        &mut tx,
        root_sector,
        build_file_entry_set("created.txt", false, file_cluster, 11, false),
    );

    let mutations = commit_closed_transaction(&mut state, &tx);
    assert_eq!(mutations.len(), 1);
    assert_eq!(
        std::fs::read(tmp.path().join("created.txt")).unwrap(),
        b"hello world"
    );
    assert_eq!(state.lookup_path("/created.txt").unwrap().size, 11);
}

#[test]
fn write_at_sequence_deletes_outer_directory_tree() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("matrix/a/b")).unwrap();
    std::fs::write(tmp.path().join("matrix/a/b/data.txt"), b"data").unwrap();
    let tree = vec![dir(
        tmp.path().join("matrix"),
        "matrix",
        vec![dir(
            tmp.path().join("matrix/a"),
            "a",
            vec![dir(
                tmp.path().join("matrix/a/b"),
                "b",
                vec![file(tmp.path().join("matrix/a/b/data.txt"), "data.txt", 4)],
            )],
        )],
    )];
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let root_sector = state.cluster_to_sector(
        state
            .directory_store()
            .directory_clusters("/")
            .unwrap()
            .first()
            .copied()
            .unwrap(),
    );

    let mut tx = PendingTransaction::new(5);
    write_empty_directory_sector(&state, &mut tx, root_sector);
    let mutations = commit_closed_transaction(&mut state, &tx);

    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            file_access::vfs::mutation::FsMutation::Delete { virtual_path, .. }
                if virtual_path == "/matrix"
        )
    }));
    assert!(!tmp.path().join("matrix").exists());
    assert!(state.lookup_path("/matrix").is_none());
}

#[test]
fn write_at_sequence_renames_file_from_directory_entry_change() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("old.txt"), b"old-data").unwrap();
    let tree = vec![file(tmp.path().join("old.txt"), "old.txt", 8)];
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let root_sector = state.cluster_to_sector(
        state
            .directory_store()
            .directory_clusters("/")
            .unwrap()
            .first()
            .copied()
            .unwrap(),
    );

    let old_cluster = state
        .lookup_path("/old.txt")
        .unwrap()
        .first_cluster
        .unwrap();
    let mut tx = PendingTransaction::new(6);
    write_directory_entry(
        &state,
        &mut tx,
        root_sector,
        build_file_entry_set("renamed.txt", false, old_cluster, 8, false),
    );
    let mutations = commit_closed_transaction(&mut state, &tx);

    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            file_access::vfs::mutation::FsMutation::Rename { from, to, .. }
                if from == "/old.txt" && to == "/renamed.txt"
        )
    }));
    assert!(!tmp.path().join("old.txt").exists());
    assert_eq!(
        std::fs::read(tmp.path().join("renamed.txt")).unwrap(),
        b"old-data"
    );
    assert!(state.lookup_path("/old.txt").is_none());
    assert!(state.lookup_path("/renamed.txt").is_some());
}

#[test]
fn write_at_sequence_truncates_existing_file_from_directory_entry_length() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("data.txt"), b"12345678").unwrap();
    let tree = vec![file(tmp.path().join("data.txt"), "data.txt", 8)];
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &tree, snapshot(), 16 * 1024 * 1024)
            .unwrap();
    let root_sector = state.cluster_to_sector(
        state
            .directory_store()
            .directory_clusters("/")
            .unwrap()
            .first()
            .copied()
            .unwrap(),
    );

    let data_cluster = state
        .lookup_path("/data.txt")
        .unwrap()
        .first_cluster
        .unwrap();
    let mut tx = PendingTransaction::new(7);
    write_directory_entry(
        &state,
        &mut tx,
        root_sector,
        build_file_entry_set("data.txt", false, data_cluster, 2, false),
    );
    let mutations = commit_closed_transaction(&mut state, &tx);

    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            file_access::vfs::mutation::FsMutation::Truncate { virtual_path, len }
                if virtual_path == "/data.txt" && *len == 2
        )
    }));
    assert_eq!(std::fs::read(tmp.path().join("data.txt")).unwrap(), b"12");
    assert_eq!(state.lookup_path("/data.txt").unwrap().size, 2);
}
