# Git Commit Message Rules

所有 commit 必须遵循 Conventional Commits 规范，subject 和 body 使用中文描述。

## 1. Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

示例：

```
feat(login): 添加 LDAP 认证支持

- 支持 LDAP 用户登录
- 添加 LDAP 配置解析器
- 添加连接重试机制
- 更新登录错误处理

Refs: #102
```

## 2. Commit Type

type 使用英文 Conventional Commits 标准关键字：

| Type | Description |
|------|-------------|
| feat | 新功能 |
| fix | Bug 修复 |
| docs | 文档修改 |
| style | 代码格式调整（不影响逻辑） |
| refactor | 重构（非新增功能、非修复） |
| perf | 性能优化 |
| test | 测试相关 |
| build | 构建系统修改 |
| ci | CI/CD 修改 |
| chore | 杂项修改 |
| revert | 回滚提交 |

## 3. Scope 规范

scope 使用英文，表示修改模块：

```
feat(auth):
fix(mysql):
refactor(network):
docs(readme):
```

禁止：

```
feat():
fix():
update:
```

## 4. Subject 规范

subject 使用中文，必须：

- 小写开头（中文无大小写，正常书写即可）
- 使用动词开头
- 不超过 72 字符
- 不加句号
- 明确描述"做了什么"

正确：

```
fix(mysql): 修复连接泄漏问题
feat(usb): 添加热插拔监控支持
```

## 5. Body 规范（重要）

body 使用中文，必须详细描述：

- 修改原因
- 修改内容
- 影响范围
- 兼容性
- 风险点

body 使用 markdown 列表：

```
- 添加 mysql 重连机制
- 优化 socket 缓冲区处理
- 修复 epoll 回调中的内存泄漏
- 改进 RPC 超时重试逻辑
```

禁止：

```
some fixes
update implementation
修改
代码优化
```

## 6. Footer 规范

如果有关联 issue：

```
Refs: #123
Closes: #456
```

Breaking Change：

```
BREAKING CHANGE: 移除旧 RPC 协议
```

## 7. 提交要求

1. 始终生成多行 commit message
2. 始终包含 body 段落
3. 禁止单行 commit
4. 禁止模糊描述
5. 必须解释 WHY 和 WHAT
6. 必须概括关键修改模块
7. 严格遵循 Conventional Commits 格式
8. subject 和 body 使用中文，type 和 scope 使用英文
9. 优先详细的技术描述
10. 输出必须可直接用于 git commit

## 8. 完整示例

### 新功能

```
feat(usb): 添加热插拔监控支持

- 实现 libudev 热插拔事件监听
- 添加 USB 设备插入检测
- 添加设备移除回调
- 优化设备枚举逻辑
```

### Bug 修复

```
fix(socket): 修复 epoll fd 内存泄漏

- 释放未使用的 epoll 事件
- 修复重复 fd 注册问题
- 优化 socket 关闭处理
```

### 重构

```
refactor(rule): 重新设计规则分发架构

- 分离规则解析器与分发器
- 优化模块回调注册
- 降低 hb 与 defense 之间的耦合
```

## 9. 禁止的 Commit Message

禁止：

```
update
fix bug
fix
修改
代码优化
test
tmp
wip
final
done
提交代码
```

## 10. AI Agent 输出示例

必须输出：

```
fix(audit): 修复审计监控中的文件描述符泄漏

- 修复错误分支中未关闭的 fd
- 优化监控清理逻辑
- 减少审计事件处理开销
- 改进异常退出处理

Refs: #235
```

禁止输出：

```
fix bug
```
