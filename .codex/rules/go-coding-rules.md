# Go 编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 Go 代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

参考来源取向：Effective Go、Go Code Review Comments、Google Go Style Guide、Uber Go Style Guide、gofmt/go vet/staticcheck 等常见 Go 工程实践。

## 0. 规范说明

### 0.1 适用范围

本规范适用于 Go 编写的业务服务、基础库、命令行工具、自动化工具和测试代码。

如果项目已有更具体的编码规范、Go 版本约束、lint 配置或安全规范，必须优先使用项目已有规范。

### 0.2 总体原则

### 简单直接。

说明：Go 代码必须优先保持简单、显式、可读，不要把其他语言的复杂抽象习惯搬到 Go 中。

### 错误显式处理。

说明：Go 代码必须显式处理错误，不要吞掉错误，不要用 panic 代替正常错误返回。

### 工具统一格式。

说明：Go 代码格式必须交给 gofmt 或 gofmt 兼容工具处理。

## 1. 包与文件

### 包必须表达清晰职责

- 包名必须短小、清晰、全小写。

- 包名不要使用下划线、驼峰或复数形式，除非项目已有约定。

- 文件名使用小写英文单词，多个单词使用下划线连接。

示例：

```text
user_profile.go
user_profile_test.go
```

- 不要使用 `common`、`utils`、`misc` 这类职责模糊的包名。

- 包内文件可以按功能拆分，但必须属于同一包职责。

## 2. import

### import 必须清晰表达依赖

- import 必须由 gofmt/goimports 组织。

- 禁止保留无用 import。

- 禁止使用 dot import，测试中特殊场景除外。

错误示例：

```go
import . "fmt"
```

正确示例：

```go
import "fmt"
```

- 空白 import 必须写注释说明原因。

示例：

```go
import _ "github.com/lib/pq" // 注册 PostgreSQL 驱动。
```

## 3. 代码风格

### 代码风格必须由 gofmt 固化

- Go 代码必须使用 gofmt 或 gofmt 兼容工具格式化。

- import 必须使用 goimports 或项目约定工具整理。

- 缩进、空格、换行、大括号位置必须以 gofmt 输出为准。

- 包名必须短小、清晰、全小写。

- 文件名使用小写英文单词，多个单词使用下划线连接。

- 导出名称使用 `PascalCase`。

- 非导出名称使用 `camelCase`。

- 错误变量统一命名为 `err`。

- 上下文变量统一命名为 `ctx`。

- 不要手工对齐制造额外空格。

错误示例：

```go
name        := "alice"
retryCount  := 3
```

正确示例：

```go
name := "alice"
retryCount := 3
```

- Go 文档注释必须以被注释标识符开头。

示例：

```go
// LoadUserProfile 根据用户 ID 加载用户资料。
func LoadUserProfile(userID string) (*UserProfile, error) {
    return nil, nil
}
```

## 4. 函数

### 一个函数只完成一个功能

- 函数名必须准确表达功能。

错误示例：

```go
func HandleData(text string) error {
    return nil
}
```

正确示例：

```go
func ParseUserProfile(text string) (*UserProfile, error) {
    return nil, nil
}
```

- 函数一般不超过 50 行。超过时必须检查是否可以拆分。

- 参数过多时，必须使用结构体组织参数。

错误示例：

```go
func CreateUser(name string, age int, email string, phone string, level int) error {
    return nil
}
```

正确示例：

```go
type CreateUserRequest struct {
    Name  string
    Age   int
    Email string
    Phone string
    Level int
}

func CreateUser(req CreateUserRequest) error {
    return nil
}
```

- 不要为了单个实现提前定义接口。

## 5. 接口

### 接口由使用方定义

- 只有存在多个实现、测试替身或明确抽象边界时，才定义接口。

错误示例：

```go
type UserServiceInterface interface {
    LoadUserProfile(id string) (*UserProfile, error)
}
```

正确示例：

```go
type UserProfileLoader interface {
    LoadUserProfile(id string) (*UserProfile, error)
}
```

- 接口名必须表达能力。单方法接口项目没有约定时，可使用 `-er` 风格。

示例：

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}
```

- 导出的接口必须有中文注释。

- 接口注释必须说明功能、参数和返回。

- 不要使用 `Interface`、`I` 前缀或后缀表达接口。

错误示例：

```go
type IUserService interface {
    LoadUserProfile(id string) (*UserProfile, error)
}
```

正确示例：

```go
type UserProfileLoader interface {
    LoadUserProfile(id string) (*UserProfile, error)
}
```

## 6. 命名

### 命名必须清晰且符合 Go 惯例

- 包名使用小写短名。

- 导出名称使用 `PascalCase`。

- 非导出名称使用 `camelCase`。

- 常量、变量、函数命名必须表达业务含义。

- 不要使用无意义名称，例如 `data`、`info`、`tmp`，除非作用域很小且含义明确。

- 错误变量使用 `err`。

- 上下文参数命名为 `ctx`。

示例：

```go
func LoadUserProfile(ctx context.Context, userID string) (*UserProfile, error) {
    return nil, nil
}
```

- 全局变量必须避免使用。必须使用时，项目没有既有约定时使用 `g` 或明确业务名前缀，并通过函数封装访问。

错误示例：

```go
var UserCount int

UserCount++
```

正确示例：

```go
var gUserCount int

func IncreaseUserCount() {
    gUserCount++
}
```

## 7. 错误处理

### 错误必须显式处理

- 每个有意义的 `error` 返回值必须检查。

错误示例：

```go
profile, _ := LoadUserProfile(ctx, userID)
```

正确示例：

```go
profile, err := LoadUserProfile(ctx, userID)
if err != nil {
    return nil, err
}
```

- 错误向上传递时必须保留上下文。

示例：

```go
profile, err := LoadUserProfile(ctx, userID)
if err != nil {
    return nil, fmt.Errorf("load user profile %q: %w", userID, err)
}
```

- 错误字符串不要首字母大写，不要以句号结尾。

- 不要用 panic 处理正常业务错误。

## 8. 结构体与数据模型

### 结构体必须表达稳定数据结构

- 结构体字段必须表达业务含义。

- 导出字段必须有稳定语义。

- 不要长期使用 `map[string]interface{}` 代替明确结构。

错误示例：

```go
user := map[string]interface{}{
    "id":   "u1",
    "name": "Alice",
}
```

正确示例：

```go
type UserProfile struct {
    ID   string
    Name string
}
```

- 结构体较大时，传参优先使用指针。

## 9. 并发

### goroutine 生命周期必须清晰

- 启动 goroutine 必须明确退出条件。

- 不要泄漏 goroutine。

- 共享可变状态必须使用 channel、mutex、atomic 或其他明确同步机制。

- 不要在持锁状态下执行不可控耗时操作。

- 需要取消的操作必须传递 `context.Context`。

示例：

```go
func Watch(ctx context.Context) error {
    for {
        select {
        case <-ctx.Done():
            return ctx.Err()
        default:
            poll()
        }
    }
}
```

## 10. 质量保证

### 代码必须便于检查、维护和定位问题

- 接口必须进行参数合法性检查。

错误示例：

```go
func LoadUserProfile(ctx context.Context, userID string) (*UserProfile, error) {
    return repository.Load(ctx, userID)
}
```

正确示例：

```go
func LoadUserProfile(ctx context.Context, userID string) (*UserProfile, error) {
    if userID == "" {
        return nil, errors.New("user id is empty")
    }

    return repository.Load(ctx, userID)
}
```

- 外部输入、跨模块输入、配置输入、文件输入和网络输入必须进行合法性检查。

- 禁止保留死代码、无用函数、无用变量和无用接口。

- 重复代码必须提取为函数、方法或接口。

- 函数嵌套层级超过 3 层时，必须优先通过提前返回或拆分函数降低复杂度。

## 11. 程序效率

### 优先保证清晰，确认瓶颈后再优化

- 不要为了未经验证的性能收益牺牲代码清晰度。

- 只有确认性能是瓶颈时，才进行针对性优化。

- 循环内不要重复执行不变计算、重复 IO、重复查询或重复正则编译。

错误示例：

```go
for _, name := range names {
    if regexp.MustCompile(pattern).MatchString(name) {
        ProcessName(name)
    }
}
```

正确示例：

```go
compiledPattern := regexp.MustCompile(pattern)
for _, name := range names {
    if compiledPattern.MatchString(name) {
        ProcessName(name)
    }
}
```

- 大量字符串拼接必须使用 `strings.Builder` 或项目约定方式。

错误示例：

```go
result := ""
for _, item := range items {
    result += item
}
```

正确示例：

```go
var builder strings.Builder
for _, item := range items {
    builder.WriteString(item)
}
result := builder.String()
```

## 12. 注释与文档

### 注释说明原因，不重复代码

- 文档内容和代码注释必须使用中文，除非项目已有其他语言规范。

- 导出的包、类型、函数、方法、常量和变量必须有注释。

- Go 文档注释必须以被注释标识符开头。

示例：

```go
// LoadUserProfile 根据用户 ID 加载用户资料。
//
// 参数:
//   - userID: 用户唯一 ID，不能为空。
//
// 返回:
//   - 用户资料和错误信息。
func LoadUserProfile(userID string) (*UserProfile, error) {
    return nil, nil
}
```

- 复杂业务规则、边界条件、外部系统约束必须写注释。

## 13. 格式与控制流

### 格式必须交给 gofmt

- 所有 Go 代码必须通过 gofmt。

- 不要手工对齐制造额外空格。

- `if` 条件必须表达清晰。

- 复杂条件必须拆分为具名变量或独立函数。

错误示例：

```go
if user != nil && user.Status == StatusActive && user.RetryCount < maxRetryCount {
    RetryLogin(user)
}
```

正确示例：

```go
canRetry := user != nil &&
    user.Status == StatusActive &&
    user.RetryCount < maxRetryCount
if canRetry {
    RetryLogin(user)
}
```

- 不要写不必要的 `else`。

错误示例：

```go
if err != nil {
    return err
} else {
    return nil
}
```

正确示例：

```go
if err != nil {
    return err
}
return nil
```

- `switch` 必须覆盖关键分支；默认分支必须处理非法值或说明原因。

- `for` 循环必须明确退出条件。

- 不要在遍历 slice 时直接修改同一个 slice。

错误示例：

```go
for i, user := range users {
    if !user.Active {
        users = append(users[:i], users[i+1:]...)
    }
}
```

正确示例：

```go
activeUsers := users[:0]
for _, user := range users {
    if user.Active {
        activeUsers = append(activeUsers, user)
    }
}
```

- `select` 必须处理退出或取消信号，长期阻塞场景必须说明原因。

## 14. 测试

### 代码必须可测试

- 新增业务逻辑必须配套测试。

- 修复 bug 时必须增加能复现该 bug 的测试。

- 测试文件命名必须使用 `_test.go` 后缀。

- 测试函数命名必须使用 `TestXxx`。

- 表驱动测试必须使用清晰用例名。

示例：

```go
func TestLoadUserProfileMissingUser(t *testing.T) {
    _, err := LoadUserProfile(context.Background(), "missing")
    if err == nil {
        t.Fatal("expected error")
    }
}
```

## 15. 安全性

### 所有外部输入都不可信

- 所有外部输入必须校验。

- 不要把外部输入直接拼接到 SQL、Shell 命令、文件路径或模板中。

- 不要在日志中记录密码、token、密钥、身份证号等敏感信息。

- 使用 `crypto/rand` 生成安全随机数，不要用 `math/rand` 生成安全 token。

### SQL 安全

错误示例：

```go
query := "select * from users where id = '" + userID + "'"
```

正确示例：

```go
query := "select * from users where id = ?"
rows, err := db.QueryContext(ctx, query, userID)
```

### 命令执行安全

- 执行外部命令时，必须使用参数数组，不要拼接命令字符串。

错误示例：

```go
exec.Command("sh", "-c", "cat "+fileName)
```

正确示例：

```go
exec.Command("cat", fileName)
```

## 16. 工具与检查

### 新代码必须通过项目工具链

- 代码必须通过 gofmt。

- 代码必须通过项目要求的 go vet、staticcheck、golangci-lint 或等价工具。

- 代码必须通过 `go test ./...` 或项目指定测试命令。

## 17. 代码修改要求

- 修改代码前必须阅读相关上下文。

- 必须优先保持项目现有架构、命名和风格。

- 不要做与当前任务无关的重构。

- 新增功能时必须同时添加或更新相关测试。

- 修改完成后必须运行相关格式化、lint 或测试命令；如果无法运行，必须说明原因。
