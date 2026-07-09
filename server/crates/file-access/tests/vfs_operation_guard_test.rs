use std::collections::HashSet;

use file_access::types::{ExecFileType, PolicySnapshot};
use file_access::vfs::operation_guard::{FsOperation, OperationGuard};

fn snapshot(permission: i32) -> PolicySnapshot {
    PolicySnapshot {
        exec_control_enabled: true,
        file_type_blacklist_enabled: true,
        auto_read_control_enabled: true,
        blacklist_extensions: HashSet::from([".blocked".to_string()]),
        permission,
    }
}

#[test]
fn readonly_denies_mutating_operations() {
    let guard = OperationGuard::new(snapshot(0));
    let err = guard
        .check(&FsOperation::CreateFile {
            virtual_path: "/a.txt".to_string(),
        })
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn blacklist_denies_create_and_rename_target() {
    let guard = OperationGuard::new(snapshot(1));
    assert!(guard
        .check(&FsOperation::CreateFile {
            virtual_path: "/bad.blocked".to_string(),
        })
        .is_err());
    assert!(guard
        .check(&FsOperation::Rename {
            from: "/a.txt".to_string(),
            to: "/bad.blocked".to_string(),
        })
        .is_err());
}

#[test]
fn blacklist_does_not_deny_delete_when_permission_is_read_write() {
    let guard = OperationGuard::new(snapshot(1));

    guard
        .check(&FsOperation::Delete {
            virtual_path: "/bad.blocked".to_string(),
        })
        .unwrap();
}

#[test]
fn readonly_still_denies_delete() {
    let guard = OperationGuard::new(snapshot(0));

    let err = guard
        .check(&FsOperation::Delete {
            virtual_path: "/bad.blocked".to_string(),
        })
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn executable_control_denies_pe_write_commit() {
    let guard = OperationGuard::new(snapshot(1));
    let err = guard
        .check(&FsOperation::WriteExecutable {
            virtual_path: "/setup.exe".to_string(),
            exec_type: ExecFileType::Pe,
        })
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}
