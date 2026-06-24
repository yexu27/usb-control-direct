# Server 端日志等级使用规范

## 日志框架

本项目使用 `tracing` + `tracing-subscriber`。通过 `RUST_LOG` 环境变量控制日志级别。

## 日志等级定义

| 等级 | 用途 | 生产环境默认可见 | 示例场景 |
|------|------|:---:|----------|
| `ERROR` | 需要人工介入的系统故障 | ✅ | 数据库损坏、clamd 不可用、TLS 握手失败、端口绑定失败 |
| `WARN` | 可自动恢复的异常、安全事件 | ✅ | 锁中毒恢复、心跳超时、扫描发现病毒、NBD 设备池耗尽、连续 CRC 失败 |
| `INFO` | 关键业务节点（生命周期事件） | ✅ | 服务启动、设备插入/拔出、登录成功/失败、策略导入完成、映射成功、白名单变更 |
| `DEBUG` | 诊断信息（排障用） | ❌ | 策略判定原因、文件树构建进度、扫描文件数、中间状态变化、参数详情 |
| `TRACE` | 逐条数据处理（极致细节） | ❌ | 每个 HID report 内容、每个文件的策略命中结果、逐条 udev 事件 |

## 使用规则

### 1. 所有模块必须添加 `use tracing::{debug, error, info, trace, warn};`

### 2. INFO 级别 —— 记录关键业务节点

每个 handler / 服务方法的入口和关键分支必须有一条 `info!` 日志：

```rust
info!(user = %username, action = "create_user", target = %target_user, "用户创建成功");
info!(serial = %sn, "U 盘映射成功");
info!(user = %username, source_ip = %ip, "用户登录成功");
```

### 3. WARN 级别 —— 记录可恢复异常和安全事件

```rust
warn!(user = %username, reason = %e, "用户登录失败");
warn!(serial = %sn, "扫描发现病毒: {} 个文件", count);
warn!(reason = %e, "心跳超时，断开连接");
```

### 4. ERROR 级别 —— 记录不可恢复的系统故障

```rust
error!(reason = %e, "clamd 不可用，扫描服务暂停");
error!(reason = %e, "TLS 握手失败");
error!(reason = %e, "数据库连接池初始化失败");
```

### 5. DEBUG 级别 —— 记录诊断信息

```rust
debug!(file = %path, policy_level = "L3", action = "deny", "文件被拒绝访问");
debug!(dev = %dev_path, fs_type = %fs_type, "检测到文件系统类型");
debug!(seq_id = seq_id, msg_type = format_args!("0x{:04X}", msg_type), "收到业务请求");
```

### 6. TRACE 级别 —— 记录逐条数据处理

```rust
trace!(report = ?bytes, "写鼠标 HID report");
trace!(key = %key_code, "按键事件");
```

## 结构化字段约定

- 用户名: `user = %username`
- 设备序列号: `serial = %sn`
- 来源 IP: `source_ip = %ip`
- 操作类型: `action = "create_user"`
- 错误原因: `reason = %e` 或 `error = %e`
- 消息类型: `msg_type = format_args!("0x{:04X}", msg_type)`
- 序列号: `seq_id = seq_id`
- 文件路径: `file = %path`
- 设备路径: `dev = %dev_path`

## 禁止事项

- ❌ 禁止在日志中记录密码、token、密钥、机器码等敏感信息
- ❌ 禁止所有事件都用 `info!`（无法过滤）
- ❌ 禁止模块零日志（运行时黑盒）
- ❌ 禁止静默吞掉错误（`let _ = ...` 不记录日志）

## 生产环境配置

默认 `RUST_LOG=info`，按需调整：

```bash
# 生产：只看关键业务事件
RUST_LOG=info

# 排障：开启协议层调试
RUST_LOG=info,protocol_gateway=debug

# 深度调试：开启全模块 trace
RUST_LOG=info,usb_control=trace
```
