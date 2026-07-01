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

#[test]
fn journal_does_not_sync_file_deleted_in_same_flush() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("single_delete.txt"), b"single").unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());
    let mut journal = WriteJournal::new();

    journal.record(FileMutation::Write {
        virtual_path: "/single_delete.txt".to_string(),
        offset: 0,
        data: b"changed".to_vec(),
    });
    journal.record(FileMutation::DeleteFile {
        virtual_path: "/single_delete.txt".to_string(),
    });

    journal.flush(&committer).unwrap();

    assert!(!tmp.path().join("single_delete.txt").exists());
}

#[test]
fn journal_does_not_sync_file_under_deleted_directory() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("mixed_dir/inner")).unwrap();
    fs::write(tmp.path().join("mixed_dir/inner/mixed.txt"), b"mixed").unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());
    let mut journal = WriteJournal::new();

    journal.record(FileMutation::Write {
        virtual_path: "/mixed_dir/inner/mixed.txt".to_string(),
        offset: 0,
        data: b"changed".to_vec(),
    });
    journal.record(FileMutation::DeleteDir {
        virtual_path: "/mixed_dir".to_string(),
    });
    journal.record(FileMutation::DeleteDir {
        virtual_path: "/mixed_dir/inner".to_string(),
    });

    journal.flush(&committer).unwrap();

    assert!(!tmp.path().join("mixed_dir").exists());
}

#[test]
fn journal_syncs_renamed_dirty_file_at_final_path() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("source.txt"), b"source").unwrap();
    let committer = RealFsCommitter::new(tmp.path().to_path_buf());
    let mut journal = WriteJournal::new();

    journal.record(FileMutation::Write {
        virtual_path: "/source.txt".to_string(),
        offset: 0,
        data: b"target".to_vec(),
    });
    journal.record(FileMutation::Rename {
        from: "/source.txt".to_string(),
        to: "/target.txt".to_string(),
    });

    journal.flush(&committer).unwrap();

    assert!(!tmp.path().join("source.txt").exists());
    assert_eq!(fs::read(tmp.path().join("target.txt")).unwrap(), b"target");
}
