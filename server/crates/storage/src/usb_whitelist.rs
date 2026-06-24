//! T01 白名单表 CRUD。

use rusqlite::params;
use tracing::debug;

use crate::error::StorageError;
use crate::model::{UsbWhitelist, UsbWhitelistInsert};
use crate::Storage;

impl Storage {
    /// 插入白名单条目。
    pub fn whitelist_insert(&self, item: &UsbWhitelistInsert) -> Result<i64, StorageError> {
        if item.serial_number.is_empty() {
            return Err(StorageError::Validation("serial_number 不能为空".into()));
        }
        debug!(sn = %item.serial_number, "插入白名单设备");
        let now = crate::now_unix();
        self.pool().with_transaction(|tx| {
            tx.execute(
                "INSERT INTO usb_whitelist (serial_number, vid, pid, device_name, capacity_bytes, device_type, description, permission, add_method, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    item.serial_number, item.vid, item.pid, item.device_name,
                    item.capacity_bytes, item.device_type, item.description,
                    item.permission, item.add_method, now,
                ],
            )?;
            let id = tx.last_insert_rowid();
            Ok(id)
        })
    }

    /// 按序列号查询白名单。
    pub fn whitelist_query_by_sn(&self, serial_number: &str) -> Result<Option<UsbWhitelist>, StorageError> {
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, serial_number, vid, pid, device_name, capacity_bytes, device_type, description, permission, add_method, created_at \
                 FROM usb_whitelist WHERE serial_number = ?1",
            )?;
            let result = stmt.query_row(params![serial_number], |row| {
                Ok(UsbWhitelist {
                    id: row.get(0)?,
                    serial_number: row.get(1)?,
                    vid: row.get(2)?,
                    pid: row.get(3)?,
                    device_name: row.get(4)?,
                    capacity_bytes: row.get(5)?,
                    device_type: row.get(6)?,
                    description: row.get(7)?,
                    permission: row.get(8)?,
                    add_method: row.get(9)?,
                    created_at: row.get(10)?,
                })
            });
            match result {
                Ok(item) => Ok(Some(item)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(StorageError::Sqlite(e)),
            }
        })
    }

    /// 查询全部白名单。
    pub fn whitelist_query_all(&self) -> Result<Vec<UsbWhitelist>, StorageError> {
        debug!("查询白名单全量列表");
        self.pool().with_read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, serial_number, vid, pid, device_name, capacity_bytes, device_type, description, permission, add_method, created_at \
                 FROM usb_whitelist ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(UsbWhitelist {
                    id: row.get(0)?,
                    serial_number: row.get(1)?,
                    vid: row.get(2)?,
                    pid: row.get(3)?,
                    device_name: row.get(4)?,
                    capacity_bytes: row.get(5)?,
                    device_type: row.get(6)?,
                    description: row.get(7)?,
                    permission: row.get(8)?,
                    add_method: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })?;
            let mut items = Vec::new();
            for row in rows {
                items.push(row?);
            }
            Ok(items)
        })
    }

    /// 更新白名单条目（权限和描述）。
    pub fn whitelist_update(&self, id: i64, permission: i32, description: Option<&str>) -> Result<(), StorageError> {
        debug!(id = id, permission = permission, "更新白名单设备");
        self.pool().with_transaction(|tx| {
            let affected = tx.execute(
                "UPDATE usb_whitelist SET permission = ?1, description = ?2 WHERE id = ?3",
                params![permission, description, id],
            )?;
            if affected == 0 {
                return Err(StorageError::NotFound(format!("whitelist id={}", id)));
            }
            Ok(())
        })
    }

    /// 删除白名单条目。
    pub fn whitelist_delete(&self, id: i64) -> Result<(), StorageError> {
        debug!(id = id, "删除白名单设备");
        self.pool().with_transaction(|tx| {
            let affected = tx.execute("DELETE FROM usb_whitelist WHERE id = ?1", params![id])?;
            if affected == 0 {
                return Err(StorageError::NotFound(format!("whitelist id={}", id)));
            }
            Ok(())
        })
    }

    /// 白名单总数。
    pub fn whitelist_count(&self) -> Result<i64, StorageError> {
        self.pool().with_read(|conn| {
            let count = conn.query_row("SELECT COUNT(*) FROM usb_whitelist", [], |row| row.get(0))?;
            Ok(count)
        })
    }
}
