use file_access::exfat::diff::diff_directory_snapshots;
use file_access::exfat::dir_entry::build_file_entry_set;
use file_access::exfat::dir_snapshot::DirectorySnapshot;
use file_access::vfs::mutation::{ClusterChain, FsMutation, NodeKind};

fn snapshot(path: &str, entries: Vec<Vec<u8>>) -> DirectorySnapshot {
    let mut data = Vec::new();
    for entry in entries {
        data.extend(entry);
    }
    data.resize(4096, 0);
    DirectorySnapshot::parse(path, &data).unwrap()
}

#[test]
fn diff_detects_empty_file_and_empty_dir_create() {
    let old = snapshot("/", vec![]);
    let new = snapshot(
        "/",
        vec![
            build_file_entry_set("empty.txt", false, 0, 0, false),
            build_file_entry_set("empty_dir", true, 100, 0, false),
        ],
    );

    let mutations = diff_directory_snapshots(&old, &new).unwrap();
    assert!(mutations.contains(&FsMutation::CreateFile {
        parent: "/".to_string(),
        name: "empty.txt".to_string(),
        size: 0,
        valid_data_len: 0,
        chain: None,
        data_patches: vec![],
    }));
    assert!(mutations.contains(&FsMutation::CreateDir {
        parent: "/".to_string(),
        name: "empty_dir".to_string(),
        chain: Some(ClusterChain {
            first_cluster: 100,
            clusters: vec![100],
        }),
    }));
}

#[test]
fn diff_detects_delete_and_rename_by_entry_identity() {
    let old = snapshot(
        "/",
        vec![
            build_file_entry_set("old.txt", false, 88, 5, false),
            build_file_entry_set("gone.txt", false, 89, 7, false),
        ],
    );
    let new = snapshot(
        "/",
        vec![build_file_entry_set("new.txt", false, 88, 5, false)],
    );

    let mutations = diff_directory_snapshots(&old, &new).unwrap();
    assert!(mutations.contains(&FsMutation::Rename {
        from: "/old.txt".to_string(),
        to: "/new.txt".to_string(),
        kind: NodeKind::File,
    }));
    assert!(mutations.contains(&FsMutation::Delete {
        virtual_path: "/gone.txt".to_string(),
        kind: NodeKind::File,
    }));
}

#[test]
fn diff_detects_same_name_file_truncate() {
    let old = snapshot(
        "/",
        vec![build_file_entry_set("file.txt", false, 88, 5, false)],
    );
    let new = snapshot(
        "/",
        vec![build_file_entry_set("file.txt", false, 88, 2, false)],
    );

    let mutations = diff_directory_snapshots(&old, &new).unwrap();
    assert!(mutations.contains(&FsMutation::Truncate {
        virtual_path: "/file.txt".to_string(),
        len: 2,
    }));
}

#[test]
fn diff_detects_rename_and_truncate_by_same_entry_identity() {
    let old = snapshot(
        "/",
        vec![build_file_entry_set("old.txt", false, 88, 8, false)],
    );
    let new = snapshot(
        "/",
        vec![build_file_entry_set("new.txt", false, 88, 4, false)],
    );

    let mutations = diff_directory_snapshots(&old, &new).unwrap();
    assert!(mutations.contains(&FsMutation::Rename {
        from: "/old.txt".to_string(),
        to: "/new.txt".to_string(),
        kind: NodeKind::File,
    }));
    assert!(mutations.contains(&FsMutation::Truncate {
        virtual_path: "/new.txt".to_string(),
        len: 4,
    }));
}

#[test]
fn diff_treats_reused_cluster_at_different_entry_offset_as_delete_and_create() {
    let old = snapshot(
        "/",
        vec![
            build_file_entry_set("old.txt", false, 88, 8, false),
            build_file_entry_set("keep.txt", false, 99, 4, false),
        ],
    );
    let new = snapshot(
        "/",
        vec![
            build_file_entry_set("keep.txt", false, 99, 4, false),
            build_file_entry_set("new.txt", false, 88, 4, false),
        ],
    );

    let mutations = diff_directory_snapshots(&old, &new).unwrap();
    assert!(!mutations.iter().any(|mutation| {
        matches!(
            mutation,
            FsMutation::Rename {
                from,
                to,
                kind: NodeKind::File,
            } if from == "/old.txt" && to == "/new.txt"
        )
    }));
    assert!(mutations.contains(&FsMutation::Delete {
        virtual_path: "/old.txt".to_string(),
        kind: NodeKind::File,
    }));
    assert!(mutations.contains(&FsMutation::CreateFile {
        parent: "/".to_string(),
        name: "new.txt".to_string(),
        size: 4,
        valid_data_len: 4,
        chain: Some(ClusterChain {
            first_cluster: 88,
            clusters: vec![88],
        }),
        data_patches: vec![],
    }));
}
