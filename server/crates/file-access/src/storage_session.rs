//! 大容量存储受控会话管理器。
//!
//! 阶段 1 中，该模块成为 storage 链路的生命周期 owner。
//! 它临时复用现有 `DeviceMapper` 完成虚拟介质、NBD 和 gadget 映射。
//! 阶段 2 会继续拆分 `DeviceMapper` 内部职责。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{watch, Mutex};
use tracing::{info, warn};
use usb_identify::mount::{
    dev_name_from_path, mount_partition, mount_path_for, mount_target_exists, MountOperations,
};
use usb_identify::traits::{
    AuthorizedStorageDevice, DeviceMapper, MapContext, MappedSession, Scanner,
    StorageSessionController, StorageSessionError, StorageSessionHandle,
};

const DEFAULT_NBD_POOL_SIZE: u32 = 4;

/// Storage session 使用的挂载操作抽象。
pub trait StorageSessionMountOps: Send + Sync {
    fn mount_partition(
        &self,
        dev_path: &str,
        mount_point: &str,
        read_only: bool,
    ) -> Result<(), StorageSessionError>;

    fn umount(&self, mount_point: &str) -> Result<(), StorageSessionError>;

    fn mount_target_exists(&self, mount_point: &str) -> Result<bool, StorageSessionError>;
}

/// 真实 RK 环境挂载操作。
#[derive(Debug, Default)]
pub struct RealStorageSessionMountOps;

impl StorageSessionMountOps for RealStorageSessionMountOps {
    fn mount_partition(
        &self,
        dev_path: &str,
        mount_point: &str,
        read_only: bool,
    ) -> Result<(), StorageSessionError> {
        mount_partition(dev_path, mount_point, read_only)
            .map_err(|e| StorageSessionError::Failed(e.to_string()))
    }

    fn umount(&self, mount_point: &str) -> Result<(), StorageSessionError> {
        usb_identify::mount::RealMountOps
            .umount(mount_point)
            .map_err(|e| StorageSessionError::Failed(e.to_string()))
    }

    fn mount_target_exists(&self, mount_point: &str) -> Result<bool, StorageSessionError> {
        mount_target_exists(mount_point).map_err(|e| StorageSessionError::Failed(e.to_string()))
    }
}

/// NBD 设备号池。
#[derive(Debug)]
pub struct NbdIndexPool {
    available: Vec<u32>,
    in_use: HashSet<u32>,
}

impl NbdIndexPool {
    pub fn new(pool_size: u32) -> Self {
        Self {
            available: (0..pool_size).collect(),
            in_use: HashSet::new(),
        }
    }

    pub fn acquire(&mut self) -> Option<u32> {
        let idx = self.available.pop()?;
        self.in_use.insert(idx);
        Some(idx)
    }

    pub fn release(&mut self, idx: u32) {
        self.in_use.remove(&idx);
        if !self.available.contains(&idx) {
            self.available.push(idx);
        }
    }
}

impl Default for NbdIndexPool {
    fn default() -> Self {
        Self::new(DEFAULT_NBD_POOL_SIZE)
    }
}

struct ActiveStorageSession {
    device: AuthorizedStorageDevice,
    nbd_index: u32,
    mount_path: Option<PathBuf>,
    mapped_session: Option<MappedSession>,
    cancel_tx: watch::Sender<bool>,
}

/// 大容量存储受控会话管理器。
pub struct StorageSessionManager {
    scanner: Arc<dyn Scanner>,
    mapper: Arc<dyn DeviceMapper>,
    mount_ops: Arc<dyn StorageSessionMountOps>,
    sessions: Arc<Mutex<HashMap<String, ActiveStorageSession>>>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
}

impl StorageSessionManager {
    /// 创建生产环境 manager。
    pub fn new(scanner: Arc<dyn Scanner>, mapper: Arc<dyn DeviceMapper>) -> Self {
        Self::with_mount_ops(
            scanner,
            mapper,
            Arc::new(RealStorageSessionMountOps),
            NbdIndexPool::default(),
        )
    }

    /// 创建可注入挂载和 NBD 池的 manager，供测试使用。
    pub fn with_mount_ops(
        scanner: Arc<dyn Scanner>,
        mapper: Arc<dyn DeviceMapper>,
        mount_ops: Arc<dyn StorageSessionMountOps>,
        nbd_pool: NbdIndexPool,
    ) -> Self {
        Self {
            scanner,
            mapper,
            mount_ops,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            nbd_pool: Arc::new(Mutex::new(nbd_pool)),
        }
    }

    fn session_id(device: &AuthorizedStorageDevice) -> String {
        format!("storage_{}", device.parent_path.replace('/', "_"))
    }
}

impl StorageSessionController for StorageSessionManager {
    fn has_active_storage(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move { !self.sessions.lock().await.is_empty() })
    }

    fn start_authorized_storage(
        &self,
        device: AuthorizedStorageDevice,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<StorageSessionHandle, StorageSessionError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            if device.dev_path.is_empty() {
                return Err(StorageSessionError::Rejected("dev_path 为空".into()));
            }

            let nbd_index = self
                .nbd_pool
                .lock()
                .await
                .acquire()
                .ok_or_else(|| StorageSessionError::Rejected("NBD 设备号池耗尽".into()))?;

            let session_id = Self::session_id(&device);
            let parent_path = device.parent_path.clone();
            let (cancel_tx, cancel_rx) = watch::channel(false);
            let active = ActiveStorageSession {
                device: device.clone(),
                nbd_index,
                mount_path: None,
                mapped_session: None,
                cancel_tx,
            };

            self.sessions
                .lock()
                .await
                .insert(parent_path.clone(), active);

            let scanner = Arc::clone(&self.scanner);
            let mapper = Arc::clone(&self.mapper);
            let mount_ops = Arc::clone(&self.mount_ops);
            let sessions = Arc::clone(&self.sessions);
            let nbd_pool = Arc::clone(&self.nbd_pool);

            tokio::spawn(async move {
                run_storage_pipeline(
                    device, nbd_index, cancel_rx, scanner, mapper, mount_ops, sessions, nbd_pool,
                )
                .await;
            });

            Ok(StorageSessionHandle {
                session_id,
                parent_path,
            })
        })
    }

    fn stop_by_parent(
        &self,
        parent_path: String,
        reason: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), StorageSessionError>> + Send + '_>,
    > {
        Box::pin(async move {
            let session = self.sessions.lock().await.remove(&parent_path);
            if let Some(session) = session {
                cleanup_session(
                    session,
                    reason,
                    Arc::clone(&self.mapper),
                    Arc::clone(&self.mount_ops),
                    Arc::clone(&self.nbd_pool),
                )
                .await;
            }
            Ok(())
        })
    }

    fn stop_all(
        &self,
        reason: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), StorageSessionError>> + Send + '_>,
    > {
        Box::pin(async move {
            let sessions = {
                let mut locked = self.sessions.lock().await;
                locked
                    .drain()
                    .map(|(_, session)| session)
                    .collect::<Vec<_>>()
            };

            for session in sessions {
                cleanup_session(
                    session,
                    reason.clone(),
                    Arc::clone(&self.mapper),
                    Arc::clone(&self.mount_ops),
                    Arc::clone(&self.nbd_pool),
                )
                .await;
            }

            Ok(())
        })
    }
}

async fn run_storage_pipeline(
    device: AuthorizedStorageDevice,
    nbd_index: u32,
    mut cancel_rx: watch::Receiver<bool>,
    scanner: Arc<dyn Scanner>,
    mapper: Arc<dyn DeviceMapper>,
    mount_ops: Arc<dyn StorageSessionMountOps>,
    sessions: Arc<Mutex<HashMap<String, ActiveStorageSession>>>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
) {
    let parent_path = device.parent_path.clone();
    let dev_name = dev_name_from_path(&device.dev_path);
    let mount_point = mount_path_for(dev_name);
    let mount_path_str = mount_point.to_string_lossy().to_string();
    let read_only = device.permission == 0;

    info!(
        serial = %device.serial_number,
        dev = %device.device_name,
        nbd = nbd_index,
        "Storage session pipeline 开始"
    );

    if let Err(e) = mount_ops.mount_partition(&device.dev_path, &mount_path_str, read_only) {
        warn!(
            serial = %device.serial_number,
            dev = %device.device_name,
            error = %e,
            "Storage session 挂载失败"
        );
        remove_failed_session(&sessions, &nbd_pool, &parent_path, nbd_index).await;
        return;
    }

    if let Some(session) = sessions.lock().await.get_mut(&parent_path) {
        session.mount_path = Some(mount_point.clone());
    } else {
        let _ = mount_ops.umount(&mount_path_str);
        nbd_pool.lock().await.release(nbd_index);
        return;
    }

    let scan_result = tokio::select! {
        result = scanner.scan(&mount_point, &device.serial_number, &device.device_name) => result,
        _ = cancel_rx.changed() => {
            info!(serial = %device.serial_number, "Storage session 扫描被取消");
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "scan_cancelled".to_string(),
                Arc::clone(&mapper),
                Arc::clone(&mount_ops),
                Arc::clone(&nbd_pool),
            ).await;
            return;
        }
    };

    let scan_result = match scan_result {
        Ok(result) => result,
        Err(e) => {
            warn!(
                serial = %device.serial_number,
                dev = %device.device_name,
                error = %e,
                "Storage session 扫描失败"
            );
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "scan_failed".to_string(),
                Arc::clone(&mapper),
                Arc::clone(&mount_ops),
                Arc::clone(&nbd_pool),
            )
            .await;
            return;
        }
    };

    let source_size_bytes = device
        .capacity_bytes
        .and_then(|size| u64::try_from(size).ok())
        .unwrap_or_else(|| block_device_size_bytes(&device.dev_path));
    let map_ctx = MapContext {
        mount_path: mount_path_str,
        scan_result,
        permission: device.permission,
        source_size_bytes,
        nbd_device: format!("/dev/nbd{}", nbd_index),
    };

    let mapped_session = tokio::select! {
        result = mapper.map_device(map_ctx) => result,
        _ = cancel_rx.changed() => {
            info!(serial = %device.serial_number, "Storage session 映射被取消");
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "map_cancelled".to_string(),
                Arc::clone(&mapper),
                Arc::clone(&mount_ops),
                Arc::clone(&nbd_pool),
            ).await;
            return;
        }
    };

    match mapped_session {
        Ok(mapped_session) => {
            if let Some(session) = sessions.lock().await.get_mut(&parent_path) {
                session.mapped_session = Some(mapped_session);
                info!(serial = %device.serial_number, "Storage session 映射成功");
            } else if let Err(e) = mapper.unmap_device(mapped_session).await {
                warn!(
                    serial = %device.serial_number,
                    error = %e,
                    "Storage session 已移除，映射回滚失败"
                );
            }
        }
        Err(e) => {
            warn!(
                serial = %device.serial_number,
                dev = %device.device_name,
                error = %e,
                "Storage session 映射失败"
            );
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "map_failed".to_string(),
                Arc::clone(&mapper),
                Arc::clone(&mount_ops),
                Arc::clone(&nbd_pool),
            )
            .await;
        }
    }
}

async fn cleanup_removed_session(
    sessions: &Arc<Mutex<HashMap<String, ActiveStorageSession>>>,
    parent_path: &str,
    reason: String,
    mapper: Arc<dyn DeviceMapper>,
    mount_ops: Arc<dyn StorageSessionMountOps>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
) {
    let session = sessions.lock().await.remove(parent_path);
    if let Some(session) = session {
        cleanup_session(session, reason, mapper, mount_ops, nbd_pool).await;
    }
}

async fn cleanup_session(
    mut session: ActiveStorageSession,
    reason: String,
    mapper: Arc<dyn DeviceMapper>,
    mount_ops: Arc<dyn StorageSessionMountOps>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
) {
    let _ = session.cancel_tx.send(true);

    if let Some(mapped_session) = session.mapped_session.take() {
        if let Err(e) = mapper.unmap_device(mapped_session).await {
            warn!(error = %e, reason = %reason, "Storage session 映射清理失败");
        }
    }

    if let Some(mount_path) = session.mount_path.take() {
        let mount_path_str = mount_path.to_string_lossy().to_string();
        let should_umount = match mount_ops.mount_target_exists(&mount_path_str) {
            Ok(active) => active,
            Err(e) => {
                warn!(
                    mount_point = %mount_path_str,
                    error = %e,
                    reason = %reason,
                    "Storage session 读取挂载状态失败，继续尝试卸载"
                );
                true
            }
        };

        if should_umount {
            if let Err(e) = mount_ops.umount(&mount_path_str) {
                warn!(
                    mount_point = %mount_path_str,
                    error = %e,
                    reason = %reason,
                    "Storage session 卸载失败"
                );
            }
        }
    }

    nbd_pool.lock().await.release(session.nbd_index);
    info!(
        serial = %session.device.serial_number,
        reason = %reason,
        "Storage session 清理完成"
    );
}

async fn remove_failed_session(
    sessions: &Arc<Mutex<HashMap<String, ActiveStorageSession>>>,
    nbd_pool: &Arc<Mutex<NbdIndexPool>>,
    parent_path: &str,
    nbd_index: u32,
) {
    sessions.lock().await.remove(parent_path);
    nbd_pool.lock().await.release(nbd_index);
}

fn block_device_size_bytes(dev_path: &str) -> u64 {
    let Ok(output) = std::process::Command::new("blockdev")
        .args(["--getsize64", dev_path])
        .output()
    else {
        return 0;
    };

    if !output.status.success() {
        return 0;
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .unwrap_or(0)
}
