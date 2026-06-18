//! S04 主引擎 — DeviceMapper trait 实现。
//!
//! 串联文件树构建 → 虚拟卷生成 → NBD 服务 → OTG Gadget 映射。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info};

use log_audit::AuditService;
use storage::Storage;
use storage::model::UsbAuditLogInsert;
use usb_identify::traits::{DeviceMapper, MapContext, MapError, MappedSession, UnmapError};

use crate::exfat::volume::VirtualVolume;
use crate::file_tree::build_file_tree;
use crate::gadget::GadgetManager;
use crate::nbd::{run_request_loop, NbdServer};
use crate::policy::{evaluate_access, load_policy_snapshot};
use crate::types::{AccessDecision, ControlledEntry, PolicySnapshot};
use crate::write_back::WriteBackManager;

/// S04 文件访问控制引擎。
pub struct FileAccessEngine {
    /// 数据库。
    storage: Storage,
    /// 审计服务。
    audit: Arc<AuditService>,
    /// NBD 设备路径。
    nbd_device: String,
    /// 当前映射状态。
    mapped: Mutex<Option<MappingState>>,
}

/// 映射运行状态。
struct MappingState {
    nbd_server: NbdServer,
    gadget: GadgetManager,
}

impl FileAccessEngine {
    /// 创建文件访问控制引擎。
    ///
    /// 参数:
    ///   - storage: 数据库实例。
    ///   - audit: 审计服务。
    ///   - nbd_device: NBD 设备路径（如 /dev/nbd0）。
    pub fn new(storage: Storage, audit: Arc<AuditService>, nbd_device: &str) -> Self {
        FileAccessEngine {
            storage,
            audit,
            nbd_device: nbd_device.to_string(),
            mapped: Mutex::new(None),
        }
    }

    /// 当前是否处于映射状态。
    pub fn is_mapped(&self) -> bool {
        self.mapped.try_lock().map(|s| s.is_some()).unwrap_or(false)
    }

    /// 记录文件阻断审计日志。
    fn log_block_event(&self, file_path: &str, matched_policy: &str) {
        let mut log = UsbAuditLogInsert {
            event_time: 0,
            device_type: None,
            interface_type: None,
            interface_class: None,
            interface_subclass: None,
            interface_protocol: None,
            device_name: None,
            device_sn: None,
            vid: None,
            pid: None,
            event_type: "file_block".to_string(),
            permission: None,
            capacity_bytes: None,
            file_path: Some(file_path.to_string()),
            matched_policy: Some(matched_policy.to_string()),
            result: "blocked".to_string(),
            fail_reason: None,
            detail: None,
        };
        if let Err(e) = self.audit.log_usb_audit(&mut log) {
            error!("写入文件阻断审计日志失败: {}", e);
        }
    }

    /// 递归遍历文件树，记录所有被策略阻断的文件审计日志。
    fn log_block_events_recursive(&self, entries: &[ControlledEntry], snapshot: &PolicySnapshot) {
        for entry in entries {
            let decision = evaluate_access(entry, snapshot);
            if let AccessDecision::Deny(ref reason) = decision {
                self.log_block_event(&entry.virtual_name, reason);
            }
            if entry.is_dir && !entry.children.is_empty() {
                self.log_block_events_recursive(&entry.children, snapshot);
            }
        }
    }
}

impl DeviceMapper for FileAccessEngine {
    fn map_device(
        &self,
        ctx: MapContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<MappedSession, MapError>> + Send + '_>,
    > {
        Box::pin(async move {
            let mount_path = Path::new(&ctx.mount_path);

            // 1. 加载策略快照
            let snapshot = load_policy_snapshot(&self.storage, ctx.permission);
            info!(
                "策略快照: exec={}, blacklist={}, autorun={}, permission={}",
                snapshot.exec_control_enabled,
                snapshot.file_type_blacklist_enabled,
                snapshot.auto_read_control_enabled,
                snapshot.permission
            );

            // 2. 构建受控文件树
            let tree = build_file_tree(mount_path, &ctx.scan_result.infected_files);
            info!("文件树构建完成: {} 个根节点", tree.len());

            // 3. 递归记录阻断事件审计日志
            self.log_block_events_recursive(&tree, &snapshot);

            // 4. 生成虚拟 exFAT 卷
            let volume = VirtualVolume::build(&tree, &snapshot);
            let total_sectors = volume.total_sectors();
            info!("虚拟卷生成完成: {} 扇区", total_sectors);

            // 5. 启动 NBD 服务
            let nbd_device_path = PathBuf::from(&self.nbd_device);
            let mut nbd_server = NbdServer::new(&nbd_device_path);
            let user_fd = nbd_server
                .start(total_sectors)
                .map_err(|e| MapError::NbdFailed(e.to_string()))?;

            // 6. 启动请求处理循环（spawn_blocking 避免阻塞异步运行时）
            let readonly = ctx.permission == 0;
            let mount_path_owned = mount_path.to_path_buf();
            let volume = Arc::new(volume);
            let volume_clone = volume.clone();
            tokio::task::spawn_blocking(move || {
                let mut write_back = WriteBackManager::new(&mount_path_owned, readonly);
                run_request_loop(user_fd, &volume_clone, &mut write_back);
            });

            // 7. 启用 OTG Gadget
            let mut gadget = GadgetManager::new();
            gadget
                .enable(&nbd_device_path, readonly)
                .map_err(|e| MapError::GadgetFailed(e.to_string()))?;

            // 8. 保存映射状态
            let session_id = format!("s04_{}", ctx.mount_path.replace('/', "_"));
            {
                let mut state = self.mapped.lock().await;
                *state = Some(MappingState { nbd_server, gadget });
            }

            info!("S04 映射完成: session={}", session_id);
            Ok(MappedSession {
                id: session_id,
                mount_path: ctx.mount_path,
            })
        })
    }

    fn unmap_device(
        &self,
        session: MappedSession,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), UnmapError>> + Send + '_>,
    > {
        Box::pin(async move {
            let mut state = self.mapped.lock().await;

            if let Some(mut mapping) = state.take() {
                if let Err(e) = mapping.gadget.disable() {
                    error!("禁用 Gadget 失败: {}", e);
                }
                mapping.nbd_server.stop();
                info!("S04 映射已清理: session={}", session.id);
            }

            Ok(())
        })
    }
}
