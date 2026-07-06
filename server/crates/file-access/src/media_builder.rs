//! 受控虚拟介质构建器。
//!
//! 本模块负责把真实挂载目录、扫描结果和策略快照转换为 `VirtualExfatFs`。
//! 它不启动 NBD，不操作 gadget，也不处理 USB 插拔生命周期。

use std::path::Path;
use std::sync::Arc;

use storage::Storage;
use tracing::{debug, info, warn};
use usb_identify::traits::ScanResult;

use crate::exfat::fs::VirtualExfatFs;
use crate::file_tree::build_file_tree;
use crate::policy::{evaluate_access, load_policy_snapshot};
use crate::types::{AccessDecision, ControlledEntry, PolicySnapshot};

/// 受控虚拟介质构建器。
pub struct VirtualMediaBuilder {
    storage: Arc<Storage>,
}

impl VirtualMediaBuilder {
    /// 创建构建器。
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// 基于真实挂载目录构建受控虚拟 exFAT 介质。
    pub fn build(
        &self,
        mount_path: &Path,
        scan_result: ScanResult,
        permission: i32,
        source_size_bytes: u64,
    ) -> Result<VirtualExfatFs, std::io::Error> {
        debug!(mount = %mount_path.display(), "开始加载文件访问策略快照");
        let snapshot = load_policy_snapshot(&self.storage, permission);
        info!(
            exec = snapshot.exec_control_enabled,
            blacklist = snapshot.file_type_blacklist_enabled,
            autorun = snapshot.auto_read_control_enabled,
            permission = snapshot.permission,
            "策略快照加载完成"
        );

        debug!(mount = %mount_path.display(), "开始构建受控文件树");
        let tree = build_file_tree(mount_path, &scan_result.infected_files);
        info!(root_count = tree.len(), "受控文件树构建完成");

        log_blocked_entries(&tree, &snapshot);

        VirtualExfatFs::build(mount_path, &tree, snapshot, source_size_bytes)
    }
}

fn log_blocked_entries(entries: &[ControlledEntry], snapshot: &PolicySnapshot) {
    for entry in entries {
        let decision = evaluate_access(entry, snapshot);
        if let AccessDecision::Deny(ref reason) = decision {
            warn!(file = %entry.virtual_name, reason = %reason, "文件被策略阻断");
        }
        if entry.is_dir && !entry.children.is_empty() {
            log_blocked_entries(&entry.children, snapshot);
        }
    }
}
