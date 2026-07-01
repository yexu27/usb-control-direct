# S04 虚拟 exFAT 端到端验收矩阵

本文件是 RK3568 + 受控 Windows 验收清单，不替代 Rust 自动化测试。

## 环境

- VM：`ssh -p 2222 -i ~/.ssh/WinPC-Personal root@172.16.0.219`
- RK3568：`ssh -i ~/.ssh/WinPC-Test root@172.16.3.95`
- 受控 Windows：`ssh -i ~/.ssh/WinPC-Test -o UserKnownHostsFile=/tmp/usb_control_win_371_known_hosts -o StrictHostKeyChecking=accept-new -p 22 Administrator@172.16.3.71`
- Windows 盘符：以 `USB_CTRL` 对应盘符为准，示例使用 `G:`

## 必须记录的证据

- VM `cargo test -p file-access -- --nocapture` 输出。
- VM 交叉编译输出。
- RK 服务启动日志、LUN file、`lsblk`。
- Windows `Get-Volume` 输出。
- Windows 操作命令和 RK 真实 U 盘 `find`/`cat` 对比。

## 验收项

| 编号 | 场景 | Windows 检查 | RK 真实 U 盘检查 | 通过标准 |
| --- | --- | --- | --- | --- |
| V01 | 初始深度目录 `/a/b/c.txt` | `Get-ChildItem G:\ -Recurse` | `find /mnt/usb_raw/sdc2 -maxdepth 5` | Windows 可见且内容可读，真实文件不变 |
| V02 | 不同目录同名文件 | `Get-ChildItem G:\a,G:\b` | `find /mnt/usb_raw/sdc2 -name readme.txt` | 两个文件均可见且内容区分 |
| V03 | 新建 0 字节根文件 | `New-Item -ItemType File G:\empty_$stamp.txt` | `find ... -name empty_$stamp.txt -printf "%s"` | RK size=0 |
| V04 | 新建 0 字节嵌套文件 | `New-Item -ItemType File G:\dir\empty_$stamp.txt` | `find ... -name empty_$stamp.txt` | RK 嵌套文件存在，size=0 |
| V05 | 新建空根目录 | `New-Item -ItemType Directory G:\empty_dir_$stamp` | `test -d /mnt/usb_raw/.../empty_dir_$stamp` | 真实目录存在 |
| V06 | 新建嵌套空目录 | `New-Item -ItemType Directory G:\nested_$stamp\a\b -Force` | `test -d /mnt/usb_raw/.../nested_$stamp/a/b` | 完整目录链存在 |
| V07 | 新建根文件并写内容 | `Set-Content G:\data_$stamp.txt` | `cat /mnt/usb_raw/.../data_$stamp.txt` | 内容一致 |
| V08 | 新建嵌套文件并写内容 | `Set-Content G:\nested_$stamp\a\b\data_$stamp.txt` | `cat /mnt/usb_raw/.../nested_$stamp/a/b/data_$stamp.txt` | 内容一致 |
| V09 | 修改已有文件 | `Set-Content` 覆盖内容 | `cat` | 内容更新 |
| V10 | 截断已有文件 | 写入更短内容 | `stat -c %s` | size 变小 |
| V11 | 扩展已有文件 | 写入更长内容 | `stat -c %s; cat` | size 和内容正确 |
| V12 | 删除文件 | `Remove-Item G:\file` | `test ! -e` | 真实文件删除 |
| V13 | 删除目录 | `Remove-Item G:\dir -Recurse` | `test ! -e` | 真实目录删除 |
| V14 | 重命名文件 | `Rename-Item` | `test -e new; test ! -e old` | 新名存在，旧名不存在 |
| V15 | 重命名目录 | `Rename-Item` | `test -d new; test ! -e old` | 新目录存在，旧目录不存在 |
| V16 | 病毒文件 | 文件可见、size=0、打开失败 | 真实文件名和内容不变 | 符合 PRD |
| V17 | 黑名单/可执行读取 | 打开时报 I/O error | 真实文件不变 | 普通文件不受影响 |
| V18 | 只读权限 | Windows 写入失败 | `find` 无新增/修改 | 真实 U 盘无变更 |
| V19 | Windows 卷健康 | `Get-Volume` | RK 无 nbdp1 循环 | 不出现 `Full Repair Needed` |
| V20 | 停服务/拔出 | 盘符移除 | LUN/NBD/mount 清理 | 设备级日志完整 |

## 重新映射持久性

完成 V03-V15 后必须停止服务、清理 LUN/NBD、重新启动服务，再次对比 Windows 和 RK：

```bash
ssh -i ~/.ssh/WinPC-Test root@172.16.3.95 'pid=$(pgrep -f "^/opt/usb-control/bin/usb-control --config /etc/usb-control/usb-control.toml" | head -n 1); test -n "$pid" && kill "$pid" || true'
ssh -i ~/.ssh/WinPC-Test root@172.16.3.95 'sleep 2; echo "" > /sys/kernel/config/usb_gadget/rockchip/functions/mass_storage.usb0/lun.0/file; nbd-client -d /dev/nbd3 2>/dev/null || true'
ssh -i ~/.ssh/WinPC-Test root@172.16.3.95 'nohup sh -c "RUST_LOG=trace /opt/usb-control/bin/usb-control --config /etc/usb-control/usb-control.toml" >/tmp/usb-control-s04-remap.log 2>&1 & echo $!'
```

重新映射后，Windows 可见状态必须与 RK 真实 U 盘一致。
