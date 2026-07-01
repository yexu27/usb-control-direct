use std::path::PathBuf;

use file_access::vfs::mutation::{FsMutation, NodeKind};
use file_access::vfs::VfsIndex;

#[test]
fn vfs_index_applies_create_empty_file_and_directory() {
    let root = PathBuf::from("/mnt/usb_raw/sdc2");
    let mut index = VfsIndex::from_controlled_tree(&root, &[]).unwrap();

    index
        .apply_mutation(&FsMutation::CreateDir {
            parent: "/".to_string(),
            name: "dir".to_string(),
            chain: None,
        })
        .unwrap();
    index
        .apply_mutation(&FsMutation::CreateFile {
            parent: "/dir".to_string(),
            name: "empty.txt".to_string(),
            size: 0,
            valid_data_len: 0,
            chain: None,
            data_patches: vec![],
        })
        .unwrap();

    assert!(index.lookup_path("/dir").is_some());
    let file = index.lookup_path("/dir/empty.txt").unwrap();
    assert_eq!(index.node(file).unwrap().size, 0);
    assert_eq!(
        index.node(file).unwrap().real_path,
        PathBuf::from("/mnt/usb_raw/sdc2/dir/empty.txt")
    );
}

#[test]
fn vfs_index_applies_rename_and_delete() {
    let root = PathBuf::from("/mnt/usb_raw/sdc2");
    let mut index = VfsIndex::from_controlled_tree(&root, &[]).unwrap();
    index
        .apply_mutation(&FsMutation::CreateDir {
            parent: "/".to_string(),
            name: "old".to_string(),
            chain: None,
        })
        .unwrap();
    index
        .apply_mutation(&FsMutation::CreateFile {
            parent: "/old".to_string(),
            name: "child.txt".to_string(),
            size: 5,
            valid_data_len: 5,
            chain: None,
            data_patches: vec![],
        })
        .unwrap();

    index
        .apply_mutation(&FsMutation::Rename {
            from: "/old".to_string(),
            to: "/new".to_string(),
            kind: NodeKind::Directory,
        })
        .unwrap();
    assert!(index.lookup_path("/old").is_none());
    assert!(index.lookup_path("/old/child.txt").is_none());
    assert!(index.lookup_path("/new").is_some());
    assert!(index.lookup_path("/new/child.txt").is_some());

    index
        .apply_mutation(&FsMutation::Delete {
            virtual_path: "/new".to_string(),
            kind: NodeKind::Directory,
        })
        .unwrap();
    assert!(index.lookup_path("/new").is_none());
    assert!(index.lookup_path("/new/child.txt").is_none());
}

#[test]
fn vfs_index_applies_truncate() {
    let root = PathBuf::from("/mnt/usb_raw/sdc2");
    let mut index = VfsIndex::from_controlled_tree(&root, &[]).unwrap();
    index
        .apply_mutation(&FsMutation::CreateFile {
            parent: "/".to_string(),
            name: "file.txt".to_string(),
            size: 10,
            valid_data_len: 10,
            chain: None,
            data_patches: vec![],
        })
        .unwrap();

    index
        .apply_mutation(&FsMutation::Truncate {
            virtual_path: "/file.txt".to_string(),
            len: 3,
        })
        .unwrap();

    let file = index.lookup_path("/file.txt").unwrap();
    assert_eq!(index.node(file).unwrap().size, 3);
}
