//! T05 USB 审计日志表 CRUD。

use rusqlite::params;

use crate::error::StorageError;
use crate::model::{UsbAuditLog, UsbAuditLogInsert};
use crate::Storage;

impl Storage {
    /// 插入 USB 审计日志。
    pub fn usb_audit_insert(&self, item: &UsbAuditLogInsert) -> Result<i64, StorageError> {
        if item.event_type.is_empty() {
            return Err(StorageError::Validation("event_type 不能为空".into()));
        }
        if item.result.is_empty() {
            return Err(StorageError::Validation("result 不能为空".into()));
        }
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO usb_audit_log (event_time, device_type, interface_type, interface_class, interface_subclass, interface_protocol, device_name, device_sn, vid, pid, event_type, permission, capacity_bytes, file_path, matched_policy, result, fail_reason, detail) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    item.event_time, item.device_type, item.interface_type,
                    item.interface_class, item.interface_subclass, item.interface_protocol,
                    item.device_name, item.device_sn, item.vid, item.pid,
                    item.event_type, item.permission, item.capacity_bytes,
                    item.file_path, item.matched_policy, item.result,
                    item.fail_reason, item.detail,
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 按时间范围查询 USB 审计日志。
    pub fn usb_audit_query_by_time(&self, from: i64, to: i64) -> Result<Vec<UsbAuditLog>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, event_time, device_type, interface_type, interface_class, interface_subclass, interface_protocol, device_name, device_sn, vid, pid, event_type, permission, capacity_bytes, file_path, matched_policy, result, fail_reason, detail \
                 FROM usb_audit_log WHERE event_time >= ?1 AND event_time <= ?2 ORDER BY event_time DESC",
            )?;
            let rows = stmt.query_map(params![from, to], |row| {
                Ok(UsbAuditLog {
                    id: row.get(0)?, event_time: row.get(1)?, device_type: row.get(2)?,
                    interface_type: row.get(3)?, interface_class: row.get(4)?,
                    interface_subclass: row.get(5)?, interface_protocol: row.get(6)?,
                    device_name: row.get(7)?, device_sn: row.get(8)?,
                    vid: row.get(9)?, pid: row.get(10)?, event_type: row.get(11)?,
                    permission: row.get(12)?, capacity_bytes: row.get(13)?,
                    file_path: row.get(14)?, matched_policy: row.get(15)?,
                    result: row.get(16)?, fail_reason: row.get(17)?, detail: row.get(18)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(StorageError::Sqlite)
        })
    }

    /// 删除最旧的一条 USB 审计日志，返回被删除记录的时间。
    pub fn usb_audit_delete_oldest(&self) -> Result<Option<i64>, StorageError> {
        self.pool().with_transaction(|tx| {
            let result: Result<i64, _> = tx.query_row(
                "DELETE FROM usb_audit_log WHERE id = (SELECT id FROM usb_audit_log ORDER BY event_time ASC LIMIT 1) RETURNING event_time",
                [], |row| row.get(0),
            );
            match result {
                Ok(time) => Ok(Some(time)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }

    /// USB 审计日志总数。
    pub fn usb_audit_count(&self) -> Result<i64, StorageError> {
        self.pool().with_read(|conn| {
            let count = conn.query_row("SELECT COUNT(*) FROM usb_audit_log", [], |row| row.get(0))?;
            Ok(count)
        })
    }
}
