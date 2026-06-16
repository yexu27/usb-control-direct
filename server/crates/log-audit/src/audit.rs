//! 三类日志写入接口 + 80% 滚动覆盖。
//!
//! 时间戳由 audit 层注入，不依赖调用方。

use std::path::Path;

use storage::model::{
    LogRetentionEventInsert, MalwareLogInsert, OperationLogInsert, UsbAuditLogInsert,
};
use storage::Storage;

use crate::error::AuditError;

/// 存储使用率阈值（80%）。
const STORAGE_THRESHOLD_PERCENT: u64 = 80;

/// 日志类别标识。
#[derive(Debug, Clone, Copy)]
pub enum LogCategory {
    /// USB 审计日志。
    UsbAudit,
    /// 恶意代码检测日志。
    Malware,
    /// 操作日志。
    Operation,
}

impl LogCategory {
    /// 转为 T11 的 log_category 字符串。
    fn as_str(&self) -> &'static str {
        match self {
            LogCategory::UsbAudit => "usb_audit",
            LogCategory::Malware => "malware",
            LogCategory::Operation => "operation",
        }
    }
}

/// 获取装置整体存储使用率。
fn get_storage_usage_percent(db_path: &Path) -> Result<u64, AuditError> {
    let stat = nix_statvfs(db_path)?;
    if stat.total == 0 {
        return Ok(0);
    }
    let used = stat.total - stat.available;
    Ok(used * 100 / stat.total)
}

struct DiskStat {
    total: u64,
    available: u64,
}

/// 使用 libc::statvfs 获取磁盘空间信息。
fn nix_statvfs(path: &Path) -> Result<DiskStat, AuditError> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    // 安全性: c_path 是有效的 NUL 结尾字符串，stat 已零初始化。
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(DiskStat {
            total: stat.f_blocks as u64 * stat.f_frsize as u64,
            available: stat.f_bavail as u64 * stat.f_frsize as u64,
        })
    }
}

/// 日志审计服务。
pub struct AuditService {
    storage: Storage,
    db_path: std::path::PathBuf,
}

impl AuditService {
    /// 创建审计服务。
    pub fn new(storage: Storage, db_path: &Path) -> Self {
        AuditService {
            storage,
            db_path: db_path.to_path_buf(),
        }
    }

    /// 获取 Storage 引用（供外部使用）。
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// 写入 USB 审计日志（T05）。
    pub fn log_usb_audit(&self, item: &mut UsbAuditLogInsert) -> Result<i64, AuditError> {
        item.event_time = now_unix();
        self.maybe_overwrite(LogCategory::UsbAudit)?;
        let id = self.storage.usb_audit_insert(item)?;
        Ok(id)
    }

    /// 写入恶意代码检测日志（T06）。
    pub fn log_malware(&self, item: &mut MalwareLogInsert) -> Result<i64, AuditError> {
        item.scan_time = now_unix();
        self.maybe_overwrite(LogCategory::Malware)?;
        let id = self.storage.malware_insert(item)?;
        Ok(id)
    }

    /// 写入操作日志（T10）。
    pub fn log_operation(&self, item: &mut OperationLogInsert) -> Result<i64, AuditError> {
        item.op_time = now_unix();
        self.maybe_overwrite(LogCategory::Operation)?;
        let id = self.storage.operation_log_insert(item)?;
        Ok(id)
    }

    /// 检查存储使用率，超过 80% 时删除当前类别最旧 1 条日志。
    fn maybe_overwrite(&self, category: LogCategory) -> Result<(), AuditError> {
        let usage = get_storage_usage_percent(&self.db_path)?;
        if usage < STORAGE_THRESHOLD_PERCENT {
            return Ok(());
        }

        let now = now_unix();
        let deleted_time = match category {
            LogCategory::UsbAudit => self.storage.usb_audit_delete_oldest()?,
            LogCategory::Malware => self.storage.malware_delete_oldest()?,
            LogCategory::Operation => self.storage.operation_log_delete_oldest()?,
        };

        match deleted_time {
            Some(time) => {
                let retention = LogRetentionEventInsert {
                    trigger_time: now,
                    log_category: category.as_str().into(),
                    storage_usage_percent: usage as i32,
                    covered_from_time: Some(time),
                    covered_to_time: Some(time),
                    covered_count: 1,
                    result: 0,
                    fail_reason: None,
                };
                self.storage.retention_event_insert(&retention)?;
                Ok(())
            }
            None => {
                let retention = LogRetentionEventInsert {
                    trigger_time: now,
                    log_category: category.as_str().into(),
                    storage_usage_percent: usage as i32,
                    covered_from_time: None,
                    covered_to_time: None,
                    covered_count: 0,
                    result: 1,
                    fail_reason: Some("当前类别无可覆盖日志".into()),
                };
                self.storage.retention_event_insert(&retention)?;
                Err(AuditError::StorageFull)
            }
        }
    }
}

/// 获取当前 Unix 时间戳（秒）。
fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
