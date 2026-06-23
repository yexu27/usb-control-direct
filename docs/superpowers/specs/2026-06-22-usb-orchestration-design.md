# 装置端 USB 设备编排与键鼠控制设计

**日期**: 2026-06-22
**分支**: feat/device-usb-orchestration
**基线**: 架构 01-09, PRD v1.0.1, 差距分析报告 2026-06-22

---

## 1. 问题概述

差距分析报告识别出 4 个核心流程缺口：

| # | 问题 | 根因 |
|---|---|---|
| 1 | U 盘插入→映射链路不通 | 各模块独立可用，但 udev 事件后无主编排器串联 |
| 2 | 键盘 HID 拦截未实现 | 仅状态机，无 evdev 设备读写代码 |
| 3 | 鼠标自动映射未实现 | 仅有枚举定义 |
| 4 | OTG HID gadget 未实现 | `gadget.rs` 仅支持 `f_mass_storage` |

**约束**：各模块内部代码（状态机、NBD、扫描、白名单）质量已验证，本设计只解决**编排与连通**，不重构已有模块。

---

## 2. 架构总览

```
┌── S10 协议网关 ──────────────────────────────────┐
│  管理端连接、TLS、鉴权、角色权限（不改）               │
└──────────────────────────────────────────────────┘

┌── main.rs ───────────────────────────────────────┐
│  实例化所有服务 → 启动 udev 监听 → 启动 Orchestrator │  ◀── 改动
│  启动 gadget 初始化（含 HID function）              │  ◀── 改动
└──────────────────────────────────────────────────┘
         │                                │
         │  udev 事件                      │  管理端命令
         ▼                                ▼
┌── S01 usb-identify ───┐     ┌── S10 协议网关 ──┐
│  udev 实时监听           │     │  CMD_GET_CONNECTED │
│  描述符解析+设备分类       │◀────│  _DEVICES      │
│  mount/umount           │     │  (已有)          │
│  U盘状态机                │     └───────────────-─┘
│  ─────────────────      │
│  DeviceOrchestrator  ◀──┼── 新增：主编排器
│  (路由 Storage/Keyboard/  │
│   Mouse/Unsupported)     │
└──────────┬──────────────┘
           │
    ┌──────┼──────────┐
    ▼      ▼          ▼
┌── S02 ─┐ ┌── S03 ──┐ ┌── S05 ────┐
│ 键盘:    │ │ 病毒扫描  │ │ 白名单查询  │
│ evdev   │ │ (不改)   │ │ (不改)    │
│ 拦截+   │ └────┬────┘ └───────────┘
│ 1234    │      │
│ 验证    │      ▼
│ 鼠标:    │ ┌── S04 ──────────┐
│ evdev   │ │ NBD + exFAT +   │
│ 转发    │ │ Gadget mass_stg │
└────┬────┘ │ (不改逻辑,改入口)│
     │      └────────┬────────┘
     ▼               ▼
┌── S02 gadget ──────────────┐
│  HID gadget (f_hid)        │  ◀── 新增能力
│  /dev/hidg0 键盘 /dev/hidg1 鼠标 │
└────────────────────────────┘

所有事件 → S07 AuditService 记录 USB 审计日志
```

---

## 3. 各模块设计

### 3.1 S01 主编排器 (`usb-identify/src/orchestrator.rs`) — 新增

**职责**：接收 udev 事件，按设备类型路由到对应处理链，管理设备生命周期。

**输入**：来自 `udev_monitor` 的 `DeviceEvent`

```rust
pub enum DeviceEvent {
    StorageAdded(UsbDeviceInfo),    // 大容量存储设备插入
    KeyboardAdded(UsbDeviceInfo),   // 键盘插入
    MouseAdded(UsbDeviceInfo),      // 鼠标插入
    UnsupportedAdded(UsbDeviceInfo, String), // 不支持的设备+原因
    DeviceRemoved(String),          // sys_path
}
```

**处理链定义**：

```
StorageAdded(info)
  1. S05 WhitelistManager::check(serial) → 不在白名单?
     → S07 记 BLOCKED 日志 → return
  2. S01 RealMountOps::mount(dev, /mnt/usb_raw/{name}) → 失败?
     → S07 记 SCAN_FAILED 日志 → return
  3. S03 ScanService::scan(mount_path, info) → 结果(Clean/Infected/Failed)
     → S07 记恶意代码检测日志
     → Infected 时 S07 记病毒文件命中
  4. S04 FileAccessEngine::map(mount_path, info, scan_result, permission)
     → 成功: S07 记 MAPPED 日志
     → 失败: S07 记 MAP_FAILED 日志

KeyboardAdded(info)
  1. S02 KeyboardInterceptor::run(dev_path, hidg_keyboard_path)
     → 验证通过: S07 记映射成功
     → 验证失败或拔出: S07 记对应日志

MouseAdded(info)
  1. S02 MouseForwarder::run(dev_path, hidg_mouse_path)
     → 成功: S07 记映射成功
     → 失败: S07 记映射失败

UnsupportedAdded(info, reason)
  1. S07 记 BLOCKED 日志

DeviceRemoved(sys_path)
  1. 查找对应设备 → 类型分派 cleanup
     ├── Storage: umount + NBD stop + gadget LUN clear
     ├── Keyboard: evdev drop + hidg stop
     ├── Mouse: evdev drop + hidg stop
     └── S07 记设备移除日志
```

**并发模型**：
- Orchestrator 自身是单个 tokio task（按 FIFO 顺序处理事件）
- Storage 的 NBD server 在独立 `spawn_blocking` 中运行（长期阻塞）
- Keyboard/Mouse 的 evdev 读取在独立 `spawn_blocking` 中运行
- 同一时间允许多个 storage 映射（多 USB 口），通过 NBD 设备号池（`/dev/nbd0`—`/dev/nbd3`）区分

**NBD 设备号管理**：
```rust
struct NbdPool {
    available: Vec<u32>,  // [0, 1, 2, 3]
    in_use: HashSet<u32>,
}
```
- 映射时从池中取一个空闲的 nbdN
- 取消映射时归还
- 池耗尽时新 U 盘等待或拒绝

**与现有代码的关系**：
- `DeviceManager` 保留，改为事件生产者（udev 事件 → mpsc channel → Orchestrator）
- `UsbStateMachine` 不改，Orchestrator 在关键转换点调用 `transition()`
- `DeviceSession` 的 `mount_point` 等字段由 Orchestrator 维护

---

### 3.2 键盘 HID 拦截 (`hid-access/src/evdev_interceptor.rs`) — 新增

**依赖**：新增 `evdev = "0.12"` 到 `hid-access/Cargo.toml`

**复用现有**：`KeyboardChallenge` 状态机（不改）

**实现结构**：

```rust
pub struct KeyboardInterceptor {
    challenge: KeyboardChallenge,  // 已有状态机
    hidg_device: PathBuf,          // /dev/hidg0
    audit: Arc<AuditService>,      // 记日志
    device_info: UsbDeviceInfo,    // 设备信息（用于日志字段）
}

impl KeyboardInterceptor {
    /// 在 spawn_blocking 中运行（evdev 为阻塞 IO）
    pub fn run(mut self, input_dev_path: &Path) -> Result<(), HidAccessError> {
        let mut dev = Device::open(input_dev_path)?;  // evdev 自动 grab

        self.audit.usb_insert(&self.device_info, "keyboard")?;

        loop {
            for ev in dev.fetch_events()? {
                let Some(key_event) = self.parse_evdev_key_event(&ev) else {
                    continue;
                };

                match self.challenge.transition(key_event.into())? {
                    Transitioned(KbMapped) => {
                        self.audit.keyboard_mapped(&self.device_info)?;
                        return self.forward_loop(&mut dev);  // 进入转发
                    }
                    Transitioned(KbRejected) => {
                        self.audit.keyboard_rejected(&self.device_info)?;
                        return Err(...);
                    }
                    Transitioned(KbRemoved) => {
                        return Ok(());  // 拔出
                    }
                    Unchanged => {}  // 继续等
                }
            }
        }
    }
}
```

**evdev 到 KeyboardEvent 映射**：
- `InputEventKind::Key(key)` + `value == 1`（按下事件）→ 判断
  - 若为修饰键（`KEY_LEFTCTRL/KEY_LEFTSHIFT/KEY_CAPSLOCK/...`）→ `KeyboardEvent::ModifierKey`
  - 否则 → `KeyboardEvent::KeyPress(keycode_to_hid_usage(key))`
- `keycode_to_hid_usage()` 映射表仅需覆盖 1234 验证序列（数字 1-4 的 HID usage: 0x1E-0x21），以及转发阶段的标准按键（约 50 个）

**转发阶段**：evdev 事件 → KeyboardReport → 写 `/dev/hidg0`
- 维护 `pressed` 集合（最多 6 键，忽略超出）
- 维护 `modifiers` 位掩码

---

### 3.3 鼠标自动映射 (`hid-access/src/mouse_forwarder.rs`) — 新增

**无验证**，插入即转发。

```rust
pub struct MouseForwarder {
    hidg_device: PathBuf,
    audit: Arc<AuditService>,
    device_info: UsbDeviceInfo,
}

impl MouseForwarder {
    pub fn run(mut self, input_dev_path: &Path) -> Result<(), HidAccessError> {
        let mut dev = Device::open(input_dev_path)?;
        self.audit.usb_insert(&self.device_info, "mouse")?;

        let mut buttons: u8 = 0;
        loop {
            let mut dx: i32 = 0;
            let mut dy: i32 = 0;
            let mut wheel: i32 = 0;
            let mut changed = false;

            for ev in dev.fetch_events()? {
                match ev.kind() {
                    Key(BTN_LEFT)   => { buttons = bit_modify(buttons, 0, ev.value()); changed = true; }
                    Key(BTN_RIGHT)  => { buttons = bit_modify(buttons, 1, ev.value()); changed = true; }
                    Key(BTN_MIDDLE) => { buttons = bit_modify(buttons, 2, ev.value()); changed = true; }
                    RelAxis(REL_X)  => { dx += ev.value(); changed = true; }
                    RelAxis(REL_Y)  => { dy += ev.value(); changed = true; }
                    RelAxis(REL_WHEEL) => { wheel += ev.value(); changed = true; }
                    _ => {}
                }
            }
            if changed {
                let report = MouseReport { buttons, dx: clamp_i8(dx), dy: clamp_i8(dy), wheel: clamp_i8(wheel) };
                write_hid_report(&self.hidg_device, &report.to_bytes())?;
            }
        }
    }
}
```

---

### 3.4 OTG HID Gadget (`file-access/src/gadget.rs` 扩展 + `hid-access/src/hid_gadget.rs` 新增)

**设计原则**：gadget 在服务启动时一次性配置完成（所有 function 同时注册、链接、bind UDC）。设备接入时只操作业务层（NBD device、hidg 写入），不重配置 configfs。

**启动时 gadget 初始化**：

```rust
impl GadgetManager {
    pub fn init_all(&mut self, config: &GadgetConfig) -> Result<HidgNodes> {
        // 1. 解绑 UDC
        self.unbind_udc()?;

        // 2. 创建 usb_ctrl gadget 基础结构（已有，idVendor/idProduct/strings）
        self.setup_gadget_base()?;

        // 3. 创建三个 function
        self.setup_mass_storage_function()?;  // 已有
        self.setup_hid_keyboard()?;           // 新增
        self.setup_hid_mouse()?;              // 新增

        // 4. 链接 function → config
        self.link_function("mass_storage.usb0")?;  // 已有
        self.link_function("hid.usb0")?;            // 新增
        self.link_function("hid.usb1")?;            // 新增

        // 5. bind UDC
        self.bind_udc()?;

        // 6. 扫描 /dev/hidg* 确定键盘/鼠标对应关系
        let nodes = discover_hidg_nodes()?;
        // nodes[0] → 键盘, nodes[1] → 鼠标
        Ok(nodes)
    }
}
```

**HID function 配置**（写到 `functions/hid.usb0/` 和 `functions/hid.usb1/`）：

| 属性文件 | hid.usb0 (键盘) | hid.usb1 (鼠标) |
|---|---|---|
| `protocol` | `"1"` | `"2"` |
| `subclass` | `"1"` | `"1"` |
| `report_length` | `"8"` | `"4"` |
| `report_desc` | 标准 boot keyboard 描述符（50 字节） | 标准 boot mouse 描述符（39 字节） |

**HID Report 结构**：

键盘（8 字节）：
```
[modifier(1)] [reserved(1)] [key0..key5(6)]
```
- modifier 位掩码：bit0=左Ctrl, bit1=左Shift, bit2=左Alt, bit3=左Meta, bit4=右Ctrl, ...
- key[0..5]：当前按下的键的 HID usage code（最多 6 键同时按下）
- 全部为 0 = 无按键

鼠标（4 字节）：
```
[buttons(1)] [dx(1)] [dy(1)] [wheel(1)]
```
- buttons：bit0=左键, bit1=右键, bit2=中键
- dx/dy/wheel：有符号 8 位相对位移

**设备接入时的操作**：
- 键盘验证通过后：`KeyboardInterceptor` 循环写 `/dev/hidg0`
- 鼠标插入后：`MouseForwarder` 循环写 `/dev/hidg1`
- U 盘映射后：NBD server 初始化，`f_mass_storage` 的 LUN file 指向 `/dev/nbdX`（已有逻辑）

**设备拔出时的操作**：
- 键盘/鼠标：`spawn_blocking` task 中 evdev 读取返回错误 → 自然退出 → 释放 fd
- U 盘：umount → NBD_DISCONNECT → 清 LUN file

---

### 3.5 udev 事件改造 (`usb-identify/src/udev_monitor.rs`) — 改动

**改动点**：
1. 接受 `UnboundedSender<DeviceEvent>`（mpmc channel），事件不再直接写 DeviceManager
2. 保留启动时枚举存量设备
3. `parse_device_info` 分类后构造对应的 `DeviceEvent` 变体

**启动顺序**（main.rs）：
```
1. Storage::open()
2. AuthService, AuditService, WhitelistManager, PolicyService...
3. ScanService, FileAccessEngine 实例化
4. GadgetManager::init_all() → 获取 HidgNodes
5. DeviceOrchestrator::new(...) 实例化
6. tokio::spawn(Orchestrator::run()) 启动编排循环
7. tokio::spawn_blocking(udev_monitor_loop(sender, shutdown))
8. TcpListener::bind() → 协议服务
```

---

## 4. 依赖变更

| Crate | 新增/改动 | 用途 |
|---|---|---|
| `hid-access` | 新增依赖 `evdev = "0.12"` | Linux evdev 设备读取，已在 RK3568/kernel 4.19 验证 |
| `usb-identify` | 新增依赖 `tokio::sync::mpsc` | Orchestrator 事件接收 |
| `usb-identify` | 新增模块 `orchestrator.rs` | 主编排器 |
| `hid-access` | 新增模块 `evdev_interceptor.rs` | 键盘拦截+转发 |
| `hid-access` | 新增模块 `mouse_forwarder.rs` | 鼠标转发 |
| `hid-access` | 新增模块 `hid_report.rs` | HID report 结构体+keycode 映射表 |
| `hid-access` | 新增模块 `hid_gadget.rs` | HID gadget configfs 配置+node 发现 |
| `file-access` | 改动 `gadget.rs` | 扩展 `init_all()` 包含 HID function |
| `app` | 改动 `main.rs` | 实例化新组件，调整启动顺序 |

---

## 5. 与已有模块的关系

| 已有模块 | 改动 | 说明 |
|---|---|---|
| `KeyboardChallenge` | **不改** | 状态机不变，由 `evdev_interceptor.rs` 驱动 |
| `UsbStateMachine` | **不改** | 状态机不变，Orchestrator 在各阶段调用 `transition()` |
| `FileAccessEngine` (NBD) | **不改** | 映射入口改为接受 MapContext 参数 |
| `ClamScanner / ScanService` | **不改** | Orchestrator 直接调用 |
| `WhitelistManager` | **不改** | Orchestrator 在分类后调用 `check()` |
| `AuditService` | **不改** | Orchestrator 和各拦截器调用 |
| `DeviceManager` | **小改** | 改为持有 `Sender`，收到事件即转发 |
| `GadgetManager` | **扩展** | 增加 `init_all()` 方法，内部调用 HID function 配置 |
| `RealMountOps` | **不改** | 由 Orchestrator 调用 |

---

## 6. 测试策略

| 测试目标 | 方式 |
|---|---|
| Orchestrator 路由逻辑 | 注入 mock Scanner/DeviceMapper，验证事件→正确处理链 |
| U 盘完整链路 | 单元测试：临时目录模拟 `/mnt/usb_raw`，mock ClamAV 返回 |
| KeyboardChallenge 驱动 | 已有 13 个状态机单元测试 + 新增 evdev 模拟事件序列测试 |
| 鼠标转发 | 模拟 evdev 事件 → 验证 MouseReport 输出 |
| HID report 格式 | 序列化往返 + 边界值测试 |
| Gadget HID 配置 | configfs mock 测试（不上真实 RK3568） |
| NBD 设备号池 | 分配/归还 + 池耗尽行为测试 |

---

## 7. 不在本设计范围

- ❌ 病毒扫描引擎本身（已有）
- ❌ 文件访问控制策略引擎（已有）
- ❌ 白名单管理（已有）
- ❌ 协议网关（已有）
- ❌ 数据库 schema 变更
- ❌ 管理端 UI
