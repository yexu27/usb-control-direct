# Rust 编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 Rust 代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

参考来源取向：Rust 官方 Style Guide、Rust API Guidelines、Clippy 文档、rustfmt 默认风格，以及常见 Rust 工程实践。

## 0. 规范说明

### 0.1 适用范围

本规范适用于使用 Rust 编写的业务代码、基础库代码、命令行工具、服务端代码、嵌入式代码和测试代码。

如果项目已有更具体的编码规范、`rustfmt.toml`、Clippy 配置、MSRV 约束或安全规范，必须优先使用项目已有规范。

### 0.2 总体原则

### 清晰第一。

说明：Rust 的类型系统和所有权机制可以表达大量约束，代码必须利用这些机制提升可读性和可靠性。

### 简洁为美。

说明：不写不必要的泛型、宏和 trait 抽象。重复逻辑必须提取为函数、方法、trait 或模块接口。

### 风格一致。

说明：新增代码必须符合项目已有模块结构、命名风格、错误处理方式和工具配置。

### 安全可靠。

说明：优先使用安全 Rust。`unsafe` 只能用于必要边界，并且必须有清晰注释和局部封装。

## 1. 包与模块

### 模块必须表达清晰边界

- crate、module、文件名必须使用 `snake_case`。

- 模块职责必须单一，不要把无关功能放进同一个模块。

- 对外接口必须通过 `pub` 明确暴露。

- 内部实现不要使用 `pub`，只在必要时使用 `pub(crate)`。

错误示例：

```rust
pub fn parse_internal_cache_key(key: &str) -> String {
    key.to_string()
}
```

正确示例：

```rust
pub(crate) fn parse_internal_cache_key(key: &str) -> String {
    key.to_string()
}
```

- `mod.rs` 或扁平模块布局必须以项目既有风格为准。

- 不要通过深层模块路径暴露不稳定实现细节。

## 2. use 与依赖

### use 必须清晰表达依赖

- `use` 必须放在模块顶部。

- 标准库、第三方 crate、项目内部模块的分组必须清晰。

- 禁止保留无用 `use`。

- 不要使用过宽的 glob import，除非是测试模块或 prelude 约定。

错误示例：

```rust
use crate::user::*;
```

正确示例：

```rust
use crate::user::UserProfile;
```

- 新增依赖必须确认标准库无法合理完成，并且依赖维护活跃。

## 3. 代码风格

### 代码风格必须由 rustfmt 固化

- Rust 代码必须使用 rustfmt 格式化。

- 缩进、换行、空格、链式调用排版必须以 rustfmt 输出为准。

- 行宽必须以项目 `rustfmt.toml` 为准；项目没有配置时，使用 rustfmt 默认设置。

- `use` 必须分组清晰，禁止保留无用 `use`。

- 模块名、函数名、变量名使用 `snake_case`。

- 类型名、trait 名、enum 名、struct 名使用 `PascalCase`。

- 常量和 static 使用 `SCREAMING_SNAKE_CASE`。

- 花括号和控制块排版必须以 rustfmt 输出为准。

- 链式调用过长时，必须换行以保持可读性。

示例：

```rust
let active_users = users
    .iter()
    .filter(|user| user.is_active)
    .collect::<Vec<_>>();
```

- 文档注释使用 `///`，模块注释使用 `//!`。

## 4. 命名

### 命名必须符合 Rust 惯例

- 函数、变量、模块使用 `snake_case`。

- 类型、trait、enum、struct 使用 `PascalCase`。

- 常量和静态变量使用 `SCREAMING_SNAKE_CASE`。

示例：

```rust
const MAX_RETRY_COUNT: usize = 3;

struct UserProfile {
    user_id: String,
}

fn load_user_profile(user_id: &str) -> Result<UserProfile, UserError> {
    todo!()
}
```

- 判断类函数必须表达判断语义。

示例：

```rust
fn is_user_active(user: &UserProfile) -> bool {
    user.active
}
```

- 不要使用含义模糊的名称，例如 `data`、`info`、`tmp`、`obj`，除非上下文非常明确。

### trait 和接口命名必须表达能力

- trait 名称必须表达调用方可依赖的能力。

错误示例：

```rust
trait UserServiceTrait {
    fn load_user_profile(&self, user_id: &str) -> Result<UserProfile, UserError>;
}
```

正确示例：

```rust
trait UserProfileLoader {
    fn load_user_profile(&self, user_id: &str) -> Result<UserProfile, UserError>;
}
```

- 不要为了单个实现提前定义 trait。

- 常量必须使用 `SCREAMING_SNAKE_CASE`。

- 可变全局状态必须避免使用。必须使用时，必须通过安全封装控制访问。

## 5. 函数

### 一个函数只完成一个功能

- 函数名必须准确表达函数功能。

- 函数一般不超过 50 行。超过时必须检查是否可以拆分。

- 参数过多时，必须使用结构体或 builder 组织参数。

错误示例：

```rust
fn create_user(name: String, age: u32, email: String, phone: String, level: u8) {}
```

正确示例：

```rust
struct CreateUserRequest {
    name: String,
    age: u32,
    email: String,
    phone: String,
    level: u8,
}

fn create_user(request: CreateUserRequest) {}
```

- 不要为了绕过借用检查而复制大量数据。

- 输入只读时优先使用引用。

错误示例：

```rust
fn validate_user(user: UserProfile) -> bool {
    user.active
}
```

正确示例：

```rust
fn validate_user(user: &UserProfile) -> bool {
    user.active
}
```

## 6. 接口

### 接口必须表达稳定契约

- 被多个模块、多个调用点或多个业务流程复用的能力，必须提取为公共接口。

- 公共接口必须有中文文档注释。

- 接口文档必须说明功能、参数和返回。

示例：

```rust
/// 根据用户 ID 加载用户资料。
///
/// 参数:
/// - `user_id`: 用户唯一 ID，不能为空。
///
/// 返回:
/// - 成功时返回用户资料。
pub fn load_user_profile(user_id: &str) -> Result<UserProfile, UserError> {
    todo!()
}
```

- 修改公共接口时，必须同步检查调用方、测试和文档。

- 不要为了单个调用点创建过早抽象。

- 公共 trait 必须说明实现者需要满足的行为约束。

示例：

```rust
/// 用户资料加载能力。
///
/// 参数:
/// - `user_id`: 用户唯一 ID，不能为空。
///
/// 返回:
/// - 成功时返回用户资料；失败时返回加载错误。
pub trait UserProfileLoader {
    fn load_user_profile(&self, user_id: &str) -> Result<UserProfile, UserError>;
}
```

## 7. 类型与数据模型

### 类型必须表达业务约束

- 使用 struct 表达稳定数据模型。

- 使用 enum 表达有限状态。

错误示例：

```rust
let status = "active";
```

正确示例：

```rust
enum UserStatus {
    Active,
    Inactive,
}
```

- 不要长期使用 `HashMap<String, String>` 代替明确数据结构。

- 需要可选值时使用 `Option<T>`。

- 可能失败的操作使用 `Result<T, E>`。

- 错误类型必须表达业务含义。

## 8. 所有权与生命周期

### 所有权必须清晰

- 不要无意义地调用 `clone()`。

错误示例：

```rust
let name = user.name.clone();
println!("{}", user.name);
```

正确示例：

```rust
let name = &user.name;
println!("{}", name);
```

- 函数不需要取得所有权时，使用引用。

- 返回借用值时，生命周期必须清晰。

- 不要用生命周期标注掩盖设计问题。

## 9. 错误处理

### 错误必须被处理或向上传递

- 不要在业务代码中随意使用 `unwrap()` 和 `expect()`。

错误示例：

```rust
let config = load_config(path).unwrap();
```

正确示例：

```rust
let config = load_config(path)?;
```

- 只有在测试或已证明不可能失败的场景，才允许使用 `unwrap()`。

- 错误向上传递时必须保留上下文。

- 不要吞掉错误。

## 10. unsafe

### unsafe 必须局部、必要、可审查

- 禁止为了绕过借用检查随意使用 `unsafe`。

- `unsafe` 必须限制在最小作用域。

- 每个 `unsafe` 块必须写中文注释说明安全前提。

示例：

```rust
// 安全性:
// ptr 来自有效切片，且调用前已经检查 index 小于切片长度。
let value = unsafe { *ptr.add(index) };
```

- 封装 `unsafe` 的公共接口必须保持安全 Rust 语义。

## 11. 并发

### 共享状态必须有明确同步机制

- 多线程共享可变状态必须使用 `Mutex`、`RwLock`、原子类型或消息传递。

- 不要把锁暴露给不相关模块。

- 不要在持锁状态下执行不可控耗时操作。

- 后台线程和异步任务必须有明确关闭流程。

## 12. 质量保证

### 代码必须便于检查、维护和定位问题

- 接口必须进行参数合法性检查。

错误示例：

```rust
pub fn load_user_profile(user_id: &str) -> Result<UserProfile, UserError> {
    repository_load(user_id)
}
```

正确示例：

```rust
pub fn load_user_profile(user_id: &str) -> Result<UserProfile, UserError> {
    if user_id.is_empty() {
        return Err(UserError::InvalidUserId);
    }

    repository_load(user_id)
}
```

- 外部输入、跨模块输入、配置输入、文件输入和网络输入必须进行合法性检查。

- 禁止保留死代码、无用函数、无用变量和无用 trait。

- 重复代码必须提取为函数、方法、trait 或模块接口。

- 函数嵌套层级超过 3 层时，必须优先通过提前返回或拆分函数降低复杂度。

- 复杂条件必须拆分为具名变量或独立函数。

## 13. 程序效率

### 优先保证清晰，确认瓶颈后再优化

- 不要为了未经验证的性能收益牺牲代码清晰度。

- 只有确认性能是瓶颈时，才进行针对性优化。

- 循环内不要重复执行不变计算、重复 IO、重复查询或重复正则编译。

- 不要无意义地 `clone()`。

错误示例：

```rust
for user in users {
    process_name(user.name.clone());
}
```

正确示例：

```rust
for user in users {
    process_name(&user.name);
}
```

- 大量字符串拼接必须使用合适的缓冲区或迭代器方式。

错误示例：

```rust
let mut result = String::new();
for item in items {
    result = result + item;
}
```

正确示例：

```rust
let result = items.join("");
```

## 14. 注释与文档

### 注释说明原因，不重复代码

- 文档内容和代码注释必须使用中文，除非项目已有其他语言规范。

- 公共函数、公共类型、公共 trait、公共模块必须提供文档注释。

- 文档注释使用 `///`。

- 模块文档使用 `//!`。

示例：

```rust
//! 用户资料加载模块。

/// 用户资料。
pub struct UserProfile {
    pub user_id: String,
}
```

- 复杂业务规则、边界条件、外部系统约束必须写注释。

## 15. 格式与控制流

### 格式必须交给 rustfmt

- 代码必须通过 rustfmt 格式化。

- 默认缩进为 4 个空格，行宽以项目 rustfmt 配置为准。

- 复杂条件必须拆分为具名变量或独立函数。

错误示例：

```rust
if user.is_some() && user.as_ref().unwrap().is_active && retry_count < max_retry_count {
    retry_login(user.unwrap());
}
```

正确示例：

```rust
if let Some(user) = user {
    let can_retry = user.is_active && retry_count < max_retry_count;
    if can_retry {
        retry_login(user);
    }
}
```

- `if let` 用于清晰处理单个模式匹配。

示例：

```rust
if let Some(user) = user {
    process_user(user);
}
```

- `match` 必须覆盖所有分支。

示例：

```rust
match status {
    UserStatus::Active => enable_user(user),
    UserStatus::Inactive => disable_user(user),
}
```

- 不要写深层嵌套。嵌套超过 3 层时，优先提前返回或拆分函数。

- `for` 循环必须明确遍历对象，不要在循环中无意义复制数据。

- `while` 循环必须有明确退出条件。

## 16. 测试

### 代码必须可测试

- 新增业务逻辑必须配套测试。

- 修复 bug 时必须增加能复现该 bug 的测试。

- 测试模块使用 `#[cfg(test)]`。

示例：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_user_profile_missing_user_returns_error() {
        assert!(load_user_profile("missing").is_err());
    }
}
```

## 17. 安全性

### 所有外部输入都不可信

- 所有外部输入必须校验。

- 不要把外部输入直接拼接到 SQL、Shell 命令或文件路径中。

- 文件路径来自外部输入时，必须校验路径合法性。

- 不要在日志中记录密码、token、密钥、身份证号等敏感信息。

### 命令执行安全

- 执行外部命令时，必须使用参数数组，不要拼接命令字符串。

错误示例：

```rust
Command::new("sh")
    .arg("-c")
    .arg(format!("cat {}", file_name))
    .status()?;
```

正确示例：

```rust
Command::new("cat")
    .arg(file_name)
    .status()?;
```

### 路径安全

- 外部输入路径必须限制在允许目录内。

示例：

```rust
let full_path = base_dir.join(file_name);
if !full_path.starts_with(base_dir) {
    return Err(UserError::InvalidPath);
}
```

## 18. 工具与检查

### 新代码必须通过项目工具链

- 必须使用项目指定 Rust 版本或 MSRV。

- 代码必须通过 `cargo fmt`。

- 代码必须通过项目要求的 `cargo clippy` 规则。

- 代码必须通过 `cargo test` 或项目指定测试命令。

## 19. 代码修改要求

- 修改代码前必须阅读相关上下文。

- 必须优先保持项目现有架构、命名和风格。

- 不要做与当前任务无关的重构。

- 新增功能时必须同时添加或更新相关测试。

- 修改完成后必须运行相关格式化、lint 或测试命令；如果无法运行，必须说明原因。
