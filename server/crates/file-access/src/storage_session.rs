//! 大容量存储受控会话管理器。
//!
//! 该模块是 storage 链路的生命周期 owner。
//! 它直接驱动 mount、scan、虚拟介质构建、NBD 发布、gadget 暴露和清理回滚。

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use device_runtime::{DeviceRuntimeRegistry, DeviceRuntimeUpdate};
use storage::Storage;
use tokio::sync::{watch, Mutex};
use tracing::{info, warn};
use usb_identify::traits::{
    AuthorizedStorageDevice, ScanResult, Scanner, StorageSessionController, StorageSessionError,
    StorageSessionHandle,
};

use crate::exfat::fs::VirtualExfatFs;
use crate::gadget::GadgetRuntime;
use crate::media_builder::VirtualMediaBuilder;
use crate::publisher::{PublishedStorageRuntime, StoragePublisher, StorageRuntimePublisher};
use crate::raw_mount::{
    mount_partition, mount_path_for, mount_target_exists, MountOperations, RealMountOps,
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
        RealMountOps
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

pub(crate) trait StorageMediaBuilder: Send + Sync {
    fn build(
        &self,
        mount_path: &Path,
        scan_result: ScanResult,
        permission: i32,
        source_size_bytes: u64,
    ) -> Result<VirtualExfatFs, std::io::Error>;
}

impl StorageMediaBuilder for VirtualMediaBuilder {
    fn build(
        &self,
        mount_path: &Path,
        scan_result: ScanResult,
        permission: i32,
        source_size_bytes: u64,
    ) -> Result<VirtualExfatFs, std::io::Error> {
        self.build(mount_path, scan_result, permission, source_size_bytes)
    }
}

struct ActiveStorageSession {
    device: AuthorizedStorageDevice,
    nbd_index: u32,
    mount_path: Option<PathBuf>,
    published: Option<Box<dyn PublishedStorageRuntime>>,
    cancel_tx: watch::Sender<bool>,
}

/// 大容量存储受控会话管理器。
pub struct StorageSessionManager {
    scanner: Arc<dyn Scanner>,
    media_builder: Arc<dyn StorageMediaBuilder>,
    publisher: Arc<dyn StorageRuntimePublisher>,
    mount_ops: Arc<dyn StorageSessionMountOps>,
    runtime_registry: Arc<DeviceRuntimeRegistry>,
    sessions: Arc<Mutex<HashMap<String, ActiveStorageSession>>>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
}

fn stable_hash64(value: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    value
        .as_bytes()
        .iter()
        .fold(FNV_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
        })
}

impl StorageSessionManager {
    /// 创建生产环境 manager。
    pub fn new(
        scanner: Arc<dyn Scanner>,
        storage: Arc<Storage>,
        gadget: GadgetRuntime,
        runtime_registry: Arc<DeviceRuntimeRegistry>,
    ) -> Self {
        Self::with_components(
            scanner,
            Arc::new(VirtualMediaBuilder::new(storage)),
            Arc::new(StoragePublisher::new(gadget)),
            Arc::new(RealStorageSessionMountOps),
            runtime_registry,
            NbdIndexPool::default(),
        )
    }

    pub(crate) fn with_components(
        scanner: Arc<dyn Scanner>,
        media_builder: Arc<dyn StorageMediaBuilder>,
        publisher: Arc<dyn StorageRuntimePublisher>,
        mount_ops: Arc<dyn StorageSessionMountOps>,
        runtime_registry: Arc<DeviceRuntimeRegistry>,
        nbd_pool: NbdIndexPool,
    ) -> Self {
        Self {
            scanner,
            media_builder,
            publisher,
            mount_ops,
            runtime_registry,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            nbd_pool: Arc::new(Mutex::new(nbd_pool)),
        }
    }

    fn session_id(device: &AuthorizedStorageDevice) -> String {
        format!("storage_{:016x}", stable_hash64(&device.parent_path))
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

            let session_id = Self::session_id(&device);
            let parent_path = device.parent_path.clone();
            let (cancel_tx, cancel_rx) = watch::channel(false);

            let mut sessions_locked = self.sessions.lock().await;
            if !sessions_locked.is_empty() {
                return Err(StorageSessionError::Rejected(
                    "当前设备只有一个业务 mass_storage LUN，已有 active storage session".into(),
                ));
            }

            let nbd_index = self
                .nbd_pool
                .lock()
                .await
                .acquire()
                .ok_or_else(|| StorageSessionError::Rejected("NBD 设备号池耗尽".into()))?;

            let active = ActiveStorageSession {
                device: device.clone(),
                nbd_index,
                mount_path: None,
                published: None,
                cancel_tx,
            };

            sessions_locked.insert(parent_path.clone(), active);
            drop(sessions_locked);

            let scanner = Arc::clone(&self.scanner);
            let media_builder = Arc::clone(&self.media_builder);
            let publisher = Arc::clone(&self.publisher);
            let mount_ops = Arc::clone(&self.mount_ops);
            let runtime_registry = Arc::clone(&self.runtime_registry);
            let sessions = Arc::clone(&self.sessions);
            let nbd_pool = Arc::clone(&self.nbd_pool);
            let pipeline_session_id = session_id.clone();

            tokio::spawn(async move {
                run_storage_pipeline(
                    pipeline_session_id,
                    device,
                    nbd_index,
                    cancel_rx,
                    scanner,
                    media_builder,
                    publisher,
                    mount_ops,
                    runtime_registry,
                    sessions,
                    nbd_pool,
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
                    Arc::clone(&self.mount_ops),
                    Arc::clone(&self.runtime_registry),
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
                    Arc::clone(&self.mount_ops),
                    Arc::clone(&self.runtime_registry),
                    Arc::clone(&self.nbd_pool),
                )
                .await;
            }

            Ok(())
        })
    }
}

async fn run_storage_pipeline(
    session_id: String,
    device: AuthorizedStorageDevice,
    nbd_index: u32,
    mut cancel_rx: watch::Receiver<bool>,
    scanner: Arc<dyn Scanner>,
    media_builder: Arc<dyn StorageMediaBuilder>,
    publisher: Arc<dyn StorageRuntimePublisher>,
    mount_ops: Arc<dyn StorageSessionMountOps>,
    runtime_registry: Arc<DeviceRuntimeRegistry>,
    sessions: Arc<Mutex<HashMap<String, ActiveStorageSession>>>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
) {
    let parent_path = device.parent_path.clone();
    let mount_point = mount_path_for(&session_id);
    let mount_path_str = mount_point.to_string_lossy().to_string();
    let read_only = device.permission == 0;

    info!(
        serial = %device.serial_number,
        dev = %device.device_name,
        nbd = nbd_index,
        "Storage session pipeline 开始"
    );

    info!(
        serial = %device.serial_number,
        dev = %device.device_name,
        mount_point = %mount_path_str,
        "Storage session 进入挂载阶段"
    );
    update_runtime(
        &runtime_registry,
        &device.runtime_id,
        "processing",
        "mount",
        "",
        "",
    );

    if let Err(e) = mount_ops.mount_partition(&device.dev_path, &mount_path_str, read_only) {
        update_runtime(
            &runtime_registry,
            &device.runtime_id,
            "failed",
            "mount",
            "mount_failed",
            &e.to_string(),
        );
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
        runtime_registry.mark_removed(&device.runtime_id, "session_removed_before_mount_record");
        return;
    }

    info!(
        serial = %device.serial_number,
        dev = %device.device_name,
        mount = %mount_point.display(),
        "Storage session 进入扫描阶段"
    );
    update_runtime(
        &runtime_registry,
        &device.runtime_id,
        "processing",
        "scan",
        "",
        "",
    );

    let scan_result = tokio::select! {
        result = scanner.scan(&mount_point, &device.serial_number, &device.device_name) => result,
        _ = cancel_rx.changed() => {
            info!(serial = %device.serial_number, "Storage session 扫描被取消");
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "scan_cancelled".to_string(),
                Arc::clone(&mount_ops),
                Arc::clone(&runtime_registry),
                Arc::clone(&nbd_pool),
            ).await;
            return;
        }
    };

    let scan_result = match scan_result {
        Ok(result) => result,
        Err(e) => {
            update_runtime(
                &runtime_registry,
                &device.runtime_id,
                "failed",
                "scan",
                "scan_failed",
                &e.to_string(),
            );
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
                Arc::clone(&mount_ops),
                Arc::clone(&runtime_registry),
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

    info!(
        serial = %device.serial_number,
        dev = %device.device_name,
        "Storage session 进入虚拟介质构建阶段"
    );
    update_runtime(
        &runtime_registry,
        &device.runtime_id,
        "processing",
        "media_build",
        "",
        "",
    );

    let media = match media_builder.build(
        &mount_point,
        scan_result,
        device.permission,
        source_size_bytes,
    ) {
        Ok(media) => media,
        Err(e) => {
            update_runtime(
                &runtime_registry,
                &device.runtime_id,
                "failed",
                "media_build",
                "media_build_failed",
                &e.to_string(),
            );
            warn!(
                serial = %device.serial_number,
                dev = %device.device_name,
                error = %e,
                "Storage session 虚拟介质构建失败"
            );
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "media_build_failed".to_string(),
                Arc::clone(&mount_ops),
                Arc::clone(&runtime_registry),
                Arc::clone(&nbd_pool),
            )
            .await;
            return;
        }
    };

    info!(
        serial = %device.serial_number,
        dev = %device.device_name,
        nbd = nbd_index,
        "Storage session 进入发布阶段"
    );
    update_runtime(
        &runtime_registry,
        &device.runtime_id,
        "processing",
        "publish",
        "",
        "",
    );

    let published = tokio::select! {
        result = publisher.publish(media, nbd_index, read_only) => result,
        _ = cancel_rx.changed() => {
            info!(serial = %device.serial_number, "Storage session 发布被取消");
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "publish_cancelled".to_string(),
                Arc::clone(&mount_ops),
                Arc::clone(&runtime_registry),
                Arc::clone(&nbd_pool),
            ).await;
            return;
        }
    };

    match published {
        Ok(published) => {
            if let Some(session) = sessions.lock().await.get_mut(&parent_path) {
                session.published = Some(published);
                update_runtime(
                    &runtime_registry,
                    &device.runtime_id,
                    "mapped",
                    "publish",
                    "",
                    "",
                );
                info!(serial = %device.serial_number, "Storage session 映射成功");
            } else {
                published.stop().await;
                nbd_pool.lock().await.release(nbd_index);
                runtime_registry.mark_removed(&device.runtime_id, "session_removed_after_publish");
                warn!(
                    serial = %device.serial_number,
                    "Storage session 已移除，发布资源已回滚"
                );
            }
        }
        Err(e) => {
            update_runtime(
                &runtime_registry,
                &device.runtime_id,
                "failed",
                "publish",
                "publish_failed",
                &e.to_string(),
            );
            warn!(
                serial = %device.serial_number,
                dev = %device.device_name,
                error = %e,
                "Storage session 映射失败"
            );
            cleanup_removed_session(
                &sessions,
                &parent_path,
                "publish_failed".to_string(),
                Arc::clone(&mount_ops),
                Arc::clone(&runtime_registry),
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
    mount_ops: Arc<dyn StorageSessionMountOps>,
    runtime_registry: Arc<DeviceRuntimeRegistry>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
) {
    let session = sessions.lock().await.remove(parent_path);
    if let Some(session) = session {
        cleanup_session(session, reason, mount_ops, runtime_registry, nbd_pool).await;
    }
}

async fn cleanup_session(
    mut session: ActiveStorageSession,
    reason: String,
    mount_ops: Arc<dyn StorageSessionMountOps>,
    runtime_registry: Arc<DeviceRuntimeRegistry>,
    nbd_pool: Arc<Mutex<NbdIndexPool>>,
) {
    let _ = session.cancel_tx.send(true);

    if let Some(published) = session.published.take() {
        published.stop().await;
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
    runtime_registry.mark_removed(&session.device.runtime_id, reason.clone());
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

fn update_runtime(
    registry: &DeviceRuntimeRegistry,
    runtime_id: &str,
    status: &str,
    stage: &str,
    fail_code: &str,
    fail_reason: &str,
) {
    registry.update(
        runtime_id,
        DeviceRuntimeUpdate {
            status: status.to_string(),
            stage: stage.to_string(),
            fail_code: fail_code.to_string(),
            fail_reason: fail_reason.to_string(),
        },
    );
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::future::Future;
    use std::path::Path;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex as StdMutex};

    use device_runtime::{DeviceRuntimeCreate, DeviceRuntimeRegistry};
    use tempfile::tempdir;
    use usb_identify::traits::{ScanError, ScanResult};

    use super::*;
    use crate::types::PolicySnapshot;

    #[derive(Default)]
    struct FakeScanner {
        scan_count: AtomicUsize,
        fail: AtomicBool,
    }

    impl Scanner for FakeScanner {
        fn scan(
            &self,
            _mount_path: &Path,
            _device_sn: &str,
            _device_name: &str,
        ) -> Pin<Box<dyn Future<Output = Result<ScanResult, ScanError>> + Send + '_>> {
            self.scan_count.fetch_add(1, Ordering::SeqCst);
            let fail = self.fail.load(Ordering::SeqCst);
            Box::pin(async move {
                if fail {
                    Err(ScanError::Failed("scan failed".into()))
                } else {
                    Ok(ScanResult {
                        is_clean: true,
                        infected_files: vec![],
                    })
                }
            })
        }

        fn cancel(&self, _mount_path: &Path) {}
    }

    #[derive(Default)]
    struct FakeMountOps {
        mount_count: AtomicUsize,
        umount_count: AtomicUsize,
        fail_mount: AtomicBool,
        mounted: AtomicBool,
        last_mount_point: StdMutex<Option<String>>,
    }

    impl StorageSessionMountOps for FakeMountOps {
        fn mount_partition(
            &self,
            _dev_path: &str,
            mount_point: &str,
            _read_only: bool,
        ) -> Result<(), StorageSessionError> {
            self.mount_count.fetch_add(1, Ordering::SeqCst);
            *self.last_mount_point.lock().unwrap() = Some(mount_point.to_string());
            if self.fail_mount.load(Ordering::SeqCst) {
                return Err(StorageSessionError::Failed("mount failed".into()));
            }
            self.mounted.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn umount(&self, _mount_point: &str) -> Result<(), StorageSessionError> {
            self.umount_count.fetch_add(1, Ordering::SeqCst);
            self.mounted.store(false, Ordering::SeqCst);
            Ok(())
        }

        fn mount_target_exists(&self, _mount_point: &str) -> Result<bool, StorageSessionError> {
            Ok(self.mounted.load(Ordering::SeqCst))
        }
    }

    #[derive(Default)]
    struct FakeMediaBuilder {
        build_count: AtomicUsize,
        fail: AtomicBool,
    }

    impl StorageMediaBuilder for FakeMediaBuilder {
        fn build(
            &self,
            _mount_path: &Path,
            _scan_result: ScanResult,
            permission: i32,
            source_size_bytes: u64,
        ) -> Result<VirtualExfatFs, std::io::Error> {
            self.build_count.fetch_add(1, Ordering::SeqCst);
            if self.fail.load(Ordering::SeqCst) {
                return Err(std::io::Error::other("media build failed"));
            }

            let dir = tempdir()?;
            let snapshot = PolicySnapshot {
                exec_control_enabled: false,
                file_type_blacklist_enabled: false,
                auto_read_control_enabled: false,
                blacklist_extensions: HashSet::new(),
                permission,
            };
            VirtualExfatFs::build(dir.path(), &[], snapshot, source_size_bytes)
        }
    }

    struct FakePublishedStorage {
        stop_count: Arc<AtomicUsize>,
    }

    impl PublishedStorageRuntime for FakePublishedStorage {
        fn stop(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
            Box::pin(async move {
                self.stop_count.fetch_add(1, Ordering::SeqCst);
            })
        }
    }

    #[derive(Default)]
    struct FakePublisher {
        publish_count: AtomicUsize,
        stop_count: Arc<AtomicUsize>,
        fail_publish: AtomicBool,
    }

    impl StorageRuntimePublisher for FakePublisher {
        fn publish(
            &self,
            _fs: VirtualExfatFs,
            _nbd_index: u32,
            _readonly: bool,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            Box<dyn PublishedStorageRuntime>,
                            crate::publisher::PublishError,
                        >,
                    > + Send
                    + '_,
            >,
        > {
            self.publish_count.fetch_add(1, Ordering::SeqCst);
            let stop_count = Arc::clone(&self.stop_count);
            let fail = self.fail_publish.load(Ordering::SeqCst);
            Box::pin(async move {
                if fail {
                    return Err(crate::publisher::PublishError::Nbd("publish failed".into()));
                }
                let published: Box<dyn PublishedStorageRuntime> =
                    Box::new(FakePublishedStorage { stop_count });
                Ok(published)
            })
        }
    }

    fn test_device(serial: &str) -> AuthorizedStorageDevice {
        AuthorizedStorageDevice {
            runtime_id: format!("runtime__{serial}"),
            parent_path: format!("/sys/devices/{serial}"),
            sys_path: format!("/sys/devices/{serial}/1-1:1.0"),
            dev_path: "/dev/sda1".into(),
            serial_number: serial.into(),
            vid: "0781".into(),
            pid: "5567".into(),
            device_name: "Cruzer Blade".into(),
            capacity_bytes: Some(16 * 1024 * 1024),
            permission: 1,
        }
    }

    struct TestManager {
        manager: StorageSessionManager,
        scanner: Arc<FakeScanner>,
        media_builder: Arc<FakeMediaBuilder>,
        publisher: Arc<FakePublisher>,
        mount_ops: Arc<FakeMountOps>,
        runtime_registry: Arc<DeviceRuntimeRegistry>,
    }

    fn test_manager(pool_size: u32) -> TestManager {
        let scanner = Arc::new(FakeScanner::default());
        let media_builder = Arc::new(FakeMediaBuilder::default());
        let publisher = Arc::new(FakePublisher::default());
        let mount_ops = Arc::new(FakeMountOps::default());
        let runtime_registry = Arc::new(DeviceRuntimeRegistry::new());
        let manager = StorageSessionManager::with_components(
            scanner.clone(),
            media_builder.clone(),
            publisher.clone(),
            mount_ops.clone(),
            Arc::clone(&runtime_registry),
            NbdIndexPool::new(pool_size),
        );

        TestManager {
            manager,
            scanner,
            media_builder,
            publisher,
            mount_ops,
            runtime_registry,
        }
    }

    async fn wait_until(predicate: impl Fn() -> bool) {
        for _ in 0..50 {
            if predicate() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }

    fn create_runtime(registry: &DeviceRuntimeRegistry, device: &AuthorizedStorageDevice) {
        registry.create(DeviceRuntimeCreate {
            runtime_id: device.runtime_id.clone(),
            parent_path: device.parent_path.clone(),
            interface_path: device.sys_path.clone(),
            serial_number: device.serial_number.clone(),
            device_name: device.device_name.clone(),
            device_type: "storage".to_string(),
            interface_type: "mass_storage".to_string(),
            status: "accepted".to_string(),
            stage: "admission".to_string(),
            fail_code: String::new(),
            fail_reason: String::new(),
        });
    }

    #[tokio::test]
    async fn storage_session_starts_mount_scan_media_and_publish_pipeline() {
        let ctx = test_manager(4);
        let device = test_device("SN-1");
        create_runtime(&ctx.runtime_registry, &device);

        let handle = ctx
            .manager
            .start_authorized_storage(device.clone())
            .await
            .unwrap();

        assert_eq!(handle.parent_path, "/sys/devices/SN-1");
        wait_until(|| ctx.publisher.publish_count.load(Ordering::SeqCst) == 1).await;

        assert_eq!(ctx.mount_ops.mount_count.load(Ordering::SeqCst), 1);
        assert_eq!(ctx.scanner.scan_count.load(Ordering::SeqCst), 1);
        assert_eq!(ctx.media_builder.build_count.load(Ordering::SeqCst), 1);
        assert_eq!(ctx.publisher.publish_count.load(Ordering::SeqCst), 1);
        assert!(ctx.manager.has_active_storage().await);

        let runtime = ctx.runtime_registry.get(&device.runtime_id).unwrap();
        assert_eq!(runtime.status, "mapped");
        assert_eq!(runtime.stage, "publish");
        assert!(runtime.fail_code.is_empty());
    }

    #[tokio::test]
    async fn storage_session_mount_path_is_owned_by_session_id() {
        let ctx = test_manager(4);
        let device = test_device("SN-SESSION-MOUNT");
        create_runtime(&ctx.runtime_registry, &device);

        let handle = ctx
            .manager
            .start_authorized_storage(device.clone())
            .await
            .unwrap();

        wait_until(|| ctx.mount_ops.mount_count.load(Ordering::SeqCst) == 1).await;

        let mount_point = ctx
            .mount_ops
            .last_mount_point
            .lock()
            .unwrap()
            .clone()
            .expect("mount point should be recorded");

        assert_eq!(
            mount_point,
            format!("/mnt/usb-control/raw/{}", handle.session_id)
        );
        assert_ne!(mount_point, "/mnt/usb-control/raw/sda1");
    }

    #[tokio::test]
    async fn storage_session_id_is_short_and_does_not_expose_sysfs_path() {
        let ctx = test_manager(4);
        let device = test_device("SN-SHORT-ID");
        create_runtime(&ctx.runtime_registry, &device);

        let handle = ctx.manager.start_authorized_storage(device).await.unwrap();

        assert!(handle.session_id.starts_with("storage_"));
        assert_eq!(handle.session_id.len(), "storage_".len() + 16);
        assert!(!handle.session_id.contains("/sys/"));
        assert!(!handle.session_id.contains("devices"));
    }

    #[tokio::test]
    async fn rejects_second_active_storage_session_for_single_lun() {
        let ctx = test_manager(4);

        ctx.manager
            .start_authorized_storage(test_device("SN-1"))
            .await
            .unwrap();
        let second = ctx
            .manager
            .start_authorized_storage(test_device("SN-2"))
            .await;

        assert!(matches!(second, Err(StorageSessionError::Rejected(_))));
    }

    #[tokio::test]
    async fn stop_by_parent_stops_published_storage_and_umounts() {
        let ctx = test_manager(4);
        let device = test_device("SN-2");
        create_runtime(&ctx.runtime_registry, &device);

        ctx.manager
            .start_authorized_storage(device.clone())
            .await
            .unwrap();
        wait_until(|| ctx.publisher.publish_count.load(Ordering::SeqCst) == 1).await;

        ctx.manager
            .stop_by_parent("/sys/devices/SN-2".into(), "usb_remove".into())
            .await
            .unwrap();

        assert_eq!(ctx.publisher.stop_count.load(Ordering::SeqCst), 1);
        assert_eq!(ctx.mount_ops.umount_count.load(Ordering::SeqCst), 1);
        assert!(!ctx.manager.has_active_storage().await);

        let runtime = ctx.runtime_registry.get(&device.runtime_id).unwrap();
        assert_eq!(runtime.status, "removed");
        assert_eq!(runtime.stage, "cleanup");
        assert_eq!(runtime.fail_code, "device_removed");
    }

    #[tokio::test]
    async fn stop_by_parent_is_idempotent() {
        let ctx = test_manager(4);

        ctx.manager
            .start_authorized_storage(test_device("SN-3"))
            .await
            .unwrap();
        wait_until(|| ctx.publisher.publish_count.load(Ordering::SeqCst) == 1).await;

        ctx.manager
            .stop_by_parent("/sys/devices/SN-3".into(), "removed".into())
            .await
            .unwrap();
        ctx.manager
            .stop_by_parent("/sys/devices/SN-3".into(), "removed_again".into())
            .await
            .unwrap();

        assert_eq!(ctx.publisher.stop_count.load(Ordering::SeqCst), 1);
        assert!(!ctx.manager.has_active_storage().await);
    }

    #[tokio::test]
    async fn mount_failure_releases_session_and_nbd_index() {
        let ctx = test_manager(1);
        ctx.mount_ops.fail_mount.store(true, Ordering::SeqCst);
        let device = test_device("SN-4");
        create_runtime(&ctx.runtime_registry, &device);

        ctx.manager
            .start_authorized_storage(device.clone())
            .await
            .unwrap();

        wait_until(|| ctx.mount_ops.mount_count.load(Ordering::SeqCst) == 1).await;
        for _ in 0..50 {
            if !ctx.manager.has_active_storage().await {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(!ctx.manager.has_active_storage().await);
        assert_eq!(ctx.publisher.publish_count.load(Ordering::SeqCst), 0);
        let runtime = ctx.runtime_registry.get(&device.runtime_id).unwrap();
        assert_eq!(runtime.status, "failed");
        assert_eq!(runtime.stage, "mount");
        assert_eq!(runtime.fail_code, "mount_failed");

        ctx.mount_ops.fail_mount.store(false, Ordering::SeqCst);
        ctx.manager
            .start_authorized_storage(test_device("SN-5"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn stop_all_cleans_every_active_session() {
        let ctx = test_manager(4);

        ctx.manager
            .start_authorized_storage(test_device("SN-6"))
            .await
            .unwrap();
        wait_until(|| ctx.publisher.publish_count.load(Ordering::SeqCst) == 1).await;

        ctx.manager
            .stop_all("service_shutdown".into())
            .await
            .unwrap();

        assert_eq!(ctx.publisher.stop_count.load(Ordering::SeqCst), 1);
        assert_eq!(ctx.mount_ops.umount_count.load(Ordering::SeqCst), 1);
        assert!(!ctx.manager.has_active_storage().await);
    }
}
