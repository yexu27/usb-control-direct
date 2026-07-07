//! USB 受控设备运行态登记中心。
//!
//! 本 crate 只维护管理端可查询的 storage / keyboard / mouse 运行态快照，
//! 不执行 mount、scan、NBD、gadget 或 HID 转发等业务动作。

use std::collections::HashMap;
use std::sync::RwLock;

/// 单个受控 USB interface 的运行态快照。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceRuntimeSnapshot {
    pub runtime_id: String,
    pub parent_path: String,
    pub interface_path: String,
    pub serial_number: String,
    pub device_name: String,
    pub device_type: String,
    pub interface_type: String,
    pub status: String,
    pub stage: String,
    pub fail_code: String,
    pub fail_reason: String,
    pub connected_at: i64,
    pub updated_at: i64,
}

/// 创建运行态记录的输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceRuntimeCreate {
    pub runtime_id: String,
    pub parent_path: String,
    pub interface_path: String,
    pub serial_number: String,
    pub device_name: String,
    pub device_type: String,
    pub interface_type: String,
    pub status: String,
    pub stage: String,
    pub fail_code: String,
    pub fail_reason: String,
}

/// 更新运行态记录的输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceRuntimeUpdate {
    pub status: String,
    pub stage: String,
    pub fail_code: String,
    pub fail_reason: String,
}

/// 线程安全的受控设备运行态登记中心。
#[derive(Debug, Default)]
pub struct DeviceRuntimeRegistry {
    entries: RwLock<HashMap<String, DeviceRuntimeSnapshot>>,
}

impl DeviceRuntimeRegistry {
    /// 创建空的运行态登记中心。
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建或覆盖一个运行态记录。
    pub fn create(&self, input: DeviceRuntimeCreate) {
        let now = common::time::now_unix();
        let snapshot = DeviceRuntimeSnapshot {
            runtime_id: input.runtime_id.clone(),
            parent_path: input.parent_path,
            interface_path: input.interface_path,
            serial_number: input.serial_number,
            device_name: input.device_name,
            device_type: input.device_type,
            interface_type: input.interface_type,
            status: input.status,
            stage: input.stage,
            fail_code: input.fail_code,
            fail_reason: input.fail_reason,
            connected_at: now,
            updated_at: now,
        };

        let mut entries = self.entries.write().expect("device runtime lock poisoned");
        entries.insert(input.runtime_id, snapshot);
    }

    /// 更新已有运行态记录。
    ///
    /// 如果 `runtime_id` 不存在，本函数不创建新记录。调用方必须在设备纳入
    /// 受控链路时显式调用 `create`。
    pub fn update(&self, runtime_id: &str, update: DeviceRuntimeUpdate) {
        let mut entries = self.entries.write().expect("device runtime lock poisoned");
        if let Some(snapshot) = entries.get_mut(runtime_id) {
            snapshot.status = update.status;
            snapshot.stage = update.stage;
            snapshot.fail_code = update.fail_code;
            snapshot.fail_reason = update.fail_reason;
            snapshot.updated_at = common::time::now_unix();
        }
    }

    /// 将已有运行态记录标记为已移除。
    pub fn mark_removed(&self, runtime_id: &str, reason: impl Into<String>) {
        self.update(
            runtime_id,
            DeviceRuntimeUpdate {
                status: "removed".to_string(),
                stage: "cleanup".to_string(),
                fail_code: "device_removed".to_string(),
                fail_reason: reason.into(),
            },
        );
    }

    /// 返回全部运行态快照，按更新时间倒序排列。
    pub fn list(&self) -> Vec<DeviceRuntimeSnapshot> {
        let entries = self.entries.read().expect("device runtime lock poisoned");
        let mut snapshots = entries.values().cloned().collect::<Vec<_>>();
        snapshots.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        snapshots
    }

    /// 根据运行态 ID 返回快照。
    pub fn get(&self, runtime_id: &str) -> Option<DeviceRuntimeSnapshot> {
        let entries = self.entries.read().expect("device runtime lock poisoned");
        entries.get(runtime_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_input(runtime_id: &str) -> DeviceRuntimeCreate {
        DeviceRuntimeCreate {
            runtime_id: runtime_id.to_string(),
            parent_path: "/sys/devices/usb1/1-1".to_string(),
            interface_path: "/sys/devices/usb1/1-1:1.0".to_string(),
            serial_number: "SN001".to_string(),
            device_name: "Cruzer Blade".to_string(),
            device_type: "storage".to_string(),
            interface_type: "mass_storage".to_string(),
            status: "accepted".to_string(),
            stage: "admission".to_string(),
            fail_code: String::new(),
            fail_reason: String::new(),
        }
    }

    #[test]
    fn create_and_list_runtime_snapshot() {
        let registry = DeviceRuntimeRegistry::new();
        registry.create(create_input("runtime-1"));

        let snapshots = registry.list();

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].runtime_id, "runtime-1");
        assert_eq!(snapshots[0].status, "accepted");
        assert_eq!(snapshots[0].stage, "admission");
        assert!(snapshots[0].connected_at > 0);
        assert_eq!(snapshots[0].connected_at, snapshots[0].updated_at);
    }

    #[test]
    fn update_existing_snapshot() {
        let registry = DeviceRuntimeRegistry::new();
        registry.create(create_input("runtime-1"));

        registry.update(
            "runtime-1",
            DeviceRuntimeUpdate {
                status: "processing".to_string(),
                stage: "scan".to_string(),
                fail_code: String::new(),
                fail_reason: String::new(),
            },
        );

        let snapshot = registry.get("runtime-1").unwrap();
        assert_eq!(snapshot.status, "processing");
        assert_eq!(snapshot.stage, "scan");
        assert!(snapshot.updated_at >= snapshot.connected_at);
    }

    #[test]
    fn update_missing_snapshot_is_noop() {
        let registry = DeviceRuntimeRegistry::new();

        registry.update(
            "missing",
            DeviceRuntimeUpdate {
                status: "failed".to_string(),
                stage: "scan".to_string(),
                fail_code: "scan_failed".to_string(),
                fail_reason: "扫描失败".to_string(),
            },
        );

        assert!(registry.list().is_empty());
    }

    #[test]
    fn mark_removed_updates_existing_snapshot() {
        let registry = DeviceRuntimeRegistry::new();
        registry.create(create_input("runtime-1"));

        registry.mark_removed("runtime-1", "设备已拔出");

        let snapshot = registry.get("runtime-1").unwrap();
        assert_eq!(snapshot.status, "removed");
        assert_eq!(snapshot.stage, "cleanup");
        assert_eq!(snapshot.fail_code, "device_removed");
        assert_eq!(snapshot.fail_reason, "设备已拔出");
    }
}
