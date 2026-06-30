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

    committer.delete_file("/renamed.txt").unwrap();
    assert!(!tmp.path().join("renamed.txt").exists());
}

#[test]
fn committer_rejects_path_escape() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());
    let err = committer.create_file("/../escape.txt").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}
