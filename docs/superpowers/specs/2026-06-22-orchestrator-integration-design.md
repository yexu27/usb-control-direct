# Orchestrator 模块集成架构设计

**日期**: 2026-06-22
**分支**: feat/device-usb-orchestration
**基线**: 架构 01-09, PRD v1.0.1, 2026-06-22-usb-orchestration-design.md, 差距分析报告

---

## 1. 问题诊断

对比 `2026-06-22-usb-orchestration-design.md`（下称"前设计"）与当前代码，`DeviceOrchestrator` 的三个 handler 方法仅写审计日志，未调用已实现的模块：

| Handler | 前设计要求 | 当前代码 |
|---|---|---|
| `handle_storage` | mount → ScanService.scan() → FileAccessEngine.map_device() | 仅白名单查询 + 审计日志 |
| `handle_keyboard` | spawn KeyboardInterceptor（evdev grab + 1234 + 转发） | 仅审计日志 |
| `handle_mouse` | spawn MouseForwarder（evdev 转发） | 仅审计日志 |

**更深层问题**：前设计有 4 个工程关键细节未覆盖，直接照搬会遇到硬障碍：

| # | 缺失点 | 后果 |
|---|---|---|
| 1 | 长耗时操作的并发模型 | 单 task FIFO 循环中 await scan() 会阻塞所有后续事件（含拔出） |
| 2 | evdev 设备节点发现 | udev 给出 USB sysfs 路径，但拦截器需要 `/dev/input/eventX` |
| 3 | 设备移除的资源清理 | 无统一的会话追踪和取消机制 |
| 4 | 跨 crate GadgetManager 统一初始化 | mass_storage 在 file-access，HID 在 hid-access，需协调 bind UDC |

本文档聚焦这 4 个工程问题，在前设计基础上给出可落地的完整方案。策略/状态机/扫描等模块内部逻辑不变。

---

## 2. 架构决策

### 2.1 并发模型：两阶段处理

```
┌─ Event Loop (单 task, 快速) ─────────────────────────────────┐
│                                                              │
│  StorageAdded ─→ dm.add → 白名单检查 → 分配 NBD → sessions.insert → spawn → 返回  │
│  KeyboardAdded ─→ dm.add → 发现 evdev → sessions.insert → spawn_blocking → 返回   │
│  MouseAdded ─→ dm.add → 发现 evdev → sessions.insert → spawn_blocking → 返回      │
│  DeviceRemoved ─→ dm.remove_interface → sessions.remove → 发取消信号 → 返回        │
│                                                              │
└──────────────────────────────────────────────────────────────┘
         │                               │
         ▼                               ▼
┌─ Storage Task (独立 spawn) ─┐   ┌─ HID Task (spawn_blocking) ─┐
│  mount → scan → file tree   │   │  evdev grab → 1234/转发    │
│  → NBD → gadget enable      │   │  循环直到拔出或错误         │
│  select! cancel_watch       │   │  拔出时 evdev read 报错退出 │
└─────────────────────────────┘   └────────────────────────────┘
```

**关键原则**：
- Event loop 只做**同步判断**（白名单查询、NBD 号分配、evdev 路径发现），< 1ms
- 长耗时操作在**独立 task** 中执行，Event loop 立即回到 `rx.recv()` 等待下一个事件
- 设备拔出事件**不排队**，到达即处理（查 ActiveSession → 取消对应 task → 清理资源）

### 2.2 设备追踪：DeviceManager 与 ActiveSession 两层模型

Orchestrator 同时维护两个结构，它们的职责**不同层**，两个都需要：

| | DeviceManager（已有） | ActiveSession（新增） |
|---|---|---|
| **存什么** | 设备属性：vid/pid/serial/接口列表/设备名 | 运行时资源：cancel_tx、nbd_index、mount_path、JoinHandle |
| **消费者** | Orchestrator + **S10 协议层**（`CMD_GET_CONNECTED_DEVICES` 查询"当前连接了哪些设备"） | **仅 Orchestrator 内部**（追踪后台 task、取消、清理） |
| **生命周期** | 插入→拔出，与物理设备一一对应 | spawn task→task 结束，与后台任务一一对应 |
| **键** | parent device path | parent device path（同键，可关联） |
| **所在位置** | `Arc<RwLock<DeviceManager>>` 放入 `AppState`（协议层共享） | `HashMap<String, ActiveSession>` 在 `DeviceOrchestrator` 内部（私有） |

**为什么不能合并**：DeviceManager 通过 `Arc<RwLock<>>` 被协议层共享读取。把 `cancel_tx`（tokio 同步原语）和 `JoinHandle` 塞入会污染公开接口，且 `RwLock` 中不适合放 tokio 运行时对象。

**两层协同**：

```
Ochestrator 内部:
  DeviceManager (Arc<RwLock<>>)   ← 公开，协议层可查询设备列表
  ActiveSessions (私有 HashMap)   ← 私有，只有 Orchestrator 操作
       │
       └── 同键(parent device path)，一一对应
```

#### DeviceManager（已有，不改）

```rust
// 已有结构，不改
pub struct DeviceManager {
    records: HashMap<String, DeviceRecord>,  // key = parent device path
}

pub struct DeviceRecord {
    pub info: UsbDeviceInfo,
    pub interfaces: Vec<String>,
    pub connected_at: i64,
}
```

- `add(info)` — 同物理设备的多个接口合并为一条记录
- `remove_interface(sys_path)` — 移除一个接口；最后一个接口移除时返回 `Some(DeviceRecord)`
- `list_all()` — 供协议层查询"当前连接了哪些设备"

#### ActiveSession（新增，Orchestrator 私有）

```rust
struct ActiveSession {
    parent_path: String,                     // HashMap key，与 DeviceManager 同键
    device_type: DeviceType,                 // Storage / Keyboard / Mouse

    // Storage 专用
    nbd_index: Option<u32>,
    mount_path: Option<PathBuf>,

    // 所有类型通用
    cancel_tx: tokio::sync::watch::Sender<bool>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}
```

**生命周期**：
1. 设备事件到达 → `dm.add(info)` **且** `sessions.insert(parent_path, ActiveSession)` — 两处同时写入
2. 后台 task 完成或出错 → task 自行清理资源 → Orchestrator 从 `sessions` 移除
3. `DeviceRemoved` → `dm.remove_interface(sys_path)` **且** `sessions.remove(&parent_path)` — 两处同时移除

### 2.3 evdev 设备节点发现

USB HID 设备在内核中创建 input 子设备，sysfs 层级关系：

```
/sys/devices/.../2-1.1/2-1.1:1.0           ← udev 事件给的 USB 接口路径
                        /0003:046D:C077.0001/input/input3/event3  ← 我们要找的
```

**发现算法**：从 USB 接口 sysfs 路径出发，递归遍历子目录，找 `input/input*/event*`：

```rust
fn find_evdev_path(usb_iface_syspath: &str) -> Option<PathBuf> {
    // 1. 遍历 usb_iface_syspath 的所有子目录
    // 2. 对每个子目录，检查是否存在 input/inputX/eventX
    // 3. 对找到的 eventX，尝试 evdev::Device::open 验证是键盘还是鼠标
    //    （match InputEventKind::Key 或 RelAxis）
    // 4. 返回 /dev/input/eventX 路径
}
```

备选方案：`/dev/input/by-path/` 符号链接包含 USB 拓扑，可作为快速路径。但并非所有内核配置都启用，故作为 fallback 的第二优先级。

### 2.4 跨 crate Gadget 统一初始化

不引入跨 crate 依赖，由 `main.rs` 统一协调 configfs 配置：

```
main.rs gadget_init()
  1. 创建/清空 configfs 基础目录（通过 file-access::GadgetManager 的公开方法）
  2. 配置 f_mass_storage function（file-access）
  3. 配置 hid.usb0 function（hid-access::hid_gadget::configure_hid_function）
  4. 配置 hid.usb1 function（hid-access::hid_gadget::configure_hid_function）
  5. 链接三个 function 到 config
  6. bind UDC
  7. 调用 hid-access::hid_gadget::discover_hidg_nodes() 获取 /dev/hidg{0,1}
  8. 返回 (GadgetManager, HidgNodes)
```

需要将 `file-access::gadget::GadgetManager` 中当前私有的方法暴露为 `pub`：
- `setup_base()` → `pub`
- `setup_mass_storage_function()` → `pub`
- `link_function(name: &str)` → `pub`
- `bind_udc()` → `pub`
- `unbind_udc()` → `pub`

---

## 3. 组件设计

### 3.1 DeviceOrchestrator 完整结构

```rust
pub struct DeviceOrchestrator {
    // === 事件源 ===
    rx: mpsc::UnboundedReceiver<DeviceEvent>,

    // === 注入服务（Arc，共享所有权） ===
    whitelist: Arc<WhitelistManager>,
    audit: Arc<AuditService>,
    device_manager: Arc<RwLock<DeviceManager>>,
    scan_service: Arc<ScanService>,
    file_access_engine: Arc<FileAccessEngine>,

    // === I/O 资源（独占所有权） ===
    mount_ops: RealMountOps,
    nbd_pool: NbdPool,
    hidg_nodes: HidgNodes,          // { keyboard: /dev/hidg0, mouse: /dev/hidg1 }

    // === 运行时状态 ===
    active_sessions: HashMap<String, ActiveSession>,
}
```

### 3.2 ActiveSession

```rust
struct ActiveSession {
    parent_path: String,                     // HashMap key
    device_type: DeviceType,                 // Storage / Keyboard / Mouse
    nbd_index: Option<u32>,                  // Storage 占用
    mount_path: Option<PathBuf>,             // Storage 挂载点
    cancel_tx: watch::Sender<bool>,          // 取消信号
    task_handle: Option<JoinHandle<()>>,     // 后台 task
}
```

### 3.3 Storage 处理链（spawn 版本）

```
handle_storage(info)
  │
  ├─ [Event Loop 同步] dm.add(info.clone())   ← DeviceManager：协议层可查询
  ├─ [Event Loop 同步] 白名单检查：serial 为空或不在白名单 → block + return
  ├─ [Event Loop 同步] NBD 号池分配：池耗尽 → fail + return
  ├─ [Event Loop 同步] sessions.insert(parent_path, ActiveSession { cancel_tx, nbd_index, ... })
  │
  └─ [spawn 独立 task]
       │
       ├─ mount(dev_path, /mnt/usb_raw/{name})
       │    └─ 失败 → 记 SCAN_FAILED → 归还 NBD → sessions.remove → return
       │
       ├─ tokio::select! {
       │      result = scan_service.scan(&mount_path, &sn, &name) => result,
       │      _ = cancel_rx.changed() => { /* 用户拔出，取消扫描 */ return; }
       │  }
       │    ├─ Clean → 继续
       │    ├─ Infected → 继续（病毒文件标记 [病毒禁止访问]，仍需映射）
       │    └─ Failed → 记 SCAN_FAILED → unmount → 归还 NBD → sessions.remove → return
       │
       ├─ tokio::select! {
       │      result = file_access_engine.map_device(MapContext { ... }) => result,
       │      _ = cancel_rx.changed() => { /* 用户拔出 */ unmount → 归还 NBD → return; }
       │  }
       │    ├─ Ok(session) → 记 MAPPED 日志
       │    └─ Err → 记 MAP_FAILED → unmount → 归还 NBD → sessions.remove → return
       │
       └─ 等待拔出（task 挂起，等待 cancel_rx 或 NBD 连接断开）
            └─ 收到取消 → unmap_device → unmount → 归还 NBD → 记 REMOVED 日志
```

**关键设计点**：
- mount、scan、map_device 三个长耗时步骤都与 `cancel_rx` 做 `tokio::select!`
- 拔出信号在任何阶段到达都能立即中断，不等待当前步骤完成
- scan 失败与 scan 发现病毒的路径不同：Infected 是正常结果，Failed 才阻断映射

### 3.4 Keyboard 处理链

```
handle_keyboard(info)
  │
  ├─ [Event Loop 同步] dm.add(info.clone())   ← DeviceManager：协议层可查询
  ├─ [Event Loop 同步] find_evdev_path(info.sys_path)
  │    └─ 找不到 → 记日志 → return
  ├─ [Event Loop 同步] sessions.insert(parent_path, ActiveSession { cancel_tx, ... })
  │
  └─ [spawn_blocking]
       │
       ├─ Device::open(evdev_path)  ← evdev grab 在此隐式发生
       │    └─ 失败 → 记日志 → return
       │
       ├─ Phase 1: 验证循环（fetch_events → KeyboardChallenge::transition）
       │    ├─ KbMapped → 记映射成功日志 → 进入 Phase 2
       │    ├─ KbRejected → 记验证失败日志 → return
       │    └─ 读取错误（拔出）→ 记移除日志 → return
       │
       └─ Phase 2: 转发循环（fetch_events → KeyboardReport → write /dev/hidg0）
            └─ 读取/写入错误（拔出）→ 记移除日志 → return
```

**关键设计点**：
- `spawn_blocking` 因为 evdev 是同步阻塞 IO
- evdev grab（`EVIOCGRAB`）由 `evdev::Device::open()` 隐式执行
- 不需要显式取消——USB 拔出时 evdev read 返回 `ENODEV`，task 自然退出
- 如果同一物理键盘拔出后又插入，udev 发新事件，创建新 task

### 3.5 Mouse 处理链

```
handle_mouse(info)
  │
  ├─ [Event Loop 同步] dm.add(info.clone())   ← DeviceManager：协议层可查询
  ├─ [Event Loop 同步] find_evdev_path(info.sys_path)
  ├─ [Event Loop 同步] sessions.insert(parent_path, ActiveSession { cancel_tx, ... })
  │
  └─ [spawn_blocking]
       │
       ├─ Device::open(evdev_path)
       ├─ 记插入日志
       │
       └─ 转发循环（fetch_events → MouseReport → write /dev/hidg1）
            └─ 读取/写入错误（拔出）→ 记移除日志 → return
```

**关键设计点**：
- 鼠标无验证，插入即转发
- 与键盘相同：拔出 → evdev read 报错 → 自然退出
- 不需要显式取消机制

### 3.6 DeviceRemoved 处理

```
handle_removed(sys_path)
  │
  ├─ parent_path = parent_device_path(&sys_path)
  │
  ├─ let removed = dm.remove_interface(&sys_path)   ← DeviceManager 移除接口
  │    └─ 返回 None → 还有其他接口，物理设备未完全移除 → return（不清除 session）
  │    └─ 返回 Some(DeviceRecord) → 最后一个接口，物理设备完全移除 → 继续
  │
  ├─ session = active_sessions.remove(&parent_path)?
  │    └─ 找不到 → 可能已自然结束 → return
  │
  ├─ cancel_tx.send(true)   ← 通知后台 task 取消
  │
  └─ 兜底清理（后台 task 收到取消信号后自行清理，此处仅兜底）：
       ├─ Storage: 如果 task 已结束未清理，主动 unmount + NBD 归还
       └─ Keyboard/Mouse: task 自然退出，无需额外清理
```

**关键设计点**：
- `cancel_tx.send(true)` 是幂等的（第二个发送返回 error，忽略）
- 后台 task 收到取消信号后自行清理资源，Orchestrator 仅做兜底
- 兜底清理在 `active_sessions.remove()` 返回的 session 上执行（task 可能已结束）

---

## 4. main.rs 启动顺序

```
1. Storage::open() ×5                  ← 不改
2. AuthService, AuditService, SessionManager  ← 不改
3. WhitelistManager, DeviceManager     ← 不改
4. PolicyService, LicenseValidator     ← 不改
5. SystemUpgradeManager, VirusdbUpgradeManager  ← 不改
6. ScanService::new()                  ← 新增：实例化扫描服务
7. FileAccessEngine::new()             ← 新增：实例化文件访问引擎
8. gadget_init_all()                   ← 新增：统一 configfs 初始化
     ├─ unbind_udc()
     ├─ setup_base()
     ├─ setup_mass_storage_function()
     ├─ configure_hid_function(hid.usb0, keyboard)
     ├─ configure_hid_function(hid.usb1, mouse)
     ├─ link_function("mass_storage.usb0")
     ├─ link_function("hid.usb0")
     ├─ link_function("hid.usb1")
     ├─ bind_udc()
     └─ discover_hidg_nodes() → HidgNodes
9. DeviceOrchestrator::new(..., scan_service, file_access_engine, hidg_nodes)
10. tokio::spawn(orchestrator.run())
11. tokio::spawn(udev_monitor_loop)
12. tokio::spawn_blocking(enumerate_existing)
13. TcpListener::bind() → 协议服务    ← 不改
```

**关键设计点**：
- 步骤 8 `gadget_init_all()` 在 Orchestrator 启动**之前**，确保 `/dev/hidg0`、`/dev/hidg1` 已就绪
- 步骤 9 将 ScanService、FileAccessEngine、HidgNodes 注入 Orchestrator
- 步骤 8 如果失败（如 UDC 不可用），main 应优雅降级：记录错误但不崩溃，Orchestrator 仍可处理审计日志

---

## 5. 资源清理策略

| 场景 | Storage 清理 | Keyboard 清理 | Mouse 清理 |
|---|---|---|---|
| 正常拔出 | umount → NBD_DISCONNECT → 归还 NBD → 记日志 | evdev read 报错 → drop fd → 记日志 | 同键盘 |
| 拔出时正在扫描 | cancel_tx 中断 scan → umount → 归还 NBD → 记日志 | evdev read 报错 → drop | evdev read 报错 → drop |
| 扫描失败 | umount → 归还 NBD → 记日志 | N/A | N/A |
| NBD 启动失败 | umount → 归还 NBD → 记日志 | N/A | N/A |
| 后台 task panic | 归还 NBD（JoinHandle 检查）→ 记日志 | JoinHandle 检查 → 记日志 | 同键盘 |

---

## 6. 错误处理约定

1. **Event loop 内不 panic**：所有同步操作使用 `Result` 处理，失败记日志后 return
2. **后台 task panic 不传播**：Orchestrator 定期检查 JoinHandle，发现 panic 后清理资源
3. **cancel_tx.send() 幂等**：重复发送返回 error 时忽略
4. **spawn_blocking task 不 await**：evdev task 自然退出，Orchestrator 不主动 join（避免阻塞 event loop）

---

## 7. 与前设计的差异

| 前设计 | 本文档修正 | 原因 |
|---|---|---|
| 单 task 内 await scan() | spawn 独立 task + cancel_tx | 长耗时阻塞事件循环 |
| 未定义 evdev 设备发现 | 新增 find_evdev_path() | udev <-> evdev 桥接 |
| DeviceSession 简单维护 | ActiveSession + HashMap + cancel_tx | 拔出时需要取消+清理 |
| GadgetManager::init_all() 跨 crate | main.rs 协调两个 crate 的 gadget 函数 | 不引入 S04 → S02 反向依赖 |
| DeviceRemoved 模糊处理 | 明确的 cancel_tx + 资源清理矩阵（§5） | 防止资源泄漏 |

---

## 8. 实现顺序

按依赖关系和风险排序：

| 序号 | 任务 | 涉及文件 | 依赖 |
|---|---|---|---|
| 1 | 暴露 GadgetManager 公开方法 | `file-access/src/gadget.rs` | 无 |
| 2 | 黑名单预置数据替换为 PRD 38 项 | `storage/src/schema.rs` | 无 |
| 3 | 实现 find_evdev_path() | `usb-identify/src/orchestrator.rs` | 无 |
| 4 | 重构 Orchestrator：注入 ScanService/FileAccessEngine/HidgNodes + NbdPool | `usb-identify/src/orchestrator.rs` | 1 |
| 5 | 实现 ActiveSession + cancel_tx 机制 | `usb-identify/src/orchestrator.rs` | 4 |
| 6 | 实现 handle_storage 完整链（spawn 版本） | `usb-identify/src/orchestrator.rs` | 5 |
| 7 | 实现 handle_keyboard / handle_mouse（spawn_blocking） | `usb-identify/src/orchestrator.rs` | 3, 5 |
| 8 | 实现 handle_removed 清理逻辑 | `usb-identify/src/orchestrator.rs` | 5, 6, 7 |
| 9 | main.rs：gadget_init_all() + 实例化注入 | `app/src/main.rs` | 1, 4 |
| 10 | 端到端集成测试 | `usb-identify/tests/` | 全部 |

---

## 9. 不在本设计范围

- 病毒扫描引擎实现（已有）
- 文件访问控制策略引擎（已有）
- NBD 服务器/exFAT 生成逻辑（已有）
- 白名单 CRUD / 协议处理（已有）
- 4.1-4.8 功能级差距（存储滚动覆盖、日志半年校验等）——后续设计覆盖
- TLS 证书指纹校验
- systemd sd-notify 集成
