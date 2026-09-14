# CLAUDE.local.md - 个人开发环境配置

## Git Remote


## 多机工作流

```
Mac (代码编写) ──→ Ubuntu VM (server 交叉编译) ──→ RK3568 板子 (装置端服务程序)
                                                          ├── 网络 ←── Windows 172.16.0.219 (管理端 client 编译与运行)
                                                          └── USB ──→ Windows 172.16.3.71 (受控主机)
```
- Mac 仅做代码编写，不直接连接其他机器
- server 编译产物由 Mac 推送到 Ubuntu VM 交叉编译，再推到 RK3568 运行
- client 代码在 Windows 172.16.0.219 本地编译运行，通过网络连接 RK3568
- 受控主机 172.16.3.71 被 RK3568 通过 USB 管控，用于验证映射内容是否生效

### Windows 机器 (172.16.0.219) — 管理端编译与运行环境

- 用途: 管理端客户端（Electron）的编译与运行环境，通过网络连接 RK3568 装置端服务程序；同时也是 Ubuntu VM 宿主机
- 客户端代码工作目录: `F:\aiCode\github\usb-control-direct\client`
- User: Andisec
- SSH Key: `~/.ssh/WinPC-Personal`（已部署）

### Ubuntu 18.04 VM（交叉编译环境）

- 宿主机: Windows 172.16.0.219（VM 使用 NAT 网络）
- VM IP: 192.168.201.128
- SSH: `ssh -p 2222 -i ~/.ssh/WinPC-Personal root@172.16.0.219`
- User: root / topeet
- SSH Key: `~/.ssh/WinPC-Personal`（已部署）
- 工作目录: `/root/work/code/usb-control-direct`
- 连接较慢，超时建议设 30s 以上

**已安装的交叉编译工具链：**

| 组件 | 版本/目标 |
|------|-----------|
| rustc | 1.96.0 |
| Rust 目标 | x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu |
| 交叉工具链 | Buildroot aarch64-buildroot-linux-gnu-gcc |
| Sysroot | /home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/aarch64-buildroot-linux-gnu/sysroot |

**交叉编译注意事项：**

- 因为引入了 libudev 依赖，不能使用系统自带的交叉编译工具链，必须使用 Buildroot 提供的 sysroot 和工具链。
- 编译前需设置以下环境变量：

  ```bash
  export SYSROOT=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/aarch64-buildroot-linux-gnu/sysroot
  export TOOLCHAIN=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/bin
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=$TOOLCHAIN/aarch64-buildroot-linux-gnu-gcc
  export PKG_CONFIG_SYSROOT_DIR=$SYSROOT
  export PKG_CONFIG_PATH=$SYSROOT/usr/lib/pkgconfig
  export RUSTFLAGS="-C link-args=--sysroot=$SYSROOT"
  ```

- 完整编译命令：

  ```bash
  source ~/.cargo/env && \
  SYSROOT=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/aarch64-buildroot-linux-gnu/sysroot \
  TOOLCHAIN=/home/topeet/Linux/rk356x_linux/buildroot/output/rockchip_rk3568/host/bin \
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=$TOOLCHAIN/aarch64-buildroot-linux-gnu-gcc \
  PKG_CONFIG_SYSROOT_DIR=$SYSROOT \
  PKG_CONFIG_PATH=$SYSROOT/usr/lib/pkgconfig \
  RUSTFLAGS="-C link-args=--sysroot=$SYSROOT" \
  cargo build --release --target aarch64-unknown-linux-gnu
  ```

### RK3568 开发板 (172.16.3.95) — 运行验证环境

- 用途: 验证 Ubuntu VM 交叉编译产出的 aarch64 程序
- 系统: Ubuntu 22.04.4 LTS (aarch64)
- 内核: Linux 4.19.232
- SSH: `ssh -i ~/.ssh/WinPC-Test root@172.16.3.95`
- User: root / topeet
- SSH Key: `~/.ssh/WinPC-Test`（已部署）

### Windows 受控主机 (172.16.3.71) — 受控主机验证环境

- 用途: 被 RK3568 通过 USB 管控 USB 接口的机器，验证 USB 外设映射内容是否生效
- 系统: Windows (DESKTOP-3FC8ESF)
- SSH: `ssh -i ~/.ssh/WinPC-Test Administrator@172.16.3.71`
- User: Administrator / 1
- SSH Key: `~/.ssh/WinPC-Test`（已部署）

## 临时文件目录

Claude 工作过程中产生的临时文件（编译产物、中间文件等）统一放在 `/tmp/usb-control-direct/`：

```bash
mkdir -p /tmp/usb-control-direct
```

- 不要散放在 `/tmp/` 根目录下

## 注意事项

- Windows 输出 GBK 编码，SSH 命令末尾加 `| iconv -f GBK -t UTF-8`
- bat 文件需 CRLF 换行（Write 工具生成 LF，上传前用 `unix2dos` 转换）
