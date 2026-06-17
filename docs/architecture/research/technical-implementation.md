# usb-guard 技术实现说明

## 技术栈总览

```
┌─────────────────────────────────────────────┐
│              应用层 (Rust 2021)               │
│  filter + config + policy + state            │
├─────────────────────────────────────────────┤
│           虚拟文件系统层                      │
│  exFAT 规范实现 / FAT32 规范实现             │
│  目录项解析/构造、FAT 表、Bitmap、Upcase     │
├─────────────────────────────────────────────┤
│            块设备层                           │
│  NBD 协议 (socketpair + ioctl)              │
│  BlockDevice trait 抽象                      │
├─────────────────────────────────────────────┤
│          Linux 内核接口层                     │
│  udev (设备监听)    nix (mount/umount)       │
│  ConfigFS (Gadget)  libc (ioctl/poll)        │
│  /sys  /proc  /dev                           │
├─────────────────────────────────────────────┤
│            硬件层                             │
│  RK3568 DWC3 UDC    USB OTG    USB Host     │
└─────────────────────────────────────────────┘
```

整个项目无外部 C 库绑定（除 libc），纯 Rust 实现从 USB 设备检测到文件系统构造到块设备协议服务的完整栈。

---

## 1. monitor.rs — USB 守护进程

**职责**：udev 设备监听、自动挂载、管控 session 生命周期、被禁写入检测与重建。

| 技术 | 用途 |
|------|------|
| **udev** (crate `udev` 0.8) | 监听 Linux 内核 uevent，检测 USB block 设备热插拔（过滤条件：subsystem=block, ID_BUS=usb, DEVTYPE=partition/disk） |
| **udev Enumerator** | 启动时扫描 `/sys/block/sd*`，枚举已插入的 USB 存储设备 |
| **libc::poll()** | 非阻塞 I/O 多路复用，等待 udev socket fd 上的事件，同时支持 300ms 超时检查 Ctrl+C 和 blocked_signal |
| **nix::mount::mount()** | Linux mount 系统调用，手动挂载 USB 分区到 `/mnt/usb-guard/<devname>` |
| **nix::mount::umount2()** | 带 MNT_DETACH 标志的懒卸载，确保不阻塞 |
| **blkid** (外部命令调用) | `blkid -o value -s TYPE <dev>` 检测分区文件系统类型（ntfs/vfat/ext4） |
| **/proc/mounts 解析** | 逐行解析查找设备挂载点，判断 udisks2 是否已自动挂载 |
| **/sys/block/sdX/sdXN/size** | 读取分区扇区数 × 512 得到字节大小，用于选择最大数据分区 |
| **Arc\<AtomicBool\>** | `blocked_signal`：NBD server 线程写入 → monitor 主循环读取，无锁跨线程信号 |
| **ctrlc** (crate 3.x) | 注册 SIGINT/SIGTERM handler，设置 stop 标志触发优雅退出 |

### 被禁写入重建流程的技术实现

```
blocked_signal.swap(false) == true
  → MassStorageLun::discover() + detach()     // 写 ConfigFS 解绑 UDC
  → session.nbd.disconnect()                  // ioctl NBD_DISCONNECT
  → drop(session)                             // 释放所有句柄
  → thread::sleep(3s)                         // 等 Windows 释放页面缓存
  → start_session()                           // 重新扫描源目录 + 构建 VirtExFat + NBD + Gadget
```

---

## 2. exfat.rs — 虚拟 exFAT 文件系统（默认，1436 行）

**职责**：在内存中构建完整的 exFAT 磁盘镜像，提供扇区级读写，处理文件过滤和写入透传。

### 磁盘布局构造

| 技术 | 用途 |
|------|------|
| **exFAT 规范** (Microsoft) | 完整实现磁盘布局：MBR(LBA 0) → 空隙(2048扇区偏移) → Main Boot Region(12扇区) → Backup Boot Region(12扇区) → FAT → Cluster Heap |
| **MBR 分区表** | 扇区 0 写 MBR，分区类型 0x07(exFAT)，起始偏移 2048 扇区 |
| **Boot Sector** | 构造 512 字节引导扇区：JumpBoot + "EXFAT   " 签名 + PartitionOffset + VolumeLength + FatOffset + ClusterHeapOffset + ClusterCount + RootDirCluster + SectorsPerClusterShift(=3, 即 4KB 簇) |
| **Boot Checksum** | 对 Boot Region 前 11 个扇区计算 32 位循环右移校验和（跳过 VolumeFlags 和 PercentInUse 字段），填充第 12 扇区 |
| **Allocation Bitmap** | 每 bit 对应一个簇的使用状态，按簇分配存储 |
| **Upcase Table** | 65536 个 UTF-16 大写映射，压缩编码（连续 identity 映射段用 0xFFFF + count 表示），含 32 位数据校验和 |
| **FAT 表** | 每个簇 4 字节 LE，0xFFFFFFF8=媒体描述符，0xFFFFFFFF=链尾，其余=下一簇号 |

### 目录项构造

| 技术 | 用途 |
|------|------|
| **File Entry (0x85)** | 主目录项：SecondaryCount + FileAttributes + Timestamps |
| **Stream Extension (0xC0)** | 流扩展项：NameLength + NameHash + FirstCluster + DataLength(u64，支持 >4GB) |
| **FileName Entry (0xC1)** | 文件名项：每项最多 15 个 UTF-16 字符，按需多项 |
| **SetChecksum** | 16 位循环右移校验和，覆盖整个 entry set（跳过自身 2 字节） |
| **NameHash** | 对大写化后的文件名计算 16 位哈希，用于 exFAT 快速目录查找 |
| **Volume Label (0x83)** | 卷标目录项，UTF-16 编码，最多 11 字符 |
| **Bitmap Entry (0x81)** | 指向 Allocation Bitmap 的首簇和大小 |
| **Upcase Entry (0x82)** | 指向 Upcase Table 的首簇、大小和校验和 |

### 读写引擎

| 技术 | 用途 |
|------|------|
| **HashMap\<u32, ClusterContent\>** | 簇号 → 内容的内存映射，四种类型：Directory{data} / FileData{real_path,offset,length,blocked} / RawData{data} / Empty |
| **LBA 地址分解** | `lba → MBR / Boot / FAT / Cluster Heap` 四区域路由，heap 内再分解为 `cluster + sector_in_cluster` |
| **std::fs::File (read+seek)** | FileData 类型：按 real_path + offset 从真实 U 盘文件读取对应扇区数据 |
| **std::fs::File (write+seek+sync_all)** | 写入透传：数据直接写真实文件 + fsync 确保落盘 |
| **set_len()** | 写入后按目录项记录的逻辑大小截断文件，去除 Windows 写入的尾部 NUL 填充 |
| **pending_data: HashMap\<u32, HashMap\<u32, Vec\<u8\>\>\>** | 乱序写入缓存：Windows 先写数据簇后写目录项，缓存数据等目录项到达时 flush |

### 目录变更同步（sync_dir_changes）

| 技术 | 用途 |
|------|------|
| **目录项解析器** (parse_exfat_dir_entries) | 解析 raw bytes 为结构化条目：遍历 32 字节 entries，识别 0x85+0xC0+0xC1 entry set，提取 name/cluster/size |
| **old vs new diff** | 按名称和簇号建立双向 HashMap，对比检测：新文件、重命名（同簇不同名）、删除（旧有新无） |
| **Filter::judge()** | 对新文件名/重命名目标调用过滤引擎，判定 Allow 或 Block |
| **blocked 处理** | demo 曾通过 `fs::remove_file` 处理被拦截写入；正式方案不得删除或修改真实 U 盘中的病毒文件。病毒文件在虚拟 exFAT 视图中以 `[病毒禁止访问]` 前缀和 DataLength=0 表达，并保护目录项不被受控主机删除、重命名或修改 |
| **initial_dir_clusters 还原** | 目录写入后，用快照中被拦截文件对应的字节范围覆写回 new_dir，保护其目录项不被 Windows 修改 |
| **strip_block_prefix()** | 去除 "[病毒]"/"[不安全]" 等前缀和 " - 副本" 后缀，还原原始文件名用于过滤判定 |

---

## 3. vfat.rs — 虚拟 FAT32 文件系统（1446 行）

**职责**：与 exfat.rs 平行，提供 FAT32 格式的虚拟文件系统，兼容旧设备。

| 技术 | 与 exFAT 的差异 |
|------|------|
| **FAT32 BPB** | 引导参数块：BytsPerSec=512, SecPerClus=8, NumFATs=2, FATSz32 计算 |
| **FSInfo 扇区** | 扇区 1：Free_Count + Nxt_Free 提示，签名 0x41615252 / 0x61417272 |
| **LFN 长文件名** | 0x40\|seq 起始 + 每项 13 个 UTF-16 字符 + SFN 8.3 短文件名 + 校验和 |
| **MBR 偏移** | 仅 1 扇区（vs exFAT 的 2048 扇区） |
| **文件大小** | u32，最大 4GB（vs exFAT 的 u64） |
| **双 FAT 表** | FAT1 + FAT2 镜像备份 |

其余写入透传、目录同步、过滤逻辑与 exfat.rs 一致。

---

## 4. nbd.rs — NBD 协议服务器（384 行）

**职责**：将内存中的虚拟文件系统暴露为 Linux 块设备 `/dev/nbdN`。

| 技术 | 用途 |
|------|------|
| **NBD 协议** (Linux Network Block Device) | 用户态程序通过 socket 为内核 nbd 驱动提供块存储后端 |
| **UnixStream::pair()** | 创建 Unix domain socket 对：一端交给内核（NBD_SET_SOCK），一端 server 线程持有 |
| **libc::ioctl — NBD_SET_SOCK** | 将 socket fd 交给内核 nbd 驱动 |
| **libc::ioctl — NBD_SET_BLKSIZE** | 设置块大小为 512 字节 |
| **libc::ioctl — NBD_SET_SIZE_BLOCKS** | 设置设备总扇区数 |
| **libc::ioctl — NBD_SET_FLAGS** | 设置标志：HAS_FLAGS \| SEND_FLUSH \| (READ_ONLY) |
| **libc::ioctl — NBD_DO_IT** | 阻塞调用，内核开始通过 socket 发送 I/O 请求，直到 DISCONNECT |
| **双线程** | `nbd-ioctl` 线程持有设备 fd + socket 内核端（NBD_DO_IT 阻塞）；`nbd-server` 线程在用户端 socket 上处理请求 |
| **NBD 请求解析** | 28 字节大端序：magic(0x25609513) + cmd_type + handle[8] + from(u64) + len(u32) |
| **NBD 应答构造** | 16 字节大端序：magic(0x67446698) + error(u32) + handle[8]，READ 还附带数据 |
| **命令处理** | READ → read_range 拼装跨扇区数据；WRITE → write_range + blocked 检测；FLUSH/TRIM → Ok；DISC → 退出 |
| **BlockedWrite 信号** | WRITE 返回 EACCES 时同步设置 blocked_signal，通知 monitor 触发重建 |
| **/sys/block/nbdN/pid** | 检测设备占用，stale pid 自动回收（NBD_DISCONNECT + CLEAR_SOCK + CLEAR_QUE） |
| **/proc/mounts** | 检查 `/dev/nbdN` 或 `/dev/nbdNpM` 是否被挂载，跳过占用设备 |

---

## 5. gadget.rs — USB Gadget 管理（164 行）

**职责**：通过 Linux ConfigFS 操作 USB Gadget mass_storage LUN，控制 Windows 端看到的虚拟 U 盘。

| 技术 | 用途 |
|------|------|
| **Linux USB Gadget ConfigFS** | `/sys/kernel/config/usb_gadget/` 虚拟文件系统，用户态配置 USB 设备功能 |
| **复用板厂 gadget** | 板厂 BSP 预配 `rockchip` gadget 含 mass_storage.0/lun.0，不能创建新 gadget（Android kobject 单例限制） |
| **lun.0/file** | 写入 `/dev/nbdN` 路径 = 插入虚拟 U 盘；写空 = 弹出 |
| **lun.0/ro** | "1" = 只读，"0" = 读写 |
| **lun.0/removable** | "1" = 可移动设备（Windows 识别为 U 盘而非硬盘） |
| **lun.0/nofua** | "1" = 禁用 Force Unit Access（性能优化） |
| **UDC 绑定** | 读写 `<gadget>/UDC` 文件：写入 "fcc00000.dwc3" = 绑定（Windows 枚举设备），写入 "\\n" = 解绑（Windows 看到设备拔出） |
| **/sys/class/udc/** | 遍历发现可用的 UDC 控制器名称 |
| **attach 流程** | unbind UDC → 清空 file → 设置 ro/removable/nofua/cdrom → 写入 backing → bind UDC |
| **trigger_media_change** | detach + sleep 50ms + attach，强制 Windows 重新读取文件系统 |

---

## 6. filter.rs — 文件过滤引擎（151 行）

**职责**：对文件名进行安全判定，返回 Allow 或 Block。

| 技术 | 用途 |
|------|------|
| **HashSet\<String\>** | O(1) 查找：病毒文件名集合、可执行扩展名集合、自定义黑名单集合 |
| **大小写不敏感** | 所有名称预转 lowercase 存入 HashSet，查询时也 lowercase |
| **扩展名提取** | `name.rsplit('.').next()` 取最后一个 `.` 后的部分 |
| **优先级链** | virus_names → executable_extensions → custom_blacklist → autorun.inf，命中即返回 |
| **Verdict 枚举** | `Allow` 或 `Block { display_name: String, reason: BlockReason }` |
| **display_name 前缀** | 根据 BlockReason 添加可配置前缀：`[病毒]`、`[不安全]`、`[自动运行已阻止]`、`[禁止类型]` |

---

## 7. config.rs — 配置管理（101 行）

| 技术 | 用途 |
|------|------|
| **serde + serde_json** | `#[derive(Deserialize)]` 自动从 JSON 反序列化为 `FilterConfig` 结构体 |
| **默认值** | `impl Default` 提供内置默认配置，文件缺失时 fallback |
| **路径** | `/etc/usb-guard/filter.json`，通过 `FilterConfig::default_path()` 获取 |

---

## 8. policy.rs — 策略管理（93 行）

| 技术 | 用途 |
|------|------|
| **运行时文件存储** | `/run/usb-guard/policy`，纯文本存储策略字符串 |
| **std::fs::create_dir_all** | 自动创建 `/run/usb-guard/` 目录 |
| **FromStr trait** | `"deny"` / `"allow-ro"` / `"allow-rw"` 解析为 `Policy` 枚举 |
| **Policy 枚举方法** | `allows_attach()` 判断是否暴露设备，`is_readonly()` 判断 LUN 只读标志 |

---

## 9. block_device.rs — 块设备抽象（16 行）

| 技术 | 用途 |
|------|------|
| **trait BlockDevice: Send** | 512 字节扇区级抽象接口 |
| **动态分发** | `Arc<Mutex<dyn BlockDevice>>` 让 NBD server 不关心底层是 exFAT 还是 FAT32 |
| **默认方法** | `take_blocked_write_flag()` 默认返回 false，仅 VirtExFat 覆写 |

---

## 10. 跨模块基础设施

| 技术 | 用途 |
|------|------|
| **Rust 2021 edition, MSRV 1.75** | 内存安全，无 GC，零开销抽象 |
| **Arc\<Mutex\<dyn BlockDevice\>\>** | NBD server 线程和 monitor 主线程共享虚拟块设备 |
| **Arc\<AtomicBool\>** (SeqCst) | 无锁跨线程信号：blocked_signal（NBD→monitor）、stop（ctrlc→主循环） |
| **tracing + tracing-subscriber** | 结构化日志，`RUST_LOG=info/debug` 环境变量控制级别和模块过滤 |
| **thiserror** | `#[derive(Error)]` 宏自动实现 Display + Error，11 个错误变体 |
| **anyhow** | 顶层命令函数用 `anyhow::Result<()>` 串联错误链 |
| **release profile** | `opt-level=3` + `lto="thin"` + `codegen-units=1` + `strip=true`，最小化二进制体积 |

---

## 依赖清单

| crate | 版本 | 用途 |
|-------|------|------|
| `anyhow` | 1 | 顶层错误处理 |
| `thiserror` | 1 | 自定义错误类型派生 |
| `tracing` | 0.1 | 结构化日志 API |
| `tracing-subscriber` | 0.3 (env-filter) | 日志输出 + 环境变量级别控制 |
| `udev` | 0.8 | Linux udev 设备监听 |
| `nix` | 0.27 (mount/fs/signal/user/ioctl) | Linux 系统调用封装 |
| `ctrlc` | 3 | Ctrl+C 信号处理 |
| `libc` | 0.2 | 底层 C ABI (ioctl/poll) |
| `serde` | 1 (derive) | 序列化框架 |
| `serde_json` | 1 | JSON 解析 |
| `tempfile` | 3.10.1 (dev) | 测试用临时文件 |
