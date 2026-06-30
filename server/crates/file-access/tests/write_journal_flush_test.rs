use std::fs;

use file_access::vfs::committer::RealFsCommitter;
use file_access::vfs::journal::{FileMutation, WriteJournal};

#[test]
fn journal_flush_writes_and_syncs_dirty_file() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), b"hello").unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());
    let mut journal = WriteJournal::new();

    journal.record(FileMutation::Write {
        virtual_path: "/a.txt".to_string(),
        offset: 5,
        data: b" world".to_vec(),
    });
    assert!(journal.is_dirty());

    journal.flush(&committer).unwrap();
    assert!(!journal.is_dirty());
    assert_eq!(fs::read(tmp.path().join("a.txt")).unwrap(), b"hello world");
}

#[test]
fn journal_keeps_dirty_on_flush_error() {
    let tmp = tempfile::tempdir().unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());
    let mut journal = WriteJournal::new();

    journal.record(FileMutation::Write {
        virtual_path: "/missing/missing.txt".to_string(),
        offset: 0,
        data: b"x".to_vec(),
    });

    assert!(journal.flush(&committer).is_err());
    assert!(journal.is_dirty());
}
