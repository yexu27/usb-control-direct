//! 白名单管理服务。
//!
//! 提供白名单条目的增删改查接口，内置序列号索引缓存以加速插入许可判断。
//! 缓存在构造时从数据库全量加载，后续随写操作同步更新，无需定期刷新。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockWriteGuard};

use storage::model::{UsbWhitelist, UsbWhitelistInsert, WhitelistCacheSnapshotEntry};
use storage::{Storage, StorageError};
use tracing::{debug, info, trace, warn};

use crate::error::WhitelistError;

/// 序列号缓存条目，记录数据库行 ID 和权限值。
struct CacheEntry {
    /// 数据库行 ID。
    id: i64,
    /// 权限值（对应 T01.permission 字段）。
    permission: i32,
}

/// 白名单查询结果。
pub struct WhitelistResult {
    /// 数据库行 ID。
    pub id: i64,
    /// 权限值。
    pub permission: i32,
}

/// 添加白名单条目的请求参数。
pub struct AddWhitelistRequest {
    /// 设备序列号，不能为空。
    pub serial_number: String,
    /// 厂商 ID（可选）。
    pub vid: Option<String>,
    /// 产品 ID（可选）。
    pub pid: Option<String>,
    /// 设备名称（可选）。
    pub device_name: Option<String>,
    /// 存储容量（字节，可选）。
    pub capacity_bytes: Option<i64>,
    /// 设备类型。
    pub device_type: String,
    /// 描述（可选）。
    pub description: Option<String>,
    /// 权限值。
    pub permission: i32,
    /// 添加方式（0: 手动添加，1: 自动添加）。
    pub add_method: i32,
}

/// 白名单管理服务。
///
/// 封装对 T01 白名单表的全部业务操作，提供带缓存的序列号查询接口。
/// 构造后即可用，缓存在 [`WhitelistManager::new`] 时从数据库全量加载。
pub struct WhitelistManager {
    /// 存储层句柄。
    storage: Arc<Storage>,
    /// 串行化白名单增删改与策略全量导入。
    mutation_lock: Mutex<()>,
    /// 序列号 → 缓存条目，用于快速判断是否在白名单中。
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl WhitelistManager {
    /// 创建白名单管理服务并从数据库加载缓存。
    ///
    /// 参数:
    /// - `storage`: 已打开的存储层句柄。
    ///
    /// 返回:
    /// - 成功时返回初始化完成的 [`WhitelistManager`]；
    /// - 数据库读取失败时返回 [`WhitelistError`]。
    pub fn new(storage: Arc<Storage>) -> Result<Self, WhitelistError> {
        info!("初始化白名单管理器");
        let manager = WhitelistManager {
            storage,
            mutation_lock: Mutex::new(()),
            cache: RwLock::new(HashMap::new()),
        };
        manager.reload_cache()?;
        let cache_size = manager.cache.read().map(|c| c.len()).unwrap_or(0);
        info!(cache_size, "白名单缓存加载完成");
        Ok(manager)
    }

    /// 从数据库全量重建序列号缓存。
    ///
    /// 返回:
    /// - 重建成功时返回 `()`；数据库读取失败时返回 [`WhitelistError`]。
    pub fn reload_cache(&self) -> Result<(), WhitelistError> {
        let _mutation = match self.mutation_lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("白名单变更协调锁中毒，尝试通过全量重载恢复");
                poisoned.into_inner()
            }
        };
        let mut cache = match self.cache.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("白名单缓存锁中毒，尝试通过全量重载恢复");
                poisoned.into_inner()
            }
        };
        let items = self.storage.whitelist_query_all()?;
        let snapshot = items
            .into_iter()
            .map(|item| WhitelistCacheSnapshotEntry {
                id: item.id,
                serial_number: item.serial_number,
                permission: item.permission,
            })
            .collect();
        Self::replace_cache(&mut cache, snapshot);
        self.cache.clear_poison();
        self.mutation_lock.clear_poison();
        Ok(())
    }

    /// 在白名单变更协调锁内执行策略导入并原子替换缓存。
    ///
    /// `import` 必须执行完整数据库事务，
    /// 并返回事务内构造的完整白名单缓存快照。
    /// 增删改操作与本方法共用同一把锁，
    /// 不会在数据库提交与缓存替换之间交错。
    pub fn coordinate_policy_import<F>(&self, import: F) -> Result<(), WhitelistError>
    where
        F: FnOnce() -> Result<Vec<WhitelistCacheSnapshotEntry>, StorageError>,
    {
        let _mutation = self.lock_mutations()?;
        let mut cache = self.lock_cache_for_mutation()?;
        let snapshot = import()?;
        Self::replace_cache(&mut cache, snapshot);
        Ok(())
    }

    /// 检查序列号是否在白名单中。
    ///
    /// 参数:
    /// - `sn`: 待查询的设备序列号。
    ///
    /// 返回:
    /// - 在白名单中时返回 `Some(WhitelistResult)`，携带行 ID 和权限值；
    /// - 不在白名单中时返回 `None`。
    pub fn is_whitelisted(&self, sn: &str) -> Option<WhitelistResult> {
        trace!(sn = %sn, "白名单查询");
        if self.mutation_lock.is_poisoned() {
            return None;
        }
        let cache = self.cache.read().ok()?;
        if self.mutation_lock.is_poisoned() {
            return None;
        }
        cache.get(sn).map(|entry| WhitelistResult {
            id: entry.id,
            permission: entry.permission,
        })
    }

    /// 添加白名单条目。
    ///
    /// 参数:
    /// - `req`: 添加请求参数，`serial_number` 不能为空。
    ///
    /// 返回:
    /// - 成功时返回新增条目的数据库行 ID；
    /// - 序列号为空时返回 [`WhitelistError::SerialNumberEmpty`]；
    /// - 序列号已存在时返回 [`WhitelistError::AlreadyExists`]；
    /// - 存储层失败时返回 [`WhitelistError::Storage`]。
    pub fn add(&self, req: AddWhitelistRequest) -> Result<i64, WhitelistError> {
        let _mutation = self.lock_mutations()?;
        if req.serial_number.is_empty() {
            return Err(WhitelistError::SerialNumberEmpty);
        }
        let mut cache = self.lock_cache_for_mutation()?;

        let sn = req.serial_number.clone();
        let insert = UsbWhitelistInsert {
            serial_number: req.serial_number,
            vid: req.vid,
            pid: req.pid,
            device_name: req.device_name,
            capacity_bytes: req.capacity_bytes,
            device_type: req.device_type,
            description: req.description,
            permission: req.permission,
            add_method: req.add_method,
        };

        let permission = insert.permission;
        let id = self.storage.whitelist_insert(&insert).map_err(|e| {
            if matches!(e, StorageError::AlreadyExists) {
                WhitelistError::AlreadyExists(sn.clone())
            } else {
                WhitelistError::Storage(e)
            }
        })?;

        // 写入缓存
        cache.insert(sn.clone(), CacheEntry { id, permission });
        debug!("白名单缓存新增: sn={sn}, id={id}");

        Ok(id)
    }

    /// 删除白名单条目。
    ///
    /// 参数:
    /// - `sn`: 目标设备序列号，不能为空。
    ///
    /// 返回:
    /// - 成功时返回 `()`；
    /// - 序列号为空时返回 [`WhitelistError::SerialNumberEmpty`]；
    /// - 条目不存在时返回 [`WhitelistError::NotFound`]；
    /// - 存储层失败时返回 [`WhitelistError::Storage`]。
    pub fn remove(&self, sn: &str) -> Result<(), WhitelistError> {
        let _mutation = self.lock_mutations()?;
        if sn.is_empty() {
            return Err(WhitelistError::SerialNumberEmpty);
        }
        let mut cache = self.lock_cache_for_mutation()?;

        let item = self
            .storage
            .whitelist_query_by_sn(sn)?
            .ok_or_else(|| WhitelistError::NotFound(sn.to_string()))?;

        self.storage.whitelist_delete(item.id)?;

        cache.remove(sn);
        debug!("白名单缓存删除: sn={sn}");

        Ok(())
    }

    /// 更新白名单条目的权限和描述。
    ///
    /// 参数:
    /// - `sn`: 目标设备序列号，不能为空。
    /// - `permission`: 新的权限值。
    /// - `description`: 新的描述（传 `None` 表示清空）。
    ///
    /// 返回:
    /// - 成功时返回 `()`；
    /// - 序列号为空时返回 [`WhitelistError::SerialNumberEmpty`]；
    /// - 条目不存在时返回 [`WhitelistError::NotFound`]；
    /// - 存储层失败时返回 [`WhitelistError::Storage`]。
    pub fn update(
        &self,
        sn: &str,
        permission: Option<i32>,
        description: Option<&str>,
    ) -> Result<(), WhitelistError> {
        let _mutation = self.lock_mutations()?;
        if sn.is_empty() {
            return Err(WhitelistError::SerialNumberEmpty);
        }
        let mut cache = self.lock_cache_for_mutation()?;

        let item = self
            .storage
            .whitelist_query_by_sn(sn)?
            .ok_or_else(|| WhitelistError::NotFound(sn.to_string()))?;

        let final_permission = permission.unwrap_or(item.permission);

        self.storage
            .whitelist_update(item.id, final_permission, description)?;

        if let Some(entry) = cache.get_mut(sn) {
            entry.permission = final_permission;
        }
        debug!("白名单缓存更新: sn={sn}, permission={final_permission}");

        Ok(())
    }

    /// 查询所有白名单条目。
    ///
    /// 返回:
    /// - 成功时返回全部白名单条目列表；存储层失败时返回 [`WhitelistError`]。
    pub fn query_all(&self) -> Result<Vec<UsbWhitelist>, WhitelistError> {
        let items = self.storage.whitelist_query_all()?;
        Ok(items)
    }

    /// 按序列号查询白名单条目。
    ///
    /// 参数:
    /// - `sn`: 目标设备序列号，不能为空。
    ///
    /// 返回:
    /// - 存在时返回 `Some(UsbWhitelist)`；不存在时返回 `None`；
    ///   存储层失败时返回 [`WhitelistError`]。
    pub fn query_by_sn(&self, sn: &str) -> Result<Option<UsbWhitelist>, WhitelistError> {
        trace!(sn = %sn, "白名单查询（数据库）");
        if sn.is_empty() {
            return Err(WhitelistError::SerialNumberEmpty);
        }
        let item = self.storage.whitelist_query_by_sn(sn)?;
        Ok(item)
    }

    /// 获取 Storage 引用（供协议层直接访问存储）。
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    fn lock_mutations(&self) -> Result<MutexGuard<'_, ()>, WhitelistError> {
        match self.mutation_lock.lock() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => {
                warn!("白名单变更协调锁中毒，拒绝执行变更");
                drop(poisoned.into_inner());
                Err(WhitelistError::Internal(
                    "白名单变更状态异常，需全量重载后恢复".into(),
                ))
            }
        }
    }

    fn lock_cache_for_mutation(
        &self,
    ) -> Result<RwLockWriteGuard<'_, HashMap<String, CacheEntry>>, WhitelistError> {
        match self.cache.write() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => {
                warn!("白名单缓存锁中毒，拒绝执行变更");
                drop(poisoned.into_inner());
                Err(WhitelistError::Internal(
                    "白名单缓存状态异常，需全量重载后恢复".into(),
                ))
            }
        }
    }

    fn replace_cache(
        cache: &mut HashMap<String, CacheEntry>,
        snapshot: Vec<WhitelistCacheSnapshotEntry>,
    ) {
        let replacement = snapshot
            .into_iter()
            .map(|item| {
                (
                    item.serial_number,
                    CacheEntry {
                        id: item.id,
                        permission: item.permission,
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        *cache = replacement;
        info!("白名单缓存已原子替换，共 {} 条", cache.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use tempfile::NamedTempFile;

    /// 创建临时数据库并返回 WhitelistManager。
    fn make_manager() -> (NamedTempFile, WhitelistManager) {
        let tmp = NamedTempFile::new().unwrap();
        let storage = Arc::new(Storage::open(tmp.path()).unwrap());
        let manager = WhitelistManager::new(storage).unwrap();
        (tmp, manager)
    }

    /// 构造测试用 AddWhitelistRequest。
    fn make_request(sn: &str) -> AddWhitelistRequest {
        AddWhitelistRequest {
            serial_number: sn.to_string(),
            vid: Some("0951".to_string()),
            pid: Some("1666".to_string()),
            device_name: Some("测试设备".to_string()),
            capacity_bytes: Some(32 * 1024 * 1024 * 1024),
            device_type: "storage".to_string(),
            description: Some("测试描述".to_string()),
            permission: 1,
            add_method: 0,
        }
    }

    #[test]
    fn add_and_query_whitelist_success() {
        let (_tmp, manager) = make_manager();
        let id = manager.add(make_request("SN001")).unwrap();
        assert!(id > 0);

        let result = manager.is_whitelisted("SN001");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.id, id);
        assert_eq!(result.permission, 1);
    }

    #[test]
    fn add_empty_serial_number_returns_error() {
        let (_tmp, manager) = make_manager();
        let err = manager.add(make_request("")).unwrap_err();
        assert!(matches!(err, WhitelistError::SerialNumberEmpty));
    }

    #[test]
    fn add_duplicate_serial_number_returns_already_exists() {
        let (_tmp, manager) = make_manager();
        manager.add(make_request("SN002")).unwrap();
        let err = manager.add(make_request("SN002")).unwrap_err();
        assert!(matches!(err, WhitelistError::AlreadyExists(_)));
    }

    #[test]
    fn remove_existing_entry_success() {
        let (_tmp, manager) = make_manager();
        manager.add(make_request("SN003")).unwrap();
        manager.remove("SN003").unwrap();
        assert!(manager.is_whitelisted("SN003").is_none());
    }

    #[test]
    fn remove_nonexistent_entry_returns_not_found() {
        let (_tmp, manager) = make_manager();
        let err = manager.remove("NONEXISTENT").unwrap_err();
        assert!(matches!(err, WhitelistError::NotFound(_)));
    }

    #[test]
    fn update_permission_success() {
        let (_tmp, manager) = make_manager();
        manager.add(make_request("SN004")).unwrap();
        manager.update("SN004", Some(2), Some("新描述")).unwrap();

        let result = manager.is_whitelisted("SN004").unwrap();
        assert_eq!(result.permission, 2);
    }

    #[test]
    fn update_nonexistent_entry_returns_not_found() {
        let (_tmp, manager) = make_manager();
        let err = manager.update("NONEXISTENT", Some(2), None).unwrap_err();
        assert!(matches!(err, WhitelistError::NotFound(_)));
    }

    #[test]
    fn query_all_returns_all_entries() {
        let (_tmp, manager) = make_manager();
        manager.add(make_request("SN005")).unwrap();
        manager.add(make_request("SN006")).unwrap();
        let items = manager.query_all().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn query_by_sn_returns_correct_entry() {
        let (_tmp, manager) = make_manager();
        manager.add(make_request("SN007")).unwrap();
        let item = manager.query_by_sn("SN007").unwrap();
        assert!(item.is_some());
        assert_eq!(item.unwrap().serial_number, "SN007");
    }

    #[test]
    fn query_by_sn_empty_returns_error() {
        let (_tmp, manager) = make_manager();
        let err = manager.query_by_sn("").unwrap_err();
        assert!(matches!(err, WhitelistError::SerialNumberEmpty));
    }

    #[test]
    fn is_whitelisted_returns_none_for_unknown_sn() {
        let (_tmp, manager) = make_manager();
        assert!(manager.is_whitelisted("UNKNOWN").is_none());
    }

    #[test]
    fn reload_cache_repopulates_from_db() {
        let tmp = NamedTempFile::new().unwrap();
        let storage = Arc::new(Storage::open(tmp.path()).unwrap());
        let manager = WhitelistManager::new(storage).unwrap();

        manager.add(make_request("SN008")).unwrap();

        // 清空缓存再重建
        {
            let mut cache = manager.cache.write().unwrap();
            cache.clear();
        }
        assert!(manager.is_whitelisted("SN008").is_none());

        manager.reload_cache().unwrap();
        assert!(manager.is_whitelisted("SN008").is_some());
    }

    #[test]
    fn policy_import_and_add_share_the_same_mutation_lock() {
        let (_tmp, manager) = make_manager();
        let manager = Arc::new(manager);
        let (import_entered_tx, import_entered_rx) = mpsc::channel();
        let (release_import_tx, release_import_rx) = mpsc::channel();
        let import_manager = Arc::clone(&manager);
        let import_thread = thread::spawn(move || {
            import_manager
                .coordinate_policy_import(|| {
                    import_entered_tx.send(()).unwrap();
                    release_import_rx.recv().unwrap();
                    Ok(Vec::new())
                })
                .unwrap();
        });
        import_entered_rx.recv().unwrap();

        let (add_started_tx, add_started_rx) = mpsc::channel();
        let (add_done_tx, add_done_rx) = mpsc::channel();
        let add_manager = Arc::clone(&manager);
        let add_thread = thread::spawn(move || {
            add_started_tx.send(()).unwrap();
            let result = add_manager.add(make_request("SERIALIZED_ADD"));
            add_done_tx.send(result).unwrap();
        });
        add_started_rx.recv().unwrap();
        assert!(matches!(
            add_done_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        release_import_tx.send(()).unwrap();
        import_thread.join().unwrap();
        assert!(add_done_rx.recv().unwrap().is_ok());
        add_thread.join().unwrap();
        assert!(manager.is_whitelisted("SERIALIZED_ADD").is_some());
    }

    #[test]
    fn policy_import_blocks_authorization_reads_until_cache_snapshot_is_swapped() {
        let (_tmp, manager) = make_manager();
        let manager = Arc::new(manager);
        manager.add(make_request("A")).unwrap();
        let (committed_tx, committed_rx) = mpsc::channel();
        let (allow_swap_tx, allow_swap_rx) = mpsc::channel();
        let imported_whitelist = vec![UsbWhitelistInsert {
            serial_number: "B".into(),
            vid: None,
            pid: None,
            device_name: None,
            capacity_bytes: Some(1024),
            device_type: "storage".into(),
            description: None,
            permission: 1,
            add_method: 0,
        }];
        let imported_policies = vec![
            ("exec_control".into(), 0),
            ("auto_read_control".into(), 0),
            ("file_type_blacklist_control".into(), 0),
        ];
        let import_manager = Arc::clone(&manager);
        let import_thread = thread::spawn(move || {
            import_manager
                .coordinate_policy_import(|| {
                    let snapshot = import_manager.storage.policy_import_transaction(
                        &imported_whitelist,
                        &imported_policies,
                        &[],
                    )?;
                    committed_tx.send(()).unwrap();
                    allow_swap_rx.recv().unwrap();
                    Ok(snapshot)
                })
                .unwrap();
        });
        committed_rx.recv().unwrap();

        let (read_done_tx, read_done_rx) = mpsc::channel();
        let read_manager = Arc::clone(&manager);
        let read_thread = thread::spawn(move || {
            read_done_tx.send(read_manager.is_whitelisted("A")).unwrap();
        });
        assert!(matches!(
            read_done_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        assert!(manager
            .storage
            .whitelist_query_by_sn("A")
            .unwrap()
            .is_none());
        assert!(manager
            .storage
            .whitelist_query_by_sn("B")
            .unwrap()
            .is_some());

        allow_swap_tx.send(()).unwrap();
        import_thread.join().unwrap();
        assert!(read_done_rx.recv().unwrap().is_none());
        read_thread.join().unwrap();

        assert!(manager.is_whitelisted("A").is_none());
        assert!(manager.is_whitelisted("B").is_some());
    }

    #[test]
    fn failed_policy_import_keeps_existing_cache() {
        let (_tmp, manager) = make_manager();
        manager.add(make_request("A")).unwrap();

        let result = manager.coordinate_policy_import(|| {
            Err::<Vec<WhitelistCacheSnapshotEntry>, _>(StorageError::Validation(
                "injected import failure".into(),
            ))
        });

        assert!(matches!(result, Err(WhitelistError::Storage(_))));
        assert!(manager.is_whitelisted("A").is_some());
    }

    #[test]
    fn poisoned_mutation_lock_denies_authorization_and_mutations_until_reload() {
        let (_tmp, manager) = make_manager();
        let manager = Arc::new(manager);
        manager.add(make_request("A")).unwrap();
        let poison_manager = Arc::clone(&manager);
        let _ = thread::spawn(move || {
            let _mutation = poison_manager.mutation_lock.lock().unwrap();
            panic!("poison mutation lock for test");
        })
        .join();

        assert!(manager.is_whitelisted("A").is_none());
        assert!(matches!(
            manager.add(make_request("B")),
            Err(WhitelistError::Internal(_))
        ));
        assert!(matches!(
            manager.remove("A"),
            Err(WhitelistError::Internal(_))
        ));
        assert!(matches!(
            manager.update("A", Some(0), None),
            Err(WhitelistError::Internal(_))
        ));
        assert!(matches!(
            manager.coordinate_policy_import(|| {
                Ok::<Vec<WhitelistCacheSnapshotEntry>, StorageError>(Vec::new())
            }),
            Err(WhitelistError::Internal(_))
        ));

        manager.reload_cache().unwrap();
        assert!(manager.is_whitelisted("A").is_some());
        assert!(manager.add(make_request("B")).is_ok());
    }
}
