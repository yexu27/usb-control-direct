# R02 ClamAV 病毒扫描可行性调研报告

> **调研日期**：2026-06-16
> **调研环境**：RK3568 / Ubuntu 22.04.4 LTS / kernel 4.19.232 / aarch64
> **调研范围**：ClamAV 安装、病毒库更新、扫描功能验证、性能与资源评估

---

## §1 安装与版本

### 1.1 安装方式

通过 Ubuntu 官方源在线安装：

```bash
apt-get install -y clamav clamav-daemon
```

| 组件 | 版本 |
|---|---|
| ClamAV Engine | 1.4.4 |
| clamscan（命令行扫描） | 1.4.4 |
| clamd（守护进程） | 1.4.4 |
| freshclam（病毒库更新） | 1.4.4 |
| 架构 | aarch64 |

**离线安装说明**：生产部署需提前下载 `.deb` 包及依赖，通过 `dpkg -i` 安装。本次调研使用在线安装验证功能可行性。

### 1.2 病毒库

| 库文件 | 大小 | 签名数 | 说明 |
|---|---|---|---|
| daily.cvd | 23 MB | 340,866（含 daily + bytecode 合计） | 安装时自动下载 |
| bytecode.cvd | 276 KB | 80 | freshclam 更新成功 |
| main.cvd | 未下载 | — | CDN 限速（HTTP 429），首次安装后 24h 内被限制 |

**生产影响**：main.cvd（~160MB）包含核心签名库。离线部署时必须预置 main.cvd + daily.cvd + bytecode.cvd 三个库文件。即使缺少 main.cvd，daily.cvd 仍包含 34 万条签名，EICAR 标准测试文件可正常检出。

---

## §2 功能验证

### 2.1 EICAR 标准检测

| 测试项 | 模式 | 结果 | 耗时 |
|---|---|---|---|
| EICAR 病毒测试文件 | clamscan（冷启动） | `Eicar-Test-Signature FOUND` ✅ | 55.0 秒 |
| 正常文本文件 | clamscan（冷启动） | `OK` ✅ | 53.7 秒 |
| EICAR 病毒测试文件 | clamdscan（守护进程） | `Eicar-Test-Signature FOUND` ✅ | 0.012 秒 |
| 正常文本文件 | clamdscan（守护进程） | `OK` ✅ | 0.049 秒 |

### 2.2 两种扫描模式对比

| 维度 | clamscan（独立进程） | clamdscan（守护进程客户端） |
|---|---|---|
| 工作方式 | 每次启动加载完整病毒库 | clamd 守护进程常驻内存，客户端通过 Unix socket 通信 |
| 单文件扫描延迟 | ~55 秒（病毒库加载开销） | <0.05 秒 |
| 适用场景 | 一次性批量扫描 | 实时逐文件扫描（USB 文件逐个过检） |
| 生产推荐 | ❌ 不适合实时场景 | ✅ 推荐 |

**结论**：装置端必须使用 clamd 守护进程模式。USB 文件扫描场景为逐文件实时检测，clamscan 每次 55 秒的冷启动开销不可接受。

---

## §3 资源占用

### 3.1 clamd 守护进程

| 指标 | 值 |
|---|---|
| 常驻内存（RSS） | ~650 MB |
| 启动加载病毒库耗时 | ~110 秒 |
| CPU（空闲时） | <1% |
| CPU（加载病毒库时） | ~100%（单核） |
| Unix socket 路径 | `/var/run/clamav/clamd.ctl` |
| 工作线程数 | 最多 12 线程（默认配置） |

### 3.2 RK3568 系统资源

| 资源 | 总量 | clamd 占用 | 剩余可用 |
|---|---|---|---|
| 内存 | 3.8 GiB | ~650 MB（17%） | ~3.1 GiB |
| 存储（病毒库） | 29 GB（可用 17 GB） | ~185 MB（三库齐全时） | 充足 |

**评估**：650MB 常驻内存占 RK3568 总内存 17%，剩余 3.1GiB 可满足其他业务模块运行。若未来 main.cvd 完整加载，内存可能上升至 ~800MB-1GB，仍在可接受范围。

---

## §4 集成方案建议

### 4.1 Rust 调用方式

| 方案 | 说明 | 推荐 |
|---|---|---|
| clamdscan 命令行 | `Command::new("clamdscan").arg(path)` | ✅ 简单可靠，推荐 P01 阶段使用 |
| clamd Unix socket 协议 | Rust 直接连接 `/var/run/clamav/clamd.ctl`，发送 `SCAN path` 指令 | ✅ 性能最优，推荐 P03 正式实现 |
| libclamav FFI | 通过 C FFI 调用 libclamav | ❌ 复杂度高，维护成本大，不推荐 |

### 4.2 扫描流程设计

```
USB 插入 → 挂载到 /mnt/usb_raw
  → 遍历文件列表
    → 逐文件发送到 clamd 扫描（SCAN 命令或 clamdscan）
      → 标记结果：clean / infected / failed
        → infected 文件在虚拟 exFAT 视图中标记为病毒禁止访问
```

### 4.3 clamd 生命周期管理

- 装置端启动时通过 systemd 拉起 clamd（设置 `enabled`）
- clamd 启动需 ~110 秒加载病毒库，必须在 USB 热插拔处理前完成
- 建议装置端主进程启动时检查 clamd socket 是否就绪，未就绪时等待或降级处理

---

## §5 风险残留

| 风险项 | 说明 | 建议处理时机 |
|---|---|---|
| main.cvd 离线预置 | 生产离线部署必须预置完整病毒库（main.cvd + daily.cvd + bytecode.cvd），需建立离线更新流程 | P01 部署方案设计 |
| 病毒库更新机制 | 装置端网络隔离时无法在线更新，需设计通过管理端推送更新包的离线更新通道 | P02/P03 |
| 常驻内存 ~650MB | 当前仅加载 daily.cvd，完整三库加载后预估 800MB-1GB | P03 资源优化 |
| clamd 启动延迟 110 秒 | USB 热插拔若发生在 clamd 未就绪时，需降级策略（如排队等待或拒绝映射） | P03 |
| 大文件扫描耗时 | 未验证 GB 级文件扫描时间，clamd 默认 MaxFileSize=25MB、MaxScanSize=100MB，超限文件需调整配置或跳过 | P03 性能测试 |
| 压缩包/嵌套扫描 | clamd 默认支持 ZIP/RAR/OLE2/PDF 等压缩包扫描，但嵌套深度和文件数有上限配置，需根据业务需求调整 | P03 |

---

## §6 结论

1. **ClamAV 1.4.4 在 RK3568 aarch64 上可正常安装和运行**，功能验证通过。
2. **必须使用 clamd 守护进程模式**，单文件扫描延迟 <0.05 秒，满足实时扫描需求。
3. **常驻内存 ~650MB**，RK3568（3.8GiB）可承受，但需关注完整病毒库加载后的增长。
4. **Rust 集成推荐**：P01 阶段使用 `clamdscan` 命令行方式，P03 阶段切换到 Unix socket 直连。
5. **离线部署**是生产环境主要挑战，需预置病毒库并设计更新通道。

---

## §7 引用

- `docs/architecture/02-技术选型.md` §2.4 病毒扫描引擎
- `docs/architecture/03-模块划分.md` §S03
- `docs/architecture/11-风险识别与应对.md` §R02
