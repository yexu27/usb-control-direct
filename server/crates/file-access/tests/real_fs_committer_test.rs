use std::fs;

use file_access::vfs::committer::RealFsCommitter;

#[test]
fn committer_creates_writes_flushes_renames_and_deletes_inside_mount_root() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());

    committer.create_file("/new.txt").unwrap();
    committer.write_at("/new.txt", 0, b"hello").unwrap();
    committer.flush_file("/new.txt").unwrap();
    assert_eq!(fs::read(tmp.path().join("new.txt")).unwrap(), b"hello");

    committer.rename("/new.txt", "/renamed.txt").unwrap();
    assert!(!tmp.path().join("new.txt").exists());
    assert_eq!(fs::read(tmp.path().join("renamed.txt")).unwrap(), b"hello");

    committer.truncate("/renamed.txt", 2).unwrap();
    assert_eq!(fs::read(tmp.path().join("renamed.txt")).unwrap(), b"he");

    committer
        .delete_file_at_real_path(&tmp.path().join("renamed.txt"))
        .unwrap();
    assert!(!tmp.path().join("renamed.txt").exists());
}

#[test]
fn committer_rejects_path_escape() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());
    let err = committer.create_file("/../escape.txt").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn committer_deletes_non_empty_directory_tree_inside_mount_root() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());

    fs::create_dir_all(tmp.path().join("tree/a/b")).unwrap();
    fs::write(tmp.path().join("tree/root.txt"), b"root").unwrap();
    fs::write(tmp.path().join("tree/a/child.txt"), b"child").unwrap();
    fs::write(tmp.path().join("tree/a/b/deep.txt"), b"deep").unwrap();

    committer.delete_dir_at_real_path(&tmp.path().join("tree")).unwrap();

    assert!(!tmp.path().join("tree").exists());
}

#[test]
fn committer_delete_dir_by_real_path_keeps_escape_denied() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());

    let err = committer
        .delete_dir_at_real_path(&tmp.path().join("../outside"))
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn committer_delete_dir_is_idempotent_when_parent_was_removed() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());

    fs::create_dir_all(tmp.path().join("mixed_dir/inner")).unwrap();
    fs::write(tmp.path().join("mixed_dir/inner/mixed.txt"), b"mixed").unwrap();

    committer
        .delete_dir_at_real_path(&tmp.path().join("mixed_dir"))
        .unwrap();
    committer
        .delete_dir_at_real_path(&tmp.path().join("mixed_dir/inner"))
        .unwrap();

    assert!(!tmp.path().join("mixed_dir").exists());
}

#[test]
fn committer_creates_nested_directory_when_parent_is_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());

    committer.create_dir("/after_delete/deep").unwrap();

    assert!(tmp.path().join("after_delete/deep").is_dir());
}
