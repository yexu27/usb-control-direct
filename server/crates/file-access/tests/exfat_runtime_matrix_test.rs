use std::collections::HashSet;
use std::path::PathBuf;

use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::directory_parser::parse_entry_sets;
use file_access::exfat::fs::VirtualExfatFs;
use file_access::exfat::layout::SECTOR_SIZE;
use file_access::exfat::runtime_state::ExfatRuntimeState;
use file_access::exfat::sector_owner::SectorOwner;
use file_access::exfat::transaction::{PendingTransaction, ResolveStatus};
use file_access::types::{ControlledEntry, PolicySnapshot};
use file_access::vfs::mutation::FsMutation;

fn snapshot(permission: i32) -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: false,
        file_type_blacklist_enabled: false,
        auto_read_control_enabled: false,
        blacklist_extensions: HashSet::new(),
        permission,
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

fn root_sector(state: &ExfatRuntimeState) -> u64 {
    state.cluster_to_sector(
        state
            .directory_store()
            .directory_clusters("/")
            .unwrap()
            .first()
            .copied()
            .unwrap(),
    )
}

fn first_free(state: &ExfatRuntimeState) -> (u64, u32) {
    for sector in 0..state.total_sectors() {
        if let SectorOwner::FreeCluster { cluster } = state.sector_owner(sector) {
            return (sector, cluster);
        }
    }
    panic!("expected free cluster");
}

fn write_entries(
    state: &ExfatRuntimeState,
    tx: &mut PendingTransaction,
    sector: u64,
    entries: Vec<Vec<u8>>,
) {
    let mut data = vec![0u8; SECTOR_SIZE as usize];
    let mut cursor = 0usize;
    for entry in entries {
        data[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    state.record_write(tx, sector, &data).unwrap();
}

fn root_entry_cluster_for_fs(fs: &VirtualExfatFs, name: &str) -> u32 {
    let data = fs.read_at(fs.root_dir_offset_for_test(), 4096).unwrap();
    parse_entry_sets(&data)
        .unwrap()
        .into_iter()
        .find(|entry| entry.name == name)
        .unwrap()
        .first_cluster
}

fn write_facade_dir_entries(fs: &VirtualExfatFs, dir_cluster: u32, entries: Vec<Vec<u8>>) {
    let mut data = vec![0u8; 4096];
    let mut cursor = 0usize;
    for entry in entries {
        data[cursor..cursor + entry.len()].copy_from_slice(&entry);
        cursor += entry.len();
    }
    fs.write_at(fs.cluster_offset_for_test(dir_cluster), &data)
        .unwrap();
}

fn commit_closed_transaction(state: &mut ExfatRuntimeState, tx: &PendingTransaction) {
    let mutations = match state.try_commit_closed_transaction(tx).unwrap() {
        ResolveStatus::Complete(resolved) => resolved.mutations,
        ResolveStatus::Incomplete(reason) => {
            panic!("expected complete transaction, got incomplete: {reason:?}")
        }
        ResolveStatus::Invalid(err) => {
            panic!("expected complete transaction, got invalid: {err:?}")
        }
    };
    for mutation in mutations {
        state.commit_mutation(mutation).unwrap();
    }
}

#[test]
fn runtime_matrix_commits_create_write_rename_truncate_and_delete_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], snapshot(1), 16 * 1024 * 1024)
            .unwrap();
    let root_sector = root_sector(&state);

    let (_, dir_cluster) = first_free(&state);
    let mut tx = PendingTransaction::new(1);
    write_entries(
        &state,
        &mut tx,
        root_sector,
        vec![build_file_entry_set("matrix", true, dir_cluster, 0, false)],
    );
    commit_closed_transaction(&mut state, &tx);
    assert!(tmp.path().join("matrix").is_dir());

    let matrix_sector = state.cluster_to_sector(dir_cluster);
    let (file_sector, file_cluster) = first_free(&state);
    let mut tx = PendingTransaction::new(2);
    let mut file_data = vec![0u8; SECTOR_SIZE as usize];
    file_data[..11].copy_from_slice(b"hello world");
    state
        .record_write(&mut tx, file_sector, &file_data)
        .unwrap();
    write_entries(
        &state,
        &mut tx,
        matrix_sector,
        vec![build_file_entry_set(
            "file.txt",
            false,
            file_cluster,
            11,
            false,
        )],
    );
    commit_closed_transaction(&mut state, &tx);
    assert_eq!(
        std::fs::read(tmp.path().join("matrix/file.txt")).unwrap(),
        b"hello world"
    );

    let mut tx = PendingTransaction::new(3);
    write_entries(
        &state,
        &mut tx,
        matrix_sector,
        vec![build_file_entry_set(
            "renamed.txt",
            false,
            file_cluster,
            11,
            false,
        )],
    );
    commit_closed_transaction(&mut state, &tx);
    assert!(!tmp.path().join("matrix/file.txt").exists());
    assert!(tmp.path().join("matrix/renamed.txt").is_file());

    let mut tx = PendingTransaction::new(4);
    write_entries(
        &state,
        &mut tx,
        matrix_sector,
        vec![build_file_entry_set(
            "renamed.txt",
            false,
            file_cluster,
            2,
            false,
        )],
    );
    commit_closed_transaction(&mut state, &tx);
    assert_eq!(
        std::fs::read(tmp.path().join("matrix/renamed.txt")).unwrap(),
        b"he"
    );

    let mut tx = PendingTransaction::new(5);
    write_entries(&state, &mut tx, root_sector, Vec::new());
    commit_closed_transaction(&mut state, &tx);
    assert!(!tmp.path().join("matrix").exists());
    assert!(state.lookup_path("/matrix").is_none());
}

#[test]
fn runtime_matrix_rejects_readonly_create_without_partial_state() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &[], snapshot(0), 16 * 1024 * 1024)
            .unwrap();
    let err = state
        .commit_mutation(FsMutation::CreateFile {
            parent: "/".to_string(),
            name: "blocked.txt".to_string(),
            size: 0,
            valid_data_len: 0,
            chain: None,
            data_patches: Vec::new(),
        })
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!tmp.path().join("blocked.txt").exists());
    assert!(state.lookup_path("/blocked.txt").is_none());
}

#[test]
fn runtime_matrix_initial_mapping_preserves_nested_empty_objects() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("1/2/empty")).unwrap();
    std::fs::write(tmp.path().join("1/zero.txt"), []).unwrap();
    let tree = vec![dir(
        tmp.path().join("1"),
        "1",
        vec![
            file(tmp.path().join("1/zero.txt"), "zero.txt", 0),
            dir(
                tmp.path().join("1/2"),
                "2",
                vec![dir(tmp.path().join("1/2/empty"), "empty", vec![])],
            ),
        ],
    )];
    let state =
        ExfatRuntimeState::from_controlled_tree(tmp.path(), &tree, snapshot(1), 16 * 1024 * 1024)
            .unwrap();

    assert!(state.lookup_path("/1/zero.txt").is_some());
    assert!(state.lookup_path("/1/2/empty").unwrap().is_dir());
    assert!(state
        .directory_store()
        .directory_clusters("/1/2/empty")
        .is_some());
}

#[test]
fn facade_write_at_commits_outer_directory_tree_delete() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("codex_baseline_matrix/a/b")).unwrap();
    std::fs::create_dir_all(tmp.path().join("codex_baseline_matrix/empty")).unwrap();
    std::fs::write(
        tmp.path().join("codex_baseline_matrix/a/b/c.txt"),
        b"delete-tree",
    )
    .unwrap();
    let tree = vec![dir(
        tmp.path().join("codex_baseline_matrix"),
        "codex_baseline_matrix",
        vec![
            dir(
                tmp.path().join("codex_baseline_matrix/a"),
                "a",
                vec![dir(
                    tmp.path().join("codex_baseline_matrix/a/b"),
                    "b",
                    vec![file(
                        tmp.path().join("codex_baseline_matrix/a/b/c.txt"),
                        "c.txt",
                        11,
                    )],
                )],
            ),
            dir(
                tmp.path().join("codex_baseline_matrix/empty"),
                "empty",
                vec![],
            ),
        ],
    )];
    let fs = VirtualExfatFs::build(tmp.path(), &tree, snapshot(1), 16 * 1024 * 1024).unwrap();
    assert!(root_entry_cluster_for_fs(&fs, "codex_baseline_matrix") > 0);

    write_facade_dir_entries(&fs, 2, Vec::new());

    assert!(!tmp.path().join("codex_baseline_matrix").exists());
    assert!(fs.lookup_path("/codex_baseline_matrix").is_none());
}
