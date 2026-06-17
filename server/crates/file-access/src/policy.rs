//! 5 级优先级策略引擎。
//!
//! L1=病毒标记 → L2=可执行控制 → L3=类型黑名单 → L4=自运行控制 → L5=写保护
//! 短路求值：任一级别命中即返回 Deny。

use crate::types::{AccessDecision, ControlledEntry, PolicySnapshot};

/// 评估文件的访问决策。
///
/// 参数:
///   - entry: 受控文件树节点。
///   - snapshot: 策略快照。
///
/// 返回:
///   - AccessDecision::Allow 或 AccessDecision::Deny(命中策略描述)。
pub fn evaluate_access(entry: &ControlledEntry, snapshot: &PolicySnapshot) -> AccessDecision {
    // 目录始终允许访问
    if entry.is_dir {
        return AccessDecision::Allow;
    }

    // L1: 病毒标记（无开关，始终生效）
    if entry.is_virus {
        return AccessDecision::Deny("L1:病毒文件禁止访问".to_string());
    }

    // L2: 可执行控制
    if snapshot.exec_control_enabled {
        if let Some(exec_type) = &entry.exec_type {
            return AccessDecision::Deny(format!(
                "L2:可执行文件控制({:?})",
                exec_type
            ));
        }
    }

    // L3: 文件类型黑名单
    if snapshot.file_type_blacklist_enabled
        && !entry.extension.is_empty()
        && snapshot.blacklist_extensions.contains(&entry.extension)
    {
        return AccessDecision::Deny(format!(
            "L3:文件类型黑名单({})",
            entry.extension
        ));
    }

    // L4: 自运行控制
    if snapshot.auto_read_control_enabled {
        if entry.is_autorun_inf {
            return AccessDecision::Deny("L4:自运行控制(autorun.inf)".to_string());
        }
        if entry.is_autorun_target {
            return AccessDecision::Deny("L4:自运行控制(autorun引用文件)".to_string());
        }
        if entry.is_root_shell_script {
            return AccessDecision::Deny("L4:自运行控制(根目录shell脚本)".to_string());
        }
    }

    // L5: 写保护（通过 f_mass_storage ro=1 在 gadget 层面实现，不在此处判断）
    AccessDecision::Allow
}

/// 从 Storage 加载策略快照。
///
/// 参数:
///   - storage: 数据库实例。
///   - permission: 白名单权限（0=只读，1=读写）。
///
/// 返回:
///   - PolicySnapshot。
pub fn load_policy_snapshot(storage: &storage::Storage, permission: i32) -> PolicySnapshot {
    let exec_control_enabled = storage
        .policy_query("exec_control")
        .ok()
        .flatten()
        .map(|p| p.enabled == 1)
        .unwrap_or(false);

    let file_type_blacklist_enabled = storage
        .policy_query("file_type_blacklist_control")
        .ok()
        .flatten()
        .map(|p| p.enabled == 1)
        .unwrap_or(false);

    let auto_read_control_enabled = storage
        .policy_query("auto_read_control")
        .ok()
        .flatten()
        .map(|p| p.enabled == 1)
        .unwrap_or(false);

    let blacklist_extensions = storage
        .blacklist_query_all()
        .unwrap_or_default()
        .into_iter()
        .map(|item| item.extension.to_lowercase())
        .collect();

    PolicySnapshot {
        exec_control_enabled,
        file_type_blacklist_enabled,
        auto_read_control_enabled,
        blacklist_extensions,
        permission,
    }
}
