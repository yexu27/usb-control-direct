# Orchestrator 模块集成 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 ScanService、FileAccessEngine、KeyboardInterceptor、MouseForwarder、HID Gadget 接入 DeviceOrchestrator，补全 handler 骨架——当前 handler 仅写审计日志，未调用已实现的模块。

**Architecture:** Orchestrator Event Loop 做快速路由（白名单/evdev发现/资源分配），长耗时操作 spawn 独立 task 执行。DeviceManager（协议层可查询）与 ActiveSession（Orchestrator 私有运行时追踪）两层分离，同键关联。

**推进顺序（按依赖和复杂度）：**
1. 前置依赖：GadgetManager pub 暴露、find_evdev_path
2. 简单独立 handler：handle_keyboard → handle_mouse
3. handle_removed 清理
4. 核心复杂 handler：handle_storage（mount → scan → file tree → NBD）
5. main.rs 串联

**Tech Stack:** Rust 1.96, tokio, evdev crate, Linux configfs

**基线设计文档:** `docs/superpowers/specs/2026-06-22-orchestrator-integration-design.md`

---

## 执行环境

| 环节 | 位置 | 命令 |
|---|---|---|
| **写代码** | 当前 Mac worktree | 直接编辑 |
| **同步到 VM** | Mac → Ubuntu VM | `rsync` 到 `/root/work/code/usb-control-direct` |
| **编译** | Ubuntu 18.04 VM（172.16.0.219:2222） | `ssh -p 2222 -i ~/.ssh/WinPC-Personal root@172.16.0.219` 后 `cd /root/work/code/usb-control-direct && cargo build --target aarch64-unknown-linux-gnu` |
| **运行验证** | RK3568（172.16.3.95） | `ssh -i ~/.ssh/WinPC-Test root@172.16.3.95` 推交叉编译产物 |

**VM 工程路径:** `/root/work/code/usb-control-direct`

---

## 涉及文件

```
server/crates/
├── file-access/src/gadget.rs         ← 改动: 内部方法 pub
├── usb-identify/src/orchestrator.rs  ← 主力改动: handler 逻辑 + find_evdev_path + ActiveSession
└── app/src/main.rs                   ← 改动: gadget_init + 服务注入
```

不改的文件（直接使用已有实现）：
- `hid-access/src/evdev_interceptor.rs` — KeyboardInterceptor
- `hid-access/src/mouse_forwarder.rs` — MouseForwarder
- `hid-access/src/hid_gadget.rs` — configure_hid_function + discover_hidg_nodes
- `hid-access/src/hid_report.rs` — KEYBOARD_REPORT_DESC/LEN, MOUSE_REPORT_DESC/LEN
- `usb-identify/src/traits.rs` — Scanner + DeviceMapper trait
- `usb-identify/src/mount.rs` — RealMountOps + mount_path_for
- `usb-identify/src/monitor.rs` — DeviceManager
- `usb-identify/src/udev_monitor.rs` — udev 监听

---

## Task 1: 暴露 GadgetManager 内部方法为 pub

**Files:** `server/crates/file-access/src/gadget.rs`

**目的:** `main.rs` 的 gadget_init 需要调用这些方法统一配置 mass_storage + HID 两个 function。当前为私有方法。

- [ ] **Step 1: 改为 pub 并新增 link_function**

```rust
// setup_gadget 改为 pub（原有实现不变）
pub fn setup_gadget(&self, device_description: &str) -> Result<(), std::io::Error> {
    // ... 原有实现 ...
}

// configure_mass_storage 改为 pub（去掉末尾 symlink 逻辑）
pub fn configure_mass_storage(&self, nbd_device: &Path, readonly: bool) -> Result<(), std::io::Error> {
    // ... 原有实现（去掉末尾的 symlink 部分）...
}

// 新增: 将 function 通过 symlink 链接到 config
pub fn link_function(&self, function_name: &str) -> Result<(), std::io::Error> {
    let function = self.gadget_path.join("functions").join(function_name);
    let link_target = self.gadget_path.join("configs/c.1").join(function_name);
    if !link_target.exists() {
        std::os::unix::fs::symlink(&function, &link_target)?;
    }
    Ok(())
}

// bind_udc 改为 pub
pub fn bind_udc(&self) -> Result<(), std::io::Error> {
    write_file(&self.gadget_path.join("UDC"), &self.udc_name)
}

// unbind_udc 改为 pub
pub fn unbind_udc(&self) -> Result<(), std::io::Error> {
    write_file(&self.gadget_path.join("UDC"), "")
}

// write_file 改为 pub(crate)
pub(crate) fn write_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    fs::write(path, content)
}
```

更新 `enable()` 内部调用：

```rust
pub fn enable(&mut self, nbd_device: &Path, readonly: bool, device_description: &str) -> Result<(), std::io::Error> {
    self.setup_gadget(device_description)?;
    self.configure_mass_storage(nbd_device, readonly)?;
    self.link_function("mass_storage.usb0")?;
    self.bind_udc()?;
    self.enabled = true;
    Ok(())
}
```

- [ ] **Step 2: 编译验证**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu -p file-access 2>&1
```

- [ ] **Step 3: Commit**

```bash
git add server/crates/file-access/src/gadget.rs
git commit -m "refactor(file-access): 暴露 GadgetManager 内部方法为 pub

- setup_gadget、configure_mass_storage、bind_udc、unbind_udc 改为 pub
- 新增 pub fn link_function，将 configfs symlink 逻辑独立
- write_file 改为 pub(crate)
- 为 main.rs 统一 configfs 初始化做准备"
```

---

## Task 2: 实现 find_evdev_path() + 重构 Orchestrator 结构

**Files:** `server/crates/usb-identify/src/orchestrator.rs`

**目的:** 两个前置依赖一起做——① find_evdev_path 供 keyboard/mouse handler 使用，② 重构结构体以注入 ScanService、FileAccessEngine、HidgNodes。

- [ ] **Step 1: 添加 imports**

在现有 imports 后追加：

```rust
use std::collections::HashMap;
use tokio::sync::watch;
use crate::mount::{dev_name_from_path, mount_path_for, RealMountOps};
```

- [ ] **Step 2: 添加 ActiveSession 结构体**

```rust
/// 活动设备会话——追踪后台 task 和运行时资源。
/// 与 DeviceManager 同键（parent device path），不同职责：
///   DeviceManager: 设备属性，协议层可查询
///   ActiveSession: 运行时资源，仅 Orchestrator 内部
struct ActiveSession {
    parent_path: String,
    device_type: common::types::DeviceType,
    nbd_index: Option<u32>,
    mount_path: Option<std::path::PathBuf>,
    cancel_tx: watch::Sender<bool>,
}
```

- [ ] **Step 3: 添加 find_evdev_path 函数**

```rust
/// 从 USB 接口 sysfs 路径查找对应的 evdev 设备节点。
///
/// 内核为 USB HID 设备创建 input 子设备：
///   /sys/devices/.../2-1.1:1.0/0003:.../input/input3/event3
fn find_evdev_path(usb_iface_syspath: &str) -> Option<std::path::PathBuf> {
    use std::fs;

    let iface_dir = std::path::Path::new(usb_iface_syspath);
    if !iface_dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(iface_dir).ok()?;
    for entry in entries.flatten() {
        let input_dir = entry.path().join("input");
        if !input_dir.is_dir() {
            continue;
        }
        let input_entries = fs::read_dir(&input_dir).ok()?;
        for input_entry in input_entries.flatten() {
            let name = input_entry.file_name().to_string_lossy().to_string();
            if name.starts_with("input") {
                let event_entries = fs::read_dir(input_entry.path()).ok()?;
                for event_entry in event_entries.flatten() {
                    let event_name = event_entry.file_name().to_string_lossy().to_string();
                    if event_name.starts_with("event") {
                        let dev_path = std::path::PathBuf::from("/dev/input").join(&event_name);
                        if dev_path.exists() {
                            return Some(dev_path);
                        }
                    }
                }
            }
        }
    }
    None
}
```

- [ ] **Step 4: 重构 DeviceOrchestrator 结构体**

```rust
pub struct DeviceOrchestrator {
    rx: mpsc::UnboundedReceiver<DeviceEvent>,
    whitelist: Arc<WhitelistManager>,
    audit: Arc<AuditService>,
    device_manager: Arc<RwLock<DeviceManager>>,

    // 新增: 下游服务
    scan_service: Arc<dyn crate::traits::Scanner>,
    file_access_engine: Arc<dyn crate::traits::DeviceMapper>,

    // 新增: I/O 资源
    mount_ops: RealMountOps,
    nbd_pool: NbdPool,
    hidg_nodes: hid_access::hid_gadget::HidgNodes,

    // 新增: 运行时追踪
    active_sessions: HashMap<String, ActiveSession>,
}
```

- [ ] **Step 5: 更新 new() 签名**

```rust
impl DeviceOrchestrator {
    pub fn new(
        rx: mpsc::UnboundedReceiver<DeviceEvent>,
        whitelist: Arc<WhitelistManager>,
        audit: Arc<AuditService>,
        device_manager: Arc<RwLock<DeviceManager>>,
        scan_service: Arc<dyn crate::traits::Scanner>,
        file_access_engine: Arc<dyn crate::traits::DeviceMapper>,
        hidg_nodes: hid_access::hid_gadget::HidgNodes,
    ) -> Self {
        Self {
            rx, whitelist, audit, device_manager,
            scan_service, file_access_engine,
            mount_ops: RealMountOps,
            nbd_pool: NbdPool::new(),
            hidg_nodes,
            active_sessions: HashMap::new(),
        }
    }
```

- [ ] **Step 6: 编译验证**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu 2>&1
```
Expected: 编译失败——main.rs 中 DeviceOrchestrator::new 参数不匹配（Task 8 修复），其余编译通过。

- [ ] **Step 7: Commit**

```bash
git add server/crates/usb-identify/src/orchestrator.rs
git commit -m "refactor(usb-identify): Orchestrator 注入服务 + find_evdev_path + ActiveSession

- DeviceOrchestrator 新增下游服务、I/O 资源、ActiveSession 三层字段
- find_evdev_path: USB sysfs → /dev/input/eventX 设备发现
- ActiveSession 与 DeviceManager 同键不同职责
- 本 commit 预期 main.rs 编译失败，Task 8 修复"
```

---

## Task 3: 实现 handle_keyboard 完整处理链

**Files:** `server/crates/usb-identify/src/orchestrator.rs`

**目的:** 将 KeyboardInterceptor 接入 Orchestrator。独立任务，不依赖 storage 链路。

- [ ] **Step 1: 添加 write_audit_generic_static 独立函数**

在 impl 块外添加（供 spawned task 使用，不依赖 `&self`）：

```rust
fn write_audit_generic_static(
    audit: &AuditService, info: &UsbDeviceInfo, event_type: &str,
    result: &str, device_type: &str, interface_type: &str,
    iface_class: i32, iface_subclass: i32, iface_proto: i32,
) {
    let mut log = UsbAuditLogInsert {
        event_time: now_unix(),
        device_type: Some(device_type.into()),
        interface_type: Some(interface_type.into()),
        interface_class: Some(iface_class), interface_subclass: Some(iface_subclass),
        interface_protocol: Some(iface_proto),
        device_name: Some(info.device_name.clone()),
        device_sn: Some(info.serial_number.clone()),
        vid: Some(info.vid.clone()), pid: Some(info.pid.clone()),
        event_type: event_type.to_string(),
        permission: None, capacity_bytes: None, file_path: None, matched_policy: None,
        result: result.to_string(),
        fail_reason: None, detail: None,
    };
    let _ = audit.log_usb_audit(&mut log);
}
```

- [ ] **Step 2: 重写 handle_keyboard**

```rust
async fn handle_keyboard(&mut self, info: UsbDeviceInfo) {
    let parent_path = crate::monitor::parent_device_path(&info.sys_path);

    // DeviceManager 登记
    if let Ok(mut dm) = self.device_manager.write() {
        dm.add(info.clone());
    }

    // 发现 evdev 设备节点
    let evdev_path = match find_evdev_path(&info.sys_path) {
        Some(p) => p,
        None => {
            warn!(dev = %info.device_name, "键盘: 找不到对应 evdev 设备");
            return;
        }
    };

    // 注册 ActiveSession
    let (cancel_tx, _cancel_rx) = watch::channel(false);
    self.active_sessions.insert(parent_path, ActiveSession {
        parent_path,
        device_type: common::types::DeviceType::Keyboard,
        nbd_index: None, mount_path: None,
        cancel_tx,
    });

    let hidg_kb = self.hidg_nodes.keyboard.clone();
    let device_name = info.device_name.clone();
    let info4audit = info.clone();
    let audit = Arc::clone(&self.audit);

    info!(dev = %device_name, evdev = %evdev_path.display(), "键盘: 启动拦截器");

    // spawn_blocking: evdev IO 是阻塞的，拔出时 read 报错自然退出
    tokio::task::spawn_blocking(move || {
        use hid_access::evdev_interceptor::KeyboardInterceptor;
        let mut interceptor = KeyboardInterceptor::new(hidg_kb);
        match interceptor.run(&evdev_path) {
            Ok(()) => info!(dev = %device_name, "键盘拦截器正常退出"),
            Err(e) => warn!(dev = %device_name, error = %e, "键盘拦截器异常退出"),
        }
        write_audit_generic_static(&audit, &info4audit, "device_removed",
            "success", "keyboard", "hid_keyboard", 0x03, 0x01, 0x01);
    });
}
```

- [ ] **Step 3: 编译验证**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add server/crates/usb-identify/src/orchestrator.rs
git commit -m "feat(usb-identify): 实现 handle_keyboard 完整处理链

- find_evdev_path → spawn_blocking KeyboardInterceptor
- 拔出时 evdev read 报错自然退出
- write_audit_generic_static 供 spawned task 记日志"
```

---

## Task 4: 实现 handle_mouse 完整处理链

**Files:** `server/crates/usb-identify/src/orchestrator.rs`

**目的:** 将 MouseForwarder 接入 Orchestrator。独立任务，与 keyboard 模式相同。

- [ ] **Step 1: 重写 handle_mouse**

```rust
async fn handle_mouse(&mut self, info: UsbDeviceInfo) {
    let parent_path = crate::monitor::parent_device_path(&info.sys_path);

    if let Ok(mut dm) = self.device_manager.write() {
        dm.add(info.clone());
    }

    let evdev_path = match find_evdev_path(&info.sys_path) {
        Some(p) => p,
        None => {
            warn!(dev = %info.device_name, "鼠标: 找不到对应 evdev 设备");
            return;
        }
    };

    let (cancel_tx, _cancel_rx) = watch::channel(false);
    self.active_sessions.insert(parent_path, ActiveSession {
        parent_path,
        device_type: common::types::DeviceType::Mouse,
        nbd_index: None, mount_path: None,
        cancel_tx,
    });

    let hidg_mouse = self.hidg_nodes.mouse.clone();
    let device_name = info.device_name.clone();
    let info4audit = info.clone();
    let audit = Arc::clone(&self.audit);

    info!(dev = %device_name, evdev = %evdev_path.display(), "鼠标: 启动转发器");

    tokio::task::spawn_blocking(move || {
        use hid_access::mouse_forwarder::MouseForwarder;
        let mut forwarder = MouseForwarder::new(hidg_mouse);
        match forwarder.run(&evdev_path) {
            Ok(()) => info!(dev = %device_name, "鼠标转发器正常退出"),
            Err(e) => warn!(dev = %device_name, error = %e, "鼠标转发器异常退出"),
        }
        write_audit_generic_static(&audit, &info4audit, "device_removed",
            "success", "mouse", "hid_mouse", 0x03, 0x01, 0x02);
    });
}
```

- [ ] **Step 2: 编译验证**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu 2>&1
```

- [ ] **Step 3: Commit**

```bash
git add server/crates/usb-identify/src/orchestrator.rs
git commit -m "feat(usb-identify): 实现 handle_mouse 完整处理链

- find_evdev_path → spawn_blocking MouseForwarder
- 鼠标无验证，插入即转发
- 拔出时 evdev read 报错自然退出"
```

---

## Task 5: 实现 handle_removed 清理逻辑

**Files:** `server/crates/usb-identify/src/orchestrator.rs`

**目的:** 拔出设备时取消后台 task、清理资源。依赖 ActiveSession 的 cancel_tx 机制。

- [ ] **Step 1: 重写 handle_removed**

```rust
async fn handle_removed(&mut self, sys_path: String) {
    let parent_path = crate::monitor::parent_device_path(&sys_path);

    // DeviceManager: 移除接口。只有最后一个接口移除才返回 Some
    let removed = if let Ok(mut dm) = self.device_manager.write() {
        dm.remove_interface(&sys_path)
    } else {
        None
    };

    let Some(device_record) = removed else {
        return; // 还有其他接口，物理设备未完全拔出
    };

    info!(dev = %device_record.info.device_name, type = ?device_record.info.device_type, "设备完全拔出");

    // ActiveSession: 发取消信号 + 兜底清理
    if let Some(session) = self.active_sessions.remove(&parent_path) {
        let _ = session.cancel_tx.send(true); // 幂等

        // Storage 兜底清理
        if session.device_type == common::types::DeviceType::Storage {
            if let Some(mount_path) = &session.mount_path {
                let _ = self.mount_ops.umount(&mount_path.to_string_lossy());
            }
            if let Some(idx) = session.nbd_index {
                self.nbd_pool.release(idx);
            }
        }
    }

    // 记移除日志
    let (dev_type_str, iface_str) = match device_record.info.device_type {
        common::types::DeviceType::Storage => ("storage", "mass_storage"),
        common::types::DeviceType::Keyboard => ("keyboard", "hid_keyboard"),
        common::types::DeviceType::Mouse => ("mouse", "hid_mouse"),
        _ => ("unsupported", "unsupported"),
    };
    self.write_audit_generic(&device_record.info, "device_removed", "success",
        dev_type_str, iface_str,
        device_record.info.interface_class as i32,
        device_record.info.interface_subclass as i32,
        device_record.info.interface_protocol as i32,
    );
}
```

- [ ] **Step 2: 编译验证**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu 2>&1
```

- [ ] **Step 3: Commit**

```bash
git add server/crates/usb-identify/src/orchestrator.rs
git commit -m "feat(usb-identify): 实现 handle_removed 完整清理

- dm.remove_interface 判断物理设备是否完全拔出
- cancel_tx.send 通知后台 task 取消（幂等）
- Storage 兜底: umount + NBD 归还
- Keyboard/Mouse: task 自然退出"
```

---

## Task 6: 实现 handle_storage 完整编排链

**Files:** `server/crates/usb-identify/src/orchestrator.rs`

**目的:** 核心任务——将 ScanService 和 FileAccessEngine 接入 Orchestrator，完成 mount → scan → map 全链路。每步通过 `tokio::select!` 监听 cancel_rx 支持拔出中断。

- [ ] **Step 1: 添加 spawned task 审计辅助函数**

在 impl 块外：

```rust
fn write_audit_storage_static(
    audit: &AuditService, info: &UsbDeviceInfo,
    event_type: &str, result: &str, fail_reason: Option<&str>,
) {
    let mut log = UsbAuditLogInsert {
        event_time: now_unix(),
        device_type: Some("storage".into()),
        interface_type: Some("mass_storage".into()),
        interface_class: Some(0x08),
        interface_subclass: None, interface_protocol: None,
        device_name: Some(info.device_name.clone()),
        device_sn: Some(info.serial_number.clone()),
        vid: Some(info.vid.clone()), pid: Some(info.pid.clone()),
        event_type: event_type.to_string(),
        permission: None, capacity_bytes: info.capacity_bytes,
        file_path: None, matched_policy: None,
        result: result.to_string(),
        fail_reason: fail_reason.map(|s| s.to_string()),
        detail: None,
    };
    let _ = audit.log_usb_audit(&mut log);
}

fn write_audit_fail(audit: &AuditService, info: &UsbDeviceInfo, event_type: &str, reason: &str) {
    write_audit_storage_static(audit, info, event_type, "failed", Some(reason));
}
```

- [ ] **Step 2: 重写 handle_storage**

```rust
async fn handle_storage(&mut self, info: UsbDeviceInfo) {
    // ===== Phase 1: Event Loop 同步判断 =====

    let parent_path = crate::monitor::parent_device_path(&info.sys_path);
    if let Ok(mut dm) = self.device_manager.write() {
        dm.add(info.clone());
    }

    let serial = info.serial_number.clone();
    if serial.is_empty() {
        self.write_audit_storage(&info, "whitelist_denied", "blocked", Some("序列号为空"));
        return;
    }
    let whitelist_entry = match self.whitelist.is_whitelisted(&serial) {
        Some(e) => e,
        None => {
            self.write_audit_storage(&info, "whitelist_denied", "blocked", Some("不在白名单"));
            return;
        }
    };

    let nbd_index = match self.nbd_pool.acquire() {
        Some(idx) => idx,
        None => {
            warn!("NBD 设备号池耗尽，拒绝映射");
            self.write_audit_storage(&info, "map_failed", "failed", Some("NBD 设备号池耗尽"));
            return;
        }
    };

    let (cancel_tx, cancel_rx) = watch::channel(false);
    self.active_sessions.insert(parent_path.clone(), ActiveSession {
        parent_path: parent_path.clone(),
        device_type: common::types::DeviceType::Storage,
        nbd_index: Some(nbd_index),
        mount_path: None,
        cancel_tx,
    });

    info!(serial = %serial, nbd = nbd_index, "Storage 设备开始编排");

    let scan_service = Arc::clone(&self.scan_service);
    let file_access_engine = Arc::clone(&self.file_access_engine);
    let audit = Arc::clone(&self.audit);
    let permission = whitelist_entry.permission;

    // ===== Phase 2: spawn 独立 task =====
    tokio::spawn(async move {
        let dev_path = match &info.dev_path {
            Some(p) if !p.is_empty() => p.clone(),
            _ => { write_audit_fail(&audit, &info, "map_failed", "dev_path 为空"); return; }
        };
        let dev_name = dev_name_from_path(&dev_path);
        let mount_point = mount_path_for(dev_name);
        let mount_ops = RealMountOps;

        // 1. Mount
        if let Err(e) = mount_ops.mount(&dev_path, &mount_point.to_string_lossy(), "auto") {
            write_audit_fail(&audit, &info, "scan_failed", &e.to_string());
            return;
        }
        let mount_path_str = mount_point.to_string_lossy().to_string();

        // 2. Scan（可取消）
        let scan_result = tokio::select! {
            r = scan_service.scan(&mount_point, &serial, &info.device_name) => r,
            _ = cancel_rx.changed() => {
                info!(serial = %serial, "扫描被取消（设备拔出）");
                let _ = mount_ops.umount(&mount_path_str);
                return;
            }
        };
        let scan_result = match scan_result {
            Ok(r) => r,
            Err(e) => {
                write_audit_fail(&audit, &info, "scan_failed", &e.to_string());
                let _ = mount_ops.umount(&mount_path_str);
                return;
            }
        };

        // 3. Map（可取消）
        let map_ctx = crate::traits::MapContext {
            mount_path: mount_path_str.clone(),
            scan_result: scan_result.clone(),
            permission,
        };
        let map_result = tokio::select! {
            r = file_access_engine.map_device(map_ctx) => r,
            _ = cancel_rx.changed() => {
                info!(serial = %serial, "映射被取消（设备拔出）");
                let _ = mount_ops.umount(&mount_path_str);
                return;
            }
        };
        match map_result {
            Ok(_session) => {
                write_audit_storage_static(&audit, &info, "mapped", "mapped", None);
                info!(serial = %serial, "U 盘映射成功");
            }
            Err(e) => {
                write_audit_fail(&audit, &info, "map_failed", &e.to_string());
                let _ = mount_ops.umount(&mount_path_str);
            }
        }
    });
}
```

- [ ] **Step 3: 编译验证**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add server/crates/usb-identify/src/orchestrator.rs
git commit -m "feat(usb-identify): 实现 handle_storage 完整编排链

- spawn 独立 task 执行 mount → scan → map 链路
- tokio::select! + cancel_rx 支持拔出时随时中断
- scan 失败与 Infected 路径分离
- 各步骤失败时自动 umount + 归还 NBD"
```

---

## Task 7: main.rs gadget_init + 服务注入

**Files:** `server/crates/app/src/main.rs`

**目的:** 串联所有模块——启动时配置 USB gadget（mass_storage + HID），实例化 ScanService 和 FileAccessEngine，注入 Orchestrator。

- [ ] **Step 1: 添加 imports**

```rust
use file_access::engine::FileAccessEngine;
use file_access::gadget::GadgetManager;
use hid_access::hid_gadget::{self, configure_hid_function, discover_hidg_nodes, KEYBOARD_FUNCTION, MOUSE_FUNCTION};
use hid_access::hid_report::{KEYBOARD_REPORT_DESC, KEYBOARD_REPORT_LEN, MOUSE_REPORT_DESC, MOUSE_REPORT_LEN};
use malware_scan::clam_scanner::ClamScanner;
use malware_scan::scan_service::ScanService;
```

- [ ] **Step 2: 在 `let state = Arc::new(AppState {...})` 之前插入**

```rust
    // ===== USB Gadget 统一初始化 =====
    let gadget_mgr = {
        let mut mgr = GadgetManager::new();
        let _ = mgr.unbind_udc();
        mgr.setup_gadget("USB Security Control Device")
            .expect("gadget 基础结构初始化失败");
        mgr.configure_mass_storage(&std::path::PathBuf::from("/dev/nbd0"), true)
            .expect("mass_storage function 配置失败");
        mgr
    };

    let functions_base = std::path::PathBuf::from("/sys/kernel/config/usb_gadget/usb_ctrl/functions");
    configure_hid_function(&functions_base.join(KEYBOARD_FUNCTION), 1, KEYBOARD_REPORT_DESC, KEYBOARD_REPORT_LEN)
        .expect("HID keyboard function 配置失败");
    configure_hid_function(&functions_base.join(MOUSE_FUNCTION), 2, MOUSE_REPORT_DESC, MOUSE_REPORT_LEN)
        .expect("HID mouse function 配置失败");

    gadget_mgr.link_function("mass_storage.usb0").expect("链接 mass_storage 失败");
    gadget_mgr.link_function(KEYBOARD_FUNCTION).expect("链接 keyboard 失败");
    gadget_mgr.link_function(MOUSE_FUNCTION).expect("链接 mouse 失败");
    gadget_mgr.bind_udc().expect("UDC 绑定失败");

    let hidg_nodes = discover_hidg_nodes().expect("hidg 节点发现失败");
    info!("HID gadget: keyboard={}, mouse={}", hidg_nodes.keyboard.display(), hidg_nodes.mouse.display());

    // ===== 实例化下游服务 =====
    let audit_service = Arc::new(AuditService::new(storage_audit, &db_path));
    let storage = Arc::new(storage_main);

    let scan_service = Arc::new(ScanService::new(
        ClamScanner::new("/usr/bin/clamdscan"),
        Arc::clone(&audit_service),
        &std::path::PathBuf::from("/var/log/usb-control/scan"),
    ));

    let file_access_engine = Arc::new(FileAccessEngine::new(
        Arc::clone(&storage),
        Arc::clone(&audit_service),
        "/dev/nbd0",
    ));
```

- [ ] **Step 3: 更新 DeviceOrchestrator::new 调用**

```rust
    let orchestrator = DeviceOrchestrator::new(
        event_rx,
        Arc::clone(&state.whitelist_manager),
        Arc::clone(&state.audit_service),
        Arc::clone(&state.device_manager),
        scan_service,
        file_access_engine,
        hidg_nodes,
    );
```

- [ ] **Step 4: 编译验证**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu 2>&1
```
Expected: 全量编译通过。

- [ ] **Step 5: Commit**

```bash
git add server/crates/app/src/main.rs
git commit -m "feat(app): 集成 gadget 初始化与 Orchestrator 服务注入

- 启动时统一配置 configfs: mass_storage + HID keyboard + mouse
- 实例化 ScanService、FileAccessEngine
- 注入 Orchestrator: scan_service + file_access_engine + hidg_nodes
- gadget 在 Orchestrator 之前完成，确保 /dev/hidg0/1 就绪"
```

---

## Task 8: 端到端编译验证

- [ ] **Step 1: 全量编译**

```bash
cd /root/work/code/usb-control-direct
cargo build --target aarch64-unknown-linux-gnu 2>&1
```
Expected: 成功，无 error。

- [ ] **Step 2: 运行全部单元测试**

```bash
cargo test --target aarch64-unknown-linux-gnu 2>&1
```
Expected: 已有测试全部 PASS，无 regression。

- [ ] **Step 3: clippy 检查**

```bash
cargo clippy --target aarch64-unknown-linux-gnu -- -D warnings 2>&1
```
Expected: 无新增 warning。

- [ ] **Step 4: Commit**

```bash
git commit --allow-empty -m "test: 端到端编译通过，全量测试 PASS"
```

---

## 不在本计划范围

- ❌ 黑名单预置数据替换（gap analysis §2.1，独立修复）
- ❌ 存储 >80% 日志滚动覆盖
- ❌ 日志清理半年内禁止校验
- ❌ TLS 证书指纹校验
- ❌ heartbeat 间隔修正
- ❌ 功能级差距 §4.1-4.8
