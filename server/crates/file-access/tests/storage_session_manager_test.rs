use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use file_access::storage_session::{NbdIndexPool, StorageSessionManager, StorageSessionMountOps};
use usb_identify::traits::{
    AuthorizedStorageDevice, DeviceMapper, MapContext, MapError, MappedSession, ScanError,
    ScanResult, Scanner, StorageSessionController, StorageSessionError, UnmapError,
};

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
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ScanResult, ScanError>> + Send + '_>,
    > {
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
struct FakeMapper {
    map_count: AtomicUsize,
    unmap_count: AtomicUsize,
    fail_map: AtomicBool,
    last_ctx: Mutex<Option<MapContext>>,
}

impl DeviceMapper for FakeMapper {
    fn map_device(
        &self,
        ctx: MapContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<MappedSession, MapError>> + Send + '_>,
    > {
        self.map_count.fetch_add(1, Ordering::SeqCst);
        *self.last_ctx.lock().unwrap() = Some(ctx.clone());
        let fail = self.fail_map.load(Ordering::SeqCst);
        Box::pin(async move {
            if fail {
                Err(MapError::BuildFailed("map failed".into()))
            } else {
                Ok(MappedSession {
                    id: "mapped-1".into(),
                    mount_path: ctx.mount_path,
                    nbd_device: ctx.nbd_device,
                })
            }
        })
    }

    fn unmap_device(
        &self,
        _session: MappedSession,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), UnmapError>> + Send + '_>>
    {
        self.unmap_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

#[derive(Default)]
struct FakeMountOps {
    mount_count: AtomicUsize,
    umount_count: AtomicUsize,
    fail_mount: AtomicBool,
    mounted: AtomicBool,
}

impl StorageSessionMountOps for FakeMountOps {
    fn mount_partition(
        &self,
        _dev_path: &str,
        _mount_point: &str,
        _read_only: bool,
    ) -> Result<(), StorageSessionError> {
        self.mount_count.fetch_add(1, Ordering::SeqCst);
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

fn test_device(serial: &str) -> AuthorizedStorageDevice {
    AuthorizedStorageDevice {
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

async fn wait_until(predicate: impl Fn() -> bool) {
    for _ in 0..50 {
        if predicate() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn storage_session_starts_mount_scan_and_map_pipeline() {
    let scanner = Arc::new(FakeScanner::default());
    let mapper = Arc::new(FakeMapper::default());
    let mount_ops = Arc::new(FakeMountOps::default());
    let manager = StorageSessionManager::with_mount_ops(
        scanner.clone(),
        mapper.clone(),
        mount_ops.clone(),
        NbdIndexPool::new(4),
    );

    let handle = manager
        .start_authorized_storage(test_device("SN-1"))
        .await
        .unwrap();

    assert_eq!(handle.parent_path, "/sys/devices/SN-1");
    wait_until(|| mapper.map_count.load(Ordering::SeqCst) == 1).await;

    assert_eq!(mount_ops.mount_count.load(Ordering::SeqCst), 1);
    assert_eq!(scanner.scan_count.load(Ordering::SeqCst), 1);
    assert_eq!(mapper.map_count.load(Ordering::SeqCst), 1);
    assert!(manager.has_active_storage().await);

    let ctx = mapper.last_ctx.lock().unwrap();
    let ctx = ctx.as_ref().unwrap();
    assert_eq!(ctx.nbd_device, "/dev/nbd3");
    assert_eq!(ctx.permission, 1);
}

#[tokio::test]
async fn stop_by_parent_unmaps_umounts_and_releases_session() {
    let scanner = Arc::new(FakeScanner::default());
    let mapper = Arc::new(FakeMapper::default());
    let mount_ops = Arc::new(FakeMountOps::default());
    let manager = StorageSessionManager::with_mount_ops(
        scanner,
        mapper.clone(),
        mount_ops.clone(),
        NbdIndexPool::new(4),
    );

    manager
        .start_authorized_storage(test_device("SN-2"))
        .await
        .unwrap();
    wait_until(|| mapper.map_count.load(Ordering::SeqCst) == 1).await;

    manager
        .stop_by_parent("/sys/devices/SN-2".into(), "usb_remove".into())
        .await
        .unwrap();

    assert_eq!(mapper.unmap_count.load(Ordering::SeqCst), 1);
    assert_eq!(mount_ops.umount_count.load(Ordering::SeqCst), 1);
    assert!(!manager.has_active_storage().await);
}

#[tokio::test]
async fn mount_failure_releases_session_and_nbd_index() {
    let scanner = Arc::new(FakeScanner::default());
    let mapper = Arc::new(FakeMapper::default());
    let mount_ops = Arc::new(FakeMountOps::default());
    mount_ops.fail_mount.store(true, Ordering::SeqCst);
    let manager = StorageSessionManager::with_mount_ops(
        scanner,
        mapper.clone(),
        mount_ops.clone(),
        NbdIndexPool::new(1),
    );

    manager
        .start_authorized_storage(test_device("SN-3"))
        .await
        .unwrap();

    wait_until(|| mount_ops.mount_count.load(Ordering::SeqCst) == 1).await;
    for _ in 0..50 {
        if !manager.has_active_storage().await {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(!manager.has_active_storage().await);
    assert_eq!(mapper.map_count.load(Ordering::SeqCst), 0);

    mount_ops.fail_mount.store(false, Ordering::SeqCst);
    manager
        .start_authorized_storage(test_device("SN-4"))
        .await
        .unwrap();
}

#[tokio::test]
async fn stop_all_cleans_every_active_session() {
    let scanner = Arc::new(FakeScanner::default());
    let mapper = Arc::new(FakeMapper::default());
    let mount_ops = Arc::new(FakeMountOps::default());
    let manager = StorageSessionManager::with_mount_ops(
        scanner,
        mapper.clone(),
        mount_ops.clone(),
        NbdIndexPool::new(4),
    );

    manager
        .start_authorized_storage(test_device("SN-5"))
        .await
        .unwrap();
    wait_until(|| mapper.map_count.load(Ordering::SeqCst) == 1).await;

    manager.stop_all("service_shutdown".into()).await.unwrap();

    assert_eq!(mapper.unmap_count.load(Ordering::SeqCst), 1);
    assert_eq!(mount_ops.umount_count.load(Ordering::SeqCst), 1);
    assert!(!manager.has_active_storage().await);
}
