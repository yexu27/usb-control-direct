//! 白名单管理服务。
//!
//! 提供白名单条目的增删改查接口，内置序列号索引缓存以加速插入许可判断。
//! 缓存在构造时从数据库全量加载，后续随写操作同步更新，无需定期刷新。

use std::collections::HashMap;
use std::sync::RwLock;

use storage::model::{UsbWhitelist, UsbWhitelistInsert};
use storage::StorageError;
use storage::Storage;
use tracing::{debug, info, warn};

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
    storage: Storage,
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
    pub fn new(storage: Storage) -> Result<Self, WhitelistError> {
        let manager = WhitelistManager {
            storage,
            cache: RwLock::new(HashMap::new()),
        };
        manager.reload_cache()?;
        Ok(manager)
    }

    /// 从数据库全量重建序列号缓存。
    ///
    /// 返回:
    /// - 重建成功时返回 `()`；数据库读取失败时返回 [`WhitelistError`]。
    pub fn reload_cache(&self) -> Result<(), WhitelistError> {
        let items = self.storage.whitelist_query_all()?;
        let mut cache = self
            .cache
            .write()
            .map_err(|e| WhitelistError::Internal(format!("缓存写锁异常: {e}")))?;
        cache.clear();
        for item in items {
            cache.insert(
                item.serial_number.clone(),
                CacheEntry {
                    id: item.id,
                    permission: item.permission,
                },
            );
        }
        info!("白名单缓存已重建，共 {} 条", cache.len());
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
        let cache = self.cache.read().ok()?;
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
        if req.serial_number.is_empty() {
            return Err(WhitelistError::SerialNumberEmpty);
        }

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
        match self.cache.write() {
            Ok(mut cache) => {
                cache.insert(
                    sn.clone(),
                    CacheEntry {
                        id,
                        permission,
                    },
                );
                debug!("白名单缓存新增: sn={sn}, id={id}");
            }
            Err(e) => {
                warn!("白名单缓存写锁异常，缓存未同步: {e}");
            }
        }

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
        if sn.is_empty() {
            return Err(WhitelistError::SerialNumberEmpty);
        }

        let item = self
            .storage
            .whitelist_query_by_sn(sn)?
            .ok_or_else(|| WhitelistError::NotFound(sn.to_string()))?;

        self.storage.whitelist_delete(item.id)?;

        match self.cache.write() {
            Ok(mut cache) => {
                cache.remove(sn);
                debug!("白名单缓存删除: sn={sn}");
            }
            Err(e) => {
                warn!("白名单缓存写锁异常，缓存未同步: {e}");
            }
        }

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
        permission: i32,
        description: Option<&str>,
    ) -> Result<(), WhitelistError> {
        if sn.is_empty() {
            return Err(WhitelistError::SerialNumberEmpty);
        }

        let item = self
            .storage
            .whitelist_query_by_sn(sn)?
            .ok_or_else(|| WhitelistError::NotFound(sn.to_string()))?;

        self.storage
            .whitelist_update(item.id, permission, description)?;

        match self.cache.write() {
            Ok(mut cache) => {
                if let Some(entry) = cache.get_mut(sn) {
                    entry.permission = permission;
                }
                debug!("白名单缓存更新: sn={sn}, permission={permission}");
            }
            Err(e) => {
                warn!("白名单缓存写锁异常，缓存未同步: {e}");
            }
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    /// 创建临时数据库并返回 WhitelistManager。
    fn make_manager() -> (NamedTempFile, WhitelistManager) {
        let tmp = NamedTempFile::new().unwrap();
        let storage = Storage::open(tmp.path()).unwrap();
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
        manager.update("SN004", 2, Some("新描述")).unwrap();

        let result = manager.is_whitelisted("SN004").unwrap();
        assert_eq!(result.permission, 2);
    }

    #[test]
    fn update_nonexistent_entry_returns_not_found() {
        let (_tmp, manager) = make_manager();
        let err = manager.update("NONEXISTENT", 2, None).unwrap_err();
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
        let storage = Storage::open(tmp.path()).unwrap();
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
}
