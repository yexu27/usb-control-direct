//! USB 大容量存储会话状态。
//!
//! 一个 StorageSession 拥有一次真实 U 盘到受控主机虚拟介质映射中的运行时资源。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 大容量存储映射会话状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageSessionState {
    New,
    Mounting,
    Scanning,
    BuildingMedia,
    NbdStarting,
    Exposing,
    Mapped,
    Cleaning,
    Closed,
    Failed,
}

/// 标记清理动作的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCleanupMark {
    Started,
    AlreadyCleaning,
    AlreadyClosed,
}

/// USB 大容量存储映射会话。
#[derive(Debug, Clone)]
pub struct StorageSession {
    id: String,
    parent_path: String,
    source_dev: String,
    nbd_index: u32,
    nbd_device: String,
    mount_path: Option<PathBuf>,
    state: StorageSessionState,
    cleanup_reason: Option<String>,
}

impl StorageSession {
    /// 创建新的存储映射会话。
    pub fn new(id: String, parent_path: String, source_dev: String, nbd_index: u32) -> Self {
        Self {
            id,
            parent_path,
            source_dev,
            nbd_index,
            nbd_device: format!("/dev/nbd{nbd_index}"),
            mount_path: None,
            state: StorageSessionState::New,
            cleanup_reason: None,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn parent_path(&self) -> &str {
        &self.parent_path
    }

    pub fn source_dev(&self) -> &str {
        &self.source_dev
    }

    pub fn nbd_index(&self) -> u32 {
        self.nbd_index
    }

    pub fn nbd_device(&self) -> &str {
        &self.nbd_device
    }

    pub fn mount_path(&self) -> Option<&Path> {
        self.mount_path.as_deref()
    }

    pub fn set_mount_path(&mut self, mount_path: PathBuf) {
        self.mount_path = Some(mount_path);
    }

    pub fn state(&self) -> StorageSessionState {
        self.state
    }

    pub fn set_state(&mut self, state: StorageSessionState) {
        self.state = state;
    }

    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        self.cleanup_reason = Some(reason.into());
        self.state = StorageSessionState::Failed;
    }

    pub fn mark_cleaning(&mut self, reason: impl Into<String>) -> SessionCleanupMark {
        match self.state {
            StorageSessionState::Cleaning => SessionCleanupMark::AlreadyCleaning,
            StorageSessionState::Closed => SessionCleanupMark::AlreadyClosed,
            _ => {
                self.cleanup_reason = Some(reason.into());
                self.state = StorageSessionState::Cleaning;
                SessionCleanupMark::Started
            }
        }
    }

    pub fn mark_closed(&mut self) {
        self.state = StorageSessionState::Closed;
    }

    pub fn cleanup_reason(&self) -> Option<&str> {
        self.cleanup_reason.as_deref()
    }
}

/// 活动存储会话注册表。
#[derive(Debug, Default)]
pub struct StorageSessionRegistry {
    sessions: HashMap<String, StorageSession>,
}

impl StorageSessionRegistry {
    pub fn insert(&mut self, session: StorageSession) {
        self.sessions
            .insert(session.parent_path().to_string(), session);
    }

    pub fn get_mut(&mut self, parent_path: &str) -> Option<&mut StorageSession> {
        self.sessions.get_mut(parent_path)
    }

    pub fn remove(&mut self, parent_path: &str) -> Option<StorageSession> {
        self.sessions.remove(parent_path)
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn take_all_for_shutdown(&mut self, reason: &str) -> Vec<StorageSession> {
        self.sessions
            .drain()
            .map(|(_, mut session)| {
                session.mark_cleaning(reason);
                session
            })
            .collect()
    }
}
