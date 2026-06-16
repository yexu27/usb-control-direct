# R01 NBD 虚拟 exFAT 可行性调研报告

> **调研日期**：2026-06-16
> **调研环境**：RK3568 / Ubuntu 22.04.4 LTS / kernel 4.19.232 / aarch64
> **调研范围**：内核能力检查 + 架构链路可行性分析

---

## §1 RK3568 内核与系统能力检查结果

### 1.1 环境信息

| 维度 | 值 |
|---|---|
| 平台 | RK3568 (aarch64) |
| OS | Ubuntu 22.04.4 LTS (Jammy Jellyfish) |
| 内核 | Linux 4.19.232 SMP |
| 内存 | 3.8 GiB（可用 2.5 GiB） |
| 存储 | /dev/root 29G（已用 12G，可用 17G） |

### 1.2 NBD 块设备

| 检查项 | 结果 |
|---|---|
| `CONFIG_BLK_DEV_NBD` | `=y`（内核内置，非模块） |
| `/dev/nbd*` 设备数 | 22 个（nbd0-nbd15 + 分区设备） |
| nbd0 当前状态 | size=30629377 sectors, hw_sector_size=512（已有 backing 绑定） |
| `nbd-client` 用户态工具 | **未安装**（PoC/正式开发需安装或通过 ioctl 直接操作） |

**结论**：NBD 内核驱动已内置，设备节点可用。Rust NBD server 可通过 `socketpair` + `ioctl(NBD_SET_SOCK/NBD_SET_SIZE/NBD_DO_IT)` 直接操作内核 NBD 设备，无需依赖 `nbd-client`。

### 1.3 USB Gadget / configfs / f_mass_storage

| 检查项 | 结果 |
|---|---|
| configfs 挂载 | `configfs on /sys/kernel/config type configfs (rw)` |
| USB gadget 实例 | `/sys/kernel/config/usb_gadget/rockchip` |
| `CONFIG_USB_F_MASS_STORAGE` | `=y`（内核内置） |
| `CONFIG_USB_CONFIGFS_MASS_STORAGE` | `=y` |
| mass_storage function | `mass_storage.0`，已绑定到 config `b.1` |
| lun.0 当前状态 | file=`/tmp/fat32test.img`, removable=1, ro=0 |
| UDC | `fcc00000.dwc3` |
| HID function | `hid.usb0`, `hid.usb1`（已绑定） |

**结论**：USB gadget configfs + f_mass_storage 功能完整，已在运行状态。lun.0 当前绑定了一个 FAT32 测试镜像。正式开发时切换 lun.0 的 `file` 指向 `/dev/nbdX` 即可完成 NBD → mass_storage 链路。

### 1.4 文件系统挂载能力

| 文件系统 | 内核 /proc/filesystems | 用户态工具 | 可挂载 |
|---|---|---|---|
| FAT32 / vfat | ✅ `vfat` | — | ✅ 已验证 loop mount |
| exFAT | ✅ `exfat`（OOT 内核模块 insmod exfat.ko） | mkfs.exfat (exfatprogs 1.1.3) | ✅ 已验证 loop mount 读写正常 |
| NTFS | ✅ `ntfs`（只读） | ntfs-3g（读写 FUSE） | ✅ |
| ext2 | ✅ | — | ✅ |
| ext3 | ✅ | — | ✅ |
| ext4 | ✅ | — | ✅ |

**exFAT 入侧支持说明**：

通过 `insmod exfat.ko` 加载 OOT 内核模块后，exFAT 已出现在 `/proc/filesystems`，可正常挂载和读写。验证步骤：创建 64MB exFAT 镜像 → loop mount → 写入/读取文件 → 成功。

模块信息：`exfat 106496 字节`，加载后 `lsmod` 可见。装置端启动时需确保 `exfat.ko` 自动加载（通过 `/etc/modules` 或 systemd modules-load）。

**exFAT 入侧阻塞项已解除。**

### 1.5 环回挂载

| 检查项 | 结果 |
|---|---|
| vfat loop mount | ✅ 已验证 |
| loop 设备 | 可用 |

---

## §2 入侧文件系统兼容矩阵

| 文件系统 | 装置端处理 | 当前内核支持状态 |
|---|---|---|
| FAT32 / vfat | Linux 挂载层挂载到 `/mnt/usb_raw` 后作为目录树输入 | ✅ 可用 |
| exFAT | Linux 挂载层挂载到 `/mnt/usb_raw` 后作为目录树输入 | ✅ OOT 内核模块 exfat.ko 已验证可用 |
| NTFS | Linux 挂载层挂载到 `/mnt/usb_raw` 后作为目录树输入；读写能力依赖 ntfs-3g | ✅ 可用（ntfs-3g 已安装） |
| ext2 / ext3 / ext4 | Linux 挂载层挂载到 `/mnt/usb_raw` 后作为目录树输入 | ✅ 可用 |
| 其他文件系统 | 不纳入本版本目标支持范围；挂载失败时拒绝映射并记录 USB 审计日志 | — |

---

## §3 出侧虚拟 exFAT 约束（受控主机视角）

| 维度 | 处理规则 |
|---|---|
| 输出视图 | 统一生成虚拟 exFAT 块设备 |
| 超过 4GiB 单文件 | 必须正常表达文件大小并支持按需读取（exFAT 目录项 DataLength 为 u64） |
| 原始 ACL / 权限位 / 属主属组 / 特殊文件 | 虚拟 exFAT 视图不承载这些高级语义 |
| 病毒文件 | 目录项可见，文件名添加 `[病毒禁止访问]` 前缀，读取命中数据时返回 I/O error |
| 可执行程序 / 黑名单 / 自动读取阻断 | 目录项可见，读取命中数据时返回 I/O error |
| 白名单只读权限 | 由 f_mass_storage write-protect（`ro=1`）实现整盘写保护 |

---

## §4 架构链路可行性分析

### 4.1 完整数据链路

```
真实 U 盘 (/dev/sdX)
  → Linux 挂载层 (/mnt/usb_raw)
    → Rust NBD server（策略判断 + 虚拟 exFAT 扇区生成）
      → /dev/nbdX
        → f_mass_storage lun.0
          → 受控主机（Windows 免驱识别 exFAT）
```

### 4.2 各环节可行性

| 环节 | 可行性 | 依据 |
|---|---|---|
| Linux 挂载层 | ✅ 已验证 | vfat loop mount 成功；NTFS/ext2/3/4 内核支持已确认 |
| Rust NBD server → /dev/nbdX | ✅ 可行 | `CONFIG_BLK_DEV_NBD=y`，设备节点存在，ioctl 接口可用 |
| /dev/nbdX → f_mass_storage | ✅ 可行 | mass_storage.0/lun.0 已在运行，backing 可切换为任意块设备或文件 |
| f_mass_storage → 受控主机 | ✅ 已在使用 | UDC `fcc00000.dwc3` 已绑定，当前 lun.0 绑定 fat32test.img |

### 4.3 5 级策略短路链路

沿用既定设计：1=virus → 2=exec_control → 3=file_type_blacklist → 4=auto_read_control → 5=permission。

超过 4GiB 单文件遵循同一策略判断链路，非命中策略时正常读取，命中阻断策略时返回 I/O error。

不引入 FAT32 输出视图、size=0 病毒表达、超过 4GiB 拒绝展示等替代方案。

---

## §5 风险残留

| 风险项 | 说明 | 建议处理时机 |
|---|---|---|
| exFAT 入侧挂载 | ~~已解除~~ OOT 内核模块 exfat.ko 已验证可用，启动时需自动加载 | 部署脚本确保 insmod |
| Rust exFAT 扇区生成器复杂度 | 目录项、簇分配、bitmap 管理、upcase table、boot checksum 等 exFAT 规范细节较多，需从头实现 | P03 file-access 模块开发阶段 |
| >4GiB 文件端到端验证 | 需在真机上创建 >4GiB 源文件，经 NBD exFAT 映射后在 Windows 端验证大小和读取正确性 | P03 开发 + 硬件集成测试 |
| NBD READ 返回 I/O error 的受控主机行为 | Windows 对 mass_storage 设备 I/O error 的弹窗文案、重试策略和缓存行为需实测 | P03 硬件集成测试 |
| 大容量 U 盘首次扫描 + 映射总耗时 | 128GB+ U 盘的目录遍历 + ClamAV 扫描 + exFAT 元数据生成总耗时 | P03/P05 性能测试 |
| 长时间高并发读取稳定性 | NBD server 进程在持续 I/O 下的 CPU、内存和稳定性 | P03 压力测试 |

---

## §6 引用

- `docs/architecture/02-技术选型.md` §2.3 装置端文件访问控制实现
- `docs/architecture/03-模块划分.md` §S04
- `docs/architecture/05-数据流设计.md`
- `docs/architecture/11-风险识别与应对.md` §R01
