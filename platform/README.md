# USB 安全管理系统开发、部署与使用手册

本手册说明 USB 安全管理系统服务端和客户端的开发、编译、安装、测试及验收方法。

## 1. 系统组成

项目使用服务端 VM、RK3568 USB 管控装置、Windows 客户端开发测试机和 Windows 受控机。

| 节点 | 操作系统 | 作用 |
|---|---|---|
| 服务端 VM | Ubuntu 18.04 / x86_64 | 编辑和编译 Rust 服务端代码，交叉编译 ARM64 程序并生成 `.deb` 安装包 |
| RK3568 USB 管控装置 | Ubuntu 22.04.4 / Linux 4.19.232 / ARM64 | 安装和运行服务端，完成 USB、ClamAV、NBD、文件系统和 Gadget 装置端测试 |
| Windows 客户端开发测试机 | Windows 10/11 64 位 | 编辑、编译、安装和测试 Electron 管理客户端，通过 TCP/TLS 管理 RK3568 装置 |
| Windows 受控机 | Windows | 通过 USB 连接 RK3568 OTG 口，完成存储和 HID 受控访问测试及验收 |

物理连接关系：

```text
待管控 U 盘
    │ USB Host
    ▼
RK3568 USB 管控装置 ───── TCP/TLS ───── Windows 客户端开发测试机
    │ USB OTG
    ▼
Windows 受控机
```

## 2. 软件版本基线

| 软件或组件 | 版本或要求 |
|---|---|
| 产品版本 | 由正式发布构建参数确定；本文以 3.0.1 和 3.0.2 为示例 |
| RK3568 CPU 架构 | ARM64 / aarch64 |
| RK3568 操作系统 | Ubuntu 22.04.4 LTS |
| RK3568 Linux 内核 | 4.19.232 |
| Rust | 1.96 |
| Rust target | `aarch64-unknown-linux-gnu` |
| 服务端构建 VM | Ubuntu 18.04 x86_64 |
| 交叉编译工具链 | Buildroot aarch64 GNU 工具链及对应 sysroot |
| ClamAV | 1.4.4 已完成 RK3568 验证 |
| Windows 客户端开发测试机 | Windows 10/11 64 位 |
| Windows 受控机 | 项目支持的 Windows 版本 |
| Node.js | 20 LTS |
| Electron | 32 |
| Vue | 3.4 |
| TypeScript | 5 |

服务端交付版本由 `build-deb.sh` 和 `build-bin.sh` 的参数注入，不通过修改业务代码或 Rust workspace 版本出包。客户端版本由 `client/package.json` 管理。

## 3. 目录结构

```text
platform/
├── README.md
└── rk3568/
    └── module/
        ├── exfat.ko
        ├── nbd.ko
        └── usb_f_hid.ko
```

三个内核模块的服务端 VM 源码目录及用途：

| 文件 | 模块名 | 服务端 VM 源码目录 | 用途 | 内核基线 |
|---|---|---|---|---|
| `exfat.ko` | `exfat` | `/root/exfat_build/` | 提供真实 U 盘 exFAT 文件系统挂载能力 | `4.19.232 SMP mod_unload aarch64` |
| `nbd.ko` | `nbd` | `/root/nbd_build/` | 提供 `/dev/nbdX`，供服务端发布虚拟块设备 | `4.19.232 SMP mod_unload aarch64` |
| `usb_f_hid.ko` | `usb_f_hid` | `/root/f_hid_build/` | 提供 USB Gadget 键盘、鼠标 HID function | `4.19.232 SMP mod_unload aarch64` |

`platform/rk3568/module/` 保存与目标内核基线匹配的交付模块，模块源码及重新编译所需文件保存在上表对应的服务端 VM 目录中。

## 4. RK3568 准备与安装

### 4.1 准备板卡和烧写工具

准备以下硬件和资料：

- RK3568 板卡、电源和串口调试线；
- 支持数据传输的 USB Host、USB OTG 线缆；
- Windows 或 Linux 烧写主机；
- 项目百度网盘中的 `ubuntu22.img`；
- 百度网盘中与镜像配套的《RK3568 烧写手册》；
- 烧写手册指定的驱动、烧写工具和连接线。

按照《RK3568 烧写手册》完成镜像烧写。烧写前确认板卡型号、硬件版本和存储介质，备份板上已有数据。

### 4.2 检查系统基线

RK3568 启动后执行：

```bash
uname -m
uname -r
. /etc/os-release
printf 'OS=%s VERSION=%s\n' "$ID" "$VERSION_ID"
df -h
ip address
```

预期结果：

```text
架构：aarch64
系统：Ubuntu 22.04
内核：4.19.232
```

内核版本不匹配时不能加载 `platform/rk3568/module/` 中的 `.ko`。

### 4.3 安装内核模块

内核模块目录：

| 位置 | 目录 |
|---|---|
| 服务端 VM 仓库 | `platform/rk3568/module/` |
| RK3568 临时安装目录 | `/tmp/usb-control-module/` |

将 `exfat.ko`、`nbd.ko` 和 `usb_f_hid.ko` 放入 RK3568 的 `/tmp/usb-control-module/` 后执行安装。

在 RK3568 上检查模块信息：

```bash
cd /tmp/usb-control-module
uname -r
modinfo ./exfat.ko | grep -E '^(name|depends|vermagic):'
modinfo ./nbd.ko | grep -E '^(name|depends|vermagic):'
modinfo ./usb_f_hid.ko | grep -E '^(name|depends|vermagic):'
```

三个模块的 `vermagic` 应为：

```text
4.19.232 SMP mod_unload aarch64
```

安装到当前内核模块目录：

```bash
KERNEL_RELEASE="$(uname -r)"
sudo install -d "/lib/modules/$KERNEL_RELEASE/extra/usb-control"
sudo install -m 0644 exfat.ko nbd.ko usb_f_hid.ko \
  "/lib/modules/$KERNEL_RELEASE/extra/usb-control/"
sudo depmod -a
```

加载模块：

```bash
sudo modprobe exfat
sudo modprobe nbd max_part=0
sudo modprobe usb_f_hid
```

配置开机加载：

```bash
printf '%s\n' exfat nbd usb_f_hid | \
  sudo tee /etc/modules-load.d/usb-control.conf >/dev/null
printf '%s\n' 'options nbd max_part=0' | \
  sudo tee /etc/modprobe.d/usb-control.conf >/dev/null
```

检查模块状态：

```bash
lsmod | grep -E '^(exfat|nbd|usb_f_hid)\b'
grep -w exfat /proc/filesystems
cat /sys/module/nbd/parameters/max_part
ls -l /dev/nbd* 2>/dev/null
dmesg | tail -n 100
```

`/sys/module/nbd/parameters/max_part` 应为 `0`。生产环境使用 `nbd.max_part=0`，避免内核为虚拟介质创建 `nbdXpN` 分区设备并触发重复 udev 事件。

配置完成后重启 RK3568，并再次检查模块和 `max_part`。禁止强制加载 `vermagic` 不匹配的模块。

### 4.4 检查 USB Gadget 和 UDC

```bash
test -d /sys/kernel/config || sudo mount -t configfs none /sys/kernel/config
test -d /sys/kernel/config/usb_gadget
ls /sys/class/udc
```

服务端默认 UDC 名称为：

```text
fcc00000.dwc3
```

若 `ls /sys/class/udc` 显示的名称不同，安装服务端后修改：

```text
/etc/usb-control/usb-control.toml
```

对应配置项：

```toml
[gadget]
udc = "fcc00000.dwc3"
```

配置值应与 `ls /sys/class/udc` 的实际输出一致。

### 4.5 检查并安装 ClamAV

`usb-control` 依赖 RK3568 系统提供 ClamAV。安装前先检查系统中已有的 ClamAV 运行环境。

检查操作系统、架构和已安装软件包：

```bash
uname -m
. /etc/os-release
printf 'OS=%s VERSION=%s\n' "$ID" "$VERSION_ID"
dpkg-query -W -f='${binary:Package}\t${Version}\t${db:Status-Status}\n' \
  clamav \
  clamav-daemon \
  clamav-freshclam \
  clamdscan 2>/dev/null || true
```

检查程序、systemd 服务和病毒库：

```bash
command -v clamscan || true
command -v clamdscan || true
command -v freshclam || true
test -x /usr/sbin/clamd && echo 'clamd: OK' || echo 'clamd: MISSING'
systemctl is-active clamav-daemon.service || true
systemctl is-active clamav-freshclam.service || true
test -S /run/clamav/clamd.ctl && echo 'clamd socket: OK' || echo 'clamd socket: MISSING'

for DATABASE in main daily bytecode; do
  if test -r "/var/lib/clamav/$DATABASE.cvd" || \
     test -r "/var/lib/clamav/$DATABASE.cld"; then
    echo "$DATABASE database: OK"
  else
    echo "$DATABASE database: MISSING"
  fi
done
```

ClamAV 环境符合以下条件时不需要重新安装：

- RK3568 为 `aarch64`，操作系统为 Ubuntu 22.04；
- `clamav`、`clamav-daemon`、`clamav-freshclam`、`clamdscan` 均为 `installed`；
- `/usr/sbin/clamd`、`clamscan`、`clamdscan`、`freshclam` 均可用；
- `clamav-daemon.service` 和 `clamav-freshclam.service` 正常运行；
- `/run/clamav/clamd.ctl` 存在；
- `/var/lib/clamav/` 中 main、daily、bytecode 三组病毒库均存在。

任一条件不满足时，执行后续安装和初始化步骤。

#### 4.5.1 ClamAV 软件包及依赖

项目直接使用以下 Ubuntu 软件包：

| 软件包 | 提供内容 | 项目用途 |
|---|---|---|
| `clamav` | `clamscan` 和扫描引擎命令 | 版本检查和基础扫描能力 |
| `clamav-daemon` | `/usr/sbin/clamd`、`clamav-daemon.service` | 常驻加载病毒库并执行扫描 |
| `clamdscan` | `/usr/bin/clamdscan` | `usb-control` 调用 clamd 的命令行客户端 |
| `clamav-freshclam` | `freshclam`、`clamav-freshclam.service` | 首次下载及自动更新病毒库 |

APT 会自动安装这些软件包所需的系统依赖，包括 `clamav-base`、`libclamav12`、`libcurl4`、`libjson-c5`、`libssl3`、`zlib1g`、`libsystemd0`、`procps`、`logrotate` 和 `ca-certificates`。不需要逐个手工安装这些传递依赖。

#### 4.5.2 安装 ClamAV

启用 Ubuntu `universe` 软件源并更新软件包索引：

```bash
sudo apt-get update
sudo apt-get install -y software-properties-common ca-certificates
sudo add-apt-repository -y universe
sudo apt-get update
```

检查软件源提供的版本：

```bash
apt-cache policy clamav clamav-daemon clamav-freshclam clamdscan
```

项目已在 Ubuntu 22.04 ARM64 的 ClamAV 1.4.4 上完成验证。安装项目直接依赖的软件包：

```bash
sudo apt-get install -y \
  clamav \
  clamav-daemon \
  clamav-freshclam \
  clamdscan
```

检查安装结果：

```bash
dpkg-query -W \
  clamav \
  clamav-daemon \
  clamav-freshclam \
  clamdscan
command -v clamscan
command -v clamdscan
command -v freshclam
test -x /usr/sbin/clamd
```

#### 4.5.3 下载病毒库

首次下载病毒库前停止自动更新服务和扫描服务：

```bash
sudo systemctl stop clamav-freshclam.service
sudo systemctl stop clamav-daemon.service
sudo install -d -m 0755 -o clamav -g clamav /var/lib/clamav
sudo freshclam
sudo chown -R clamav:clamav /var/lib/clamav
```

若 `freshclam` 返回 HTTP 429，应停止重复执行并等待 ClamAV 下载服务解除限速，不使用 `wget` 或 `curl` 直接抓取病毒库文件。

`freshclam` 负责从 ClamAV 官方病毒库服务下载和更新病毒库。项目要求 `/var/lib/clamav/` 中同时存在 main、daily 和 bytecode 三组病毒库，每组可以是 `.cvd` 或 `.cld` 文件。

```bash
for DATABASE in main daily bytecode; do
  if test -r "/var/lib/clamav/$DATABASE.cvd" || \
     test -r "/var/lib/clamav/$DATABASE.cld"; then
    echo "$DATABASE database: OK"
  else
    echo "$DATABASE database: MISSING" >&2
    exit 1
  fi
done
```

#### 4.5.4 配置并启动服务

检查 `/etc/clamav/clamd.conf` 中的运行路径：

```bash
grep -E '^(User|LocalSocket|DatabaseDirectory)[[:space:]]' \
  /etc/clamav/clamd.conf
```

配置应满足：

```text
User clamav
LocalSocket /run/clamav/clamd.ctl
DatabaseDirectory /var/lib/clamav
```

Ubuntu 配置中的 `/var/run/clamav/clamd.ctl` 与 `/run/clamav/clamd.ctl` 等价。若配置值不同，修改 `/etc/clamav/clamd.conf` 后再启动服务。

启用病毒库自动更新服务和扫描服务：

```bash
sudo systemctl enable --now clamav-freshclam.service
sudo systemctl enable --now clamav-daemon.service
```

加载完整病毒库需要一定时间。服务启动后检查运行状态、Unix socket 和版本：

```bash
systemctl is-active clamav-freshclam.service
systemctl is-active clamav-daemon.service
systemctl status clamav-daemon.service --no-pager
test -S /run/clamav/clamd.ctl
sudo clamdscan --version
sudo freshclam --version
```

#### 4.5.5 验证病毒检测

执行 EICAR 标准测试文件检测：

```bash
EICAR_FILE="$(mktemp /tmp/usb-control-eicar.XXXXXX)"
printf '%s\n' \
  'X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*' \
  > "$EICAR_FILE"
if sudo clamdscan --infected --no-summary "$EICAR_FILE"; then
  SCAN_RESULT=0
else
  SCAN_RESULT="$?"
fi
rm -f "$EICAR_FILE"
test "$SCAN_RESULT" -eq 1
```

检测输出应包含 `Eicar-Test-Signature FOUND`。ClamAV 安装、三组病毒库、`clamav-daemon.service` 和 `/run/clamav/clamd.ctl` 全部正常后，再安装 `usb-control` 服务端。

## 5. 服务端构建 VM

服务端项目目录为 `/root/work/code/usb-control-direct`，服务端代码位于其 `server/` 子目录。代码编辑、构建前检查、ARM64 交叉编译和安装包构建均在服务端 VM 完成。

### 5.1 VM 配置

已验证的服务端构建环境为 Ubuntu 18.04 x86_64。建议 VM 至少配置：

| 资源 | 建议配置 |
|---|---|
| CPU | 4 核 |
| 内存 | 8 GB |
| 磁盘 | 50 GB 可用空间 |
| 网络 | 可访问代码仓库和 Rust 依赖源，或配置完整离线缓存 |

基础工具：

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential ca-certificates curl file git openssl pkg-config \
  protobuf-compiler dpkg-dev libudev-dev
```

正式 `.deb` 构建还要求以下命令可用：

```bash
command -v cargo
command -v dpkg-deb
command -v sha256sum
command -v openssl
command -v pkg-config
test -x /home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/bin/aarch64-buildroot-linux-gnu-gcc
```

### 5.2 安装 Rust 1.96

在服务端 VM 的仓库根目录执行：

```bash
cd /root/work/code/usb-control-direct
rustup toolchain install 1.96 \
  --profile minimal \
  --component rustfmt \
  --component clippy \
  --target aarch64-unknown-linux-gnu
```

检查：

```bash
(
  cd /root/work/code/usb-control-direct/server
  rustc --version
  cargo --version
  rustup target list --installed | grep aarch64-unknown-linux-gnu
)
```

`server/rust-toolchain.toml` 固定 Rust 1.96，并声明 `rustfmt`、`clippy` 和 ARM64 target。

### 5.3 准备 Buildroot 工具链和 sysroot

VM 使用 RK3568 Buildroot 工程生成的工具链和 sysroot：

| 项目 | 路径 |
|---|---|
| Buildroot host | `/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host` |
| 交叉编译器 | `/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/bin/aarch64-buildroot-linux-gnu-gcc` |
| sysroot | `/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/aarch64-buildroot-linux-gnu/sysroot` |

该 sysroot 提供 RK3568 目标环境的头文件、运行库和 `libudev`。

在执行交叉编译的终端中配置：

```bash
export SYSROOT=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/aarch64-buildroot-linux-gnu/sysroot
export TOOLCHAIN=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/bin
export PATH="$TOOLCHAIN:$PATH"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$TOOLCHAIN/aarch64-buildroot-linux-gnu-gcc"
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_SYSROOT_DIR="$SYSROOT"
export PKG_CONFIG_PATH="$SYSROOT/usr/lib/pkgconfig"
export RUSTFLAGS="-C link-args=--sysroot=$SYSROOT"
```

检查：

```bash
"$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER" --version
pkg-config --modversion libudev
test -e "$SYSROOT/usr/include/libudev.h"
```

### 5.4 服务端代码检查

```bash
(
  cd /root/work/code/usb-control-direct/server
  unset CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER
  unset PKG_CONFIG_ALLOW_CROSS PKG_CONFIG_SYSROOT_DIR PKG_CONFIG_PATH
  unset RUSTFLAGS
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
)
```

此处的 `cargo test` 属于构建前代码检查。USB Host、USB Gadget、configfs、NBD、真实文件系统挂载和 ClamAV 在 RK3568 上测试，并结合 Windows 受控机完成验收。

### 5.5 编译 ARM64 二进制

```bash
cd /root/work/code/usb-control-direct/server
source ~/.cargo/env
export SYSROOT=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/aarch64-buildroot-linux-gnu/sysroot
export TOOLCHAIN=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/bin
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$TOOLCHAIN/aarch64-buildroot-linux-gnu-gcc"
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_SYSROOT_DIR="$SYSROOT"
export PKG_CONFIG_PATH="$SYSROOT/usr/lib/pkgconfig"
export RUSTFLAGS="-C link-args=--sysroot=$SYSROOT"
cargo build --release --target aarch64-unknown-linux-gnu \
  -p usb-control-app \
  -p usb-control-db-migrate
```

输出：

```text
server/target/aarch64-unknown-linux-gnu/release/usb-control
server/target/aarch64-unknown-linux-gnu/release/usb-control-db-migrate
```

检查目标架构：

```bash
file /root/work/code/usb-control-direct/server/target/aarch64-unknown-linux-gnu/release/usb-control
file /root/work/code/usb-control-direct/server/target/aarch64-unknown-linux-gnu/release/usb-control-db-migrate
```

结果应为 ARM aarch64 ELF。

### 5.6 生成 ARM64 `.deb`

正式打包入口：

```text
server/deploy/build-deb.sh
```

配置 RK3568 Buildroot 工具链和 sysroot：

```bash
cd /root/work/code/usb-control-direct
source ~/.cargo/env
export SYSROOT=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/aarch64-buildroot-linux-gnu/sysroot
export TOOLCHAIN=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/bin
export USB_CONTROL_AARCH64_SYSROOT="$SYSROOT"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$TOOLCHAIN/aarch64-buildroot-linux-gnu-gcc"
export PKG_CONFIG=pkg-config
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_SYSROOT_DIR="$SYSROOT"
export PKG_CONFIG_PATH="$SYSROOT/usr/lib/pkgconfig"
```

打包前确认以下发布资源存在：

```text
deploy/assets/tls/server.crt
deploy/assets/tls/server.key
deploy/assets/tls/server.crt.sha256
deploy/assets/keys/license_verify.pub
deploy/assets/keys/sm4_policy.key
deploy/assets/keys/sm2_policy.key
deploy/assets/keys/sm2_policy.pub
deploy/assets/keys/upgrade_verify.pub
deploy/assets/keys/upgrade_verify.id
```

生成在线升级包时还需要升级签名私钥：

```text
deploy/assets/keys/upgrade_sign.key
```

`upgrade_sign.key` 仅在受控发布环境中用于签名，不会写入 DEB 或 `.bin`。DEB 只携带对应的 `upgrade_verify.pub` 和 `upgrade_verify.id`，供 RK3568 验证在线升级包。

执行：

```bash
bash server/deploy/build-deb.sh 3.0.1
bash server/deploy/build-deb.sh 3.0.2
```

输出：

```text
server/deploy/build/out/usb-control_V3.0.1_arm64.deb
server/deploy/build/out/usb-control_V3.0.1_arm64.deb.sha256
server/deploy/build/out/usb-control_V3.0.2_arm64.deb
server/deploy/build/out/usb-control_V3.0.2_arm64.deb.sha256
```

`build-deb.sh` 的参数就是发布版本。相同源码可以分别生成 3.0.1 和 3.0.2，不需要修改 `server/Cargo.toml` 或业务代码。

生成由 V3.0.1 在线升级到 V3.0.2 的签名 `.bin`：

```bash
bash server/deploy/build-bin.sh 3.0.2 3.0.1 1
```

三个参数依次是目标版本、允许升级的最低当前版本和升级前数据库 Schema 版本。`schema-from` 必须根据数据库兼容性显式填写，构建脚本不会根据产品版本号推断 Schema。输出：

```text
server/deploy/build/out/usb-control_V3.0.2_arm64.bin
server/deploy/build/out/usb-control_V3.0.2_arm64.bin.sha256
```

`.bin` 是 Windows Client 在线上传的签名封装，内部唯一安装载荷仍是同版本正式 DEB。

检查安装包：

```bash
sha256sum server/deploy/build/out/usb-control_V3.0.1_arm64.deb
cat server/deploy/build/out/usb-control_V3.0.1_arm64.deb.sha256
dpkg-deb --info server/deploy/build/out/usb-control_V3.0.1_arm64.deb
dpkg-deb --contents server/deploy/build/out/usb-control_V3.0.1_arm64.deb
```

## 6. 服务端安装与运行

### 6.1 安装 `.deb`

服务端安装包目录：

| 位置 | 目录 |
|---|---|
| 服务端 VM 构建输出 | `server/deploy/build/out/` |
| RK3568 临时安装目录 | `/tmp/usb-control-package/` |

RK3568 的 `/tmp/usb-control-package/` 需要包含：

```text
usb-control_V3.0.1_arm64.deb
usb-control_V3.0.1_arm64.deb.sha256
```

在 RK3568 上执行：

```bash
cd /tmp/usb-control-package
EXPECTED_SHA256="$(awk '{print $1}' usb-control_V3.0.1_arm64.deb.sha256)"
ACTUAL_SHA256="$(sha256sum usb-control_V3.0.1_arm64.deb | awk '{print $1}')"
test "$EXPECTED_SHA256" = "$ACTUAL_SHA256"
sudo apt-get install -y ./usb-control_V3.0.1_arm64.deb
```

安装脚本检查：

- CPU 架构必须为 arm64；
- 操作系统必须为 Ubuntu 22.04；
- Linux 内核必须为 4.19.x；
- `clamd` 和 `clamdscan` 必须存在；
- `clamav-daemon.service` 必须能正常启动；
- clamd socket 和三组病毒库必须存在。

安装成功后自动执行数据库迁移并启动：

```text
clamav-daemon.service
usb-control.service
```

### 6.2 安装目录

| 路径 | 内容 |
|---|---|
| `/opt/usb-control/bin/` | 主服务、短周期 updater 和数据库迁移程序 |
| `/opt/usb-control/db/` | 数据库迁移和初始化 SQL |
| `/opt/usb-control/install-meta/` | 版本及组件信息 |
| `/etc/usb-control/usb-control.toml` | 服务配置 |
| `/etc/usb-control/tls/` | TLS 证书和私钥 |
| `/etc/usb-control/keys/` | 许可校验和策略密钥 |
| `/var/lib/usb-control/device.db` | SQLite 数据库 |
| `/var/lib/usb-control/upgrade/` | 在线升级任务、历史和结果 |
| `/var/log/usb-control/` | 服务日志和扫描日志 |

### 6.3 服务配置

默认配置：

```toml
listen_addr = "0.0.0.0:9600"
database_path = "/var/lib/usb-control/device.db"
tls_cert_path = "/etc/usb-control/tls/server.crt"
tls_key_path = "/etc/usb-control/tls/server.key"

[gadget]
name = "rockchip"
config = "b.1"
udc = "fcc00000.dwc3"
keep_adb = false
```

修改配置后执行：

```bash
sudo systemctl restart usb-control
systemctl status usb-control --no-pager
```

### 6.4 安装后验证

```bash
systemctl status clamav-daemon --no-pager
systemctl status usb-control --no-pager
ss -ltnp | grep ':9600'
journalctl -u usb-control -n 200 --no-pager
```

### 6.5 升级与卸载

直接安装 V3.0.1：

```bash
sudo apt-get install -y ./usb-control_V3.0.1_arm64.deb
```

直接升级到 V3.0.2：

```bash
sudo apt-get install -y ./usb-control_V3.0.2_arm64.deb
```

直接安装和直接升级均由 DEB 完成数据库迁移、服务启动和健康检查。健康检查使用 `/opt/usb-control/install-meta/release.json` 中的实际安装版本；检查成功后，安装程序将该版本写入 SQLite `system_config.system_version`。升级保留 `/etc/usb-control` 中的现场配置与密钥、`/var/lib/usb-control` 中的数据库和授权状态，以及 `/var/log/usb-control` 中的日志。未过期授权不需要重新办理。

在线升级时，在 Windows Client 的“系统升级”界面选择 `usb-control_V3.0.2_arm64.bin`。Client 根据文件内容计算 SHA-256 并将升级包、目标版本和摘要上传到 RK3568。服务端完整接收、校验并持久化升级任务后先返回受理结果，Client 应显示“升级包已接收，Server 开始升级”。随后主服务停止，Client 按正常连接机制显示断开；新服务启动后重新连接，在操作日志中查询最终成功或失败结果。

在线升级由主服务受理任务并启动短周期 `usb-control-updater`。updater 在停服前读取 SQLite 中的业务版本和 Schema，复验签名包、内部 DEB 及升级源约束，调用系统包管理器安装，再执行数据库迁移、服务启动和健康检查。健康检查成功后，updater 使用 compare-and-set 将 `system_config.system_version` 从任务源版本提交为目标版本；只有数据库提交成功才记录升级成功结果。updater 完成后退出，不作为常驻服务运行。

Windows Client 查询和显示的是 SQLite `system_config.system_version` 中已经提交的业务版本。服务就绪和健康检查使用 `release.json` 中的实际运行版本，两者职责不同。系统不使用 `active-release.json` 或 `install-meta/VERSION` 保存当前版本。

直接安装或在线升级失败时，不恢复旧版本。检查 `journalctl -u usb-control -u usb-control-updater` 和 `/var/lib/usb-control/upgrade/` 中的任务结果，处理故障后人工重新安装可信的正式 DEB。直接 DEB 安装失败时无需通过 Client 处理。

普通卸载：

```bash
sudo dpkg -r usb-control
```

普通卸载保留配置、授权、数据库和日志，重新安装后继续使用。完全清除：

```bash
sudo dpkg --purge usb-control
```

完全清除会删除 `/etc/usb-control`、`/var/lib/usb-control` 和 `/var/log/usb-control`，仅在确定不再保留现场状态时执行。

## 7. Windows 客户端开发、编译与测试

客户端代码位于仓库的 `client/` 目录。代码编辑、开发运行、静态检查、自动化测试和 NSIS 安装包构建均在 Windows 客户端开发测试机完成。本章命令从仓库根目录执行。

### 7.1 开发环境

准备：

- Windows 10/11 64 位；
- PowerShell 5.1 或 PowerShell 7；
- Git；
- Node.js 20 LTS；
- Node.js 自带的 npm；
- npm 依赖下载能力或完整的离线 npm 缓存。

检查：

```powershell
node --version
npm --version
git --version
```

### 7.2 安装依赖和启动开发环境

```powershell
npm --prefix client ci
npm --prefix client run dev
```

项目使用 `package-lock.json` 固定依赖。`node_modules`、`out` 和 `dist` 是本地生成目录。

### 7.3 客户端检查与测试

```powershell
npm --prefix client run typecheck
npm --prefix client run lint
npm --prefix client run test
npm --prefix client run build
```

端到端测试：

```powershell
npm --prefix client run test:e2e
```

Playwright 首次执行可能需要安装对应浏览器运行环境。

### 7.4 生成 Windows 安装包

从仓库根目录执行：

```powershell
powershell -ExecutionPolicy Bypass -File client\deploy\build-nsis.ps1
```

脚本执行以下操作：

1. 读取并检查 `deploy/assets/tls/server.crt.sha256`；
2. 设置本次构建使用的 `USB_CONTROL_CERT_FINGERPRINT`；
3. 执行 `npm ci`；
4. 执行 `npm run build`；
5. 执行 `npm run dist`；
6. 生成 NSIS 安装程序和 SHA-256 文件。

输出：

```text
client/dist/USB-Control-Setup-V3.0.1.exe
client/dist/USB-Control-Setup-V3.0.1.exe.sha256
```

检查：

```powershell
Get-FileHash -Algorithm SHA256 client\dist\USB-Control-Setup-V3.0.1.exe
Get-Content client\dist\USB-Control-Setup-V3.0.1.exe.sha256
```

### 7.5 安装和连接装置

在 Windows 客户端开发测试机运行：

```text
USB-Control-Setup-V3.0.1.exe
```

安装完成后启动客户端，填写 RK3568 的地址和服务端端口。服务端默认端口为 `9600`。

检查网络连通性：

```powershell
$DeviceAddress = Read-Host 'RK3568 address'
Test-NetConnection $DeviceAddress -Port 9600
```

客户端使用 TLS 证书指纹固定。客户端构建使用的 `deploy/assets/tls/server.crt.sha256` 必须与 RK3568 安装包中的服务端证书一致。

## 8. 设备连接与系统验证

### 8.1 连接顺序

1. 启动 RK3568，确认 `clamav-daemon` 和 `usb-control` 正常运行；
2. 在 Windows 客户端开发测试机启动客户端并连接 RK3568；
3. 将待管控 U 盘接入 RK3568 USB Host 口；
4. 使用支持数据传输的 USB 线连接 RK3568 OTG 口和 Windows 受控机；
5. 在客户端完成设备识别、授权和策略配置；
6. 在 Windows 受控机检查装置映射出的 USB 存储或 HID 设备；
7. 验证允许文件可以访问、禁止文件被阻断、病毒扫描失败时不执行映射。

### 8.2 RK3568 验证命令

```bash
systemctl is-active clamav-daemon
systemctl is-active usb-control
lsusb
lsblk -o NAME,TYPE,FSTYPE,SIZE,MOUNTPOINTS,MODEL,SERIAL
ls /sys/class/udc
lsmod | grep -E '^(exfat|nbd|usb_f_hid)\b'
cat /sys/module/nbd/parameters/max_part
ss -ltnp | grep ':9600'
```

### 8.3 功能验证

- 客户端能够建立 TCP/TLS 连接；
- U 盘插入 Host 口后能被 RK3568 识别；
- ClamAV 能完成扫描；
- 授权后 Windows 受控机能够枚举虚拟 USB 设备；
- 文件访问结果符合当前策略；
- 键盘、鼠标 HID 管控符合当前策略；
- 拔出 U 盘、断开 OTG 或重启服务后资源能够正确清理；
- RK3568 重启后内核模块和两个 systemd 服务能够自动启动。

## 9. 故障排查

### 9.1 服务无法启动

```bash
systemctl status usb-control --no-pager
journalctl -u usb-control -b --no-pager
```

前台调试：

```bash
sudo systemctl stop usb-control
sudo env RUST_LOG=debug /opt/usb-control/bin/usb-control \
  --config /etc/usb-control/usb-control.toml
```

调试结束后执行：

```bash
sudo systemctl start usb-control
```

### 9.2 Windows 客户端无法连接

RK3568 检查：

```bash
ss -ltnp | grep ':9600'
ip address
journalctl -u usb-control -n 200 --no-pager
```

Windows 检查：

```powershell
$DeviceAddress = Read-Host 'RK3568 address'
Test-NetConnection $DeviceAddress -Port 9600
```

端口可达但 TLS 连接失败时，检查客户端构建使用的证书指纹、RK3568 当前服务证书和两端系统时间。

### 9.3 U 盘无法识别

```bash
lsusb
lsblk -o NAME,TYPE,FSTYPE,SIZE,MOUNTPOINTS,MODEL,SERIAL
udevadm monitor --kernel --udev
dmesg -w
```

检查 U 盘是否连接到 Host 口、供电是否稳定、文件系统是否受内核支持。

### 9.4 Windows 受控机看不到映射设备

```bash
ls /sys/class/udc
find /sys/kernel/config/usb_gadget -maxdepth 3 -type f 2>/dev/null
lsmod | grep -E '^(nbd|usb_f_hid)\b'
ls -l /dev/nbd* 2>/dev/null
cat /sys/module/nbd/parameters/max_part
dmesg | tail -n 200
```

检查 OTG 端口、USB 数据线、配置中的 UDC 名称和 Windows 设备管理器。

### 9.5 ClamAV 扫描失败

```bash
systemctl status clamav-daemon --no-pager
journalctl -u clamav-daemon -b --no-pager
test -S /run/clamav/clamd.ctl
clamdscan --version
ls -l /var/lib/clamav
```

扫描服务、socket 或病毒库异常时，系统拒绝映射 U 盘。

### 9.6 NBD 异常

```bash
cat /sys/module/nbd/parameters/max_part
ls -l /dev/nbd* 2>/dev/null
ls /sys/block/nbd* 2>/dev/null
dmesg | grep -E 'nbd[0-9]+' | tail -n 100
```

出现残留 NBD 会话时，先停止 `usb-control` 并保存日志，再检查占用进程和设备状态。

## 10. 发布检查

### RK3568

- [ ] 使用 `ubuntu22.img` 和对应的 RK3568 烧写手册；
- [ ] 系统为 Ubuntu 22.04，架构为 ARM64，内核为 4.19.232；
- [ ] `exfat.ko`、`nbd.ko`、`usb_f_hid.ko` 正常加载；
- [ ] `nbd.max_part=0`，重启后保持生效；
- [ ] configfs、UDC、USB Host 和 USB OTG 可用；
- [ ] ClamAV 服务、socket 和病毒库可用；
- [ ] 服务端 `.deb` 安装成功；
- [ ] `usb-control.service` 正常运行且 9600 端口正常监听。

### 服务端 VM

- [ ] Rust 1.96 和 ARM64 target 已安装；
- [ ] Buildroot 编译器和 sysroot 可用；
- [ ] ARM64 `libudev` 可被 pkg-config 找到；
- [ ] fmt、clippy 和构建前代码检查通过；
- [ ] V3.0.1、V3.0.2 `.deb` 及 V3.0.2 `.bin` 已生成并校验；
- [ ] DEB 中不包含升级签名私钥、测试脚本或测试数据；
- [ ] `.bin` 内部 DEB 与同版本独立 DEB 字节一致；
- [ ] 发布版本、最低当前版本和 `schema-from` 参数已经发布负责人复核。

### Windows 客户端开发测试机

- [ ] Node.js 20 LTS 和 npm 可用；
- [ ] `npm ci`、typecheck、lint、测试和 build 通过；
- [ ] NSIS 安装包和 `.sha256` 已生成；
- [ ] 客户端证书指纹与服务端证书一致；
- [ ] 在线升级受理后显示开始升级提示，断连后可重新连接并查询最终业务日志；
- [ ] 客户端连接、U 盘识别、病毒扫描、策略控制和 OTG 映射验证通过。
