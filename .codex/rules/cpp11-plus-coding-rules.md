# C++11 及以上编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 C++11 及以上代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

参考来源取向：Google C++ Style Guide、C++ Core Guidelines、LLVM/Chromium 等现代 C++ 工程实践，以及华为 C/C++ 编码规范风格。

## 0. 规范说明

### 0.1 适用范围

本规范适用于使用 C++11、C++14、C++17、C++20 或更高版本的业务代码、基础库代码、系统代码和测试代码。

项目使用的 C++ 标准必须以技术实现文档、构建配置或编译器参数为准。

### 0.2 总体原则

### 清晰第一。

说明：现代 C++ 提供了大量语言特性，必须优先选择能提升可读性和安全性的特性。

### RAII 优先。

说明：资源必须通过对象生命周期管理，避免裸露的 `new`、`delete` 和手动释放。

### 不滥用现代特性。

说明：lambda、模板、智能指针、移动语义等特性必须服务于清晰设计，不得为了炫技使用。

## 1. 头文件

### 头文件只暴露接口

- 每个对外提供接口的 `.cc`、`.cpp` 文件必须有对应头文件。

- 头文件必须自包含。

- 头文件必须使用 include guard 或项目约定方式。

- 能使用前置声明时，不要包含完整头文件。

- 头文件中不要使用 `using namespace`。

示例：

```cpp
#ifndef ACCOUNT_USER_PROFILE_H_
#define ACCOUNT_USER_PROFILE_H_

#include <string>

class UserProfile;

int LoadUserProfile(const std::string& user_id, UserProfile* profile);

#endif
```

## 2. 代码风格

### 代码风格必须由项目工具固化

- 如果项目有 `.clang-format`、cpplint 或其他格式化配置，必须以项目配置为准。

- 项目没有配置时，缩进使用 4 个空格，禁止使用 Tab。

- 单行代码长度默认不超过 120 字符；如果项目已有行宽配置，按项目配置执行。

- include 顺序必须清晰：
  1. 当前模块对应头文件
  2. 标准库头文件
  3. 第三方库头文件
  4. 项目内部头文件

- 每组 include 之间保留一个空行。

示例：

```cpp
#include "user_profile.h"

#include <memory>
#include <string>
#include <vector>

#include "base/status.h"
```

- `if`、`else`、`for`、`while`、`switch` 的代码块必须使用大括号。

- 指针和引用符号的位置必须与项目既有风格一致。

- `auto` 只能在类型明显或能显著降低噪音时使用。

- lambda 捕获列表必须显式表达需要捕获的变量。

- 不要在一行写多个语句。

- 删除行尾空格。

- 文件末尾必须保留一个换行符。

## 3. 命名

### 命名必须表达含义

- 文件名、类型名、函数名、变量名必须符合项目既有风格。

- 没有项目约定时，类型使用 `PascalCase`，函数使用 `PascalCase` 或项目统一风格，变量使用 `snake_case`。

- 常量名必须表达业务含义。

- 全局变量必须使用统一前缀。项目没有约定时使用 `g_`。

- 文件作用域静态变量必须使用统一前缀。项目没有约定时使用 `s_`。

- 文件名必须使用小写英文单词，多个单词使用下划线连接。

- 类名、结构体名、枚举名必须表达业务含义。

- 枚举类型优先使用 `enum class`。

## 4. 函数

### 一个函数只完成一个功能

- 函数名必须准确表达功能。

- 函数一般不超过 80 行。

- 参数过多时，必须使用结构体或参数对象组织参数。

- 只读大对象参数优先使用 `const T&`。

错误示例：

```cpp
bool ValidateUser(UserProfile profile);
```

正确示例：

```cpp
bool ValidateUser(const UserProfile& profile);
```

- 输出参数必须在接口注释中说明。

- 能用返回值表达结果时，不要滥用输出参数。

- 函数名必须优先使用“动词 + 名词”结构。

- 判断类函数必须使用 `Is`、`Has`、`Can`、`Should` 或项目约定前缀表达判断含义。

示例：

```cpp
bool IsUserActive(const UserProfile& user);
bool HasWritePermission(const UserProfile& user);
```

## 5. 接口

### 接口必须表达稳定契约

- 被多个模块、多个调用点复用的能力，必须提取为公共接口。

- 所有接口必须有中文注释。

- 接口注释必须说明功能、参数和返回。

示例：

```cpp
/*
 * 功能:
 *   根据用户 ID 加载用户资料。
 *
 * 参数:
 *   user_id: 用户唯一 ID，不能为空。
 *
 * 返回:
 *   用户资料对象。
 */
UserProfile LoadUserProfile(const std::string& user_id);
```

- 修改公共接口时，必须同步检查调用方、测试和文档。

- 接口命名必须表达业务语义，不要使用 `HandleData`、`ProcessInfo` 这类模糊名称。

## 6. 类

### 类必须维护明确不变量

- 类必须表示明确业务概念、资源所有权或状态封装。

- 构造函数必须让对象进入有效状态。

- 析构函数不得抛出异常。

- 需要多态删除的基类析构函数必须为 `virtual`。

- 重写虚函数必须使用 `override`。

错误示例：

```cpp
class Loader : public BaseLoader {
 public:
  void Load();
};
```

正确示例：

```cpp
class Loader : public BaseLoader {
 public:
  void Load() override;
};
```

- 优先组合，谨慎继承。

## 7. 现代特性使用

### 现代特性必须提升清晰度

- `auto` 只能在类型明显或能减少噪音时使用。

错误示例：

```cpp
auto value = LoadUserProfile(user_id);
```

正确示例：

```cpp
std::map<std::string, UserProfile>::iterator iter = profiles.find(user_id);
```

说明：如果项目允许，迭代器这类冗长类型可以使用 `auto`；业务返回值不清晰时不要使用。

- lambda 必须保持短小，不要把复杂业务逻辑塞进 lambda。

- 捕获列表必须显式表达需要捕获的变量，不要随意使用 `[=]` 或 `[&]`。

错误示例：

```cpp
auto matcher = [&]() { return user.status() == UserStatus::kActive; };
```

正确示例：

```cpp
auto matcher = [&user]() { return user.status() == UserStatus::kActive; };
```

- 移动语义只能用于明确转移所有权的场景。

- `std::move` 之后的对象不要再依赖其原有值。

## 8. 资源管理

### 使用 RAII 管理资源

- 不要裸露管理 `new` 和 `delete`，除非接口边界必须这样做。

- 独占所有权使用 `std::unique_ptr`。

- 共享所有权必须有明确理由，才允许使用 `std::shared_ptr`。

- 不要使用 `std::shared_ptr` 代替清晰的对象生命周期设计。

错误示例：

```cpp
UserProfile* profile = new UserProfile();
Load(profile);
delete profile;
```

正确示例：

```cpp
#include <memory>

std::unique_ptr<UserProfile> profile(new UserProfile());
Load(profile.get());
```

- 锁必须使用锁对象管理生命周期。

示例：

```cpp
std::lock_guard<std::mutex> lock(mutex_);
UpdateCache();
```

## 9. 类型与转换

### 类型转换必须显式表达意图

- 空指针使用 `nullptr`。

- 不要使用 C 风格强转。

错误示例：

```cpp
int value = (int)size;
```

正确示例：

```cpp
int value = static_cast<int>(size);
```

- 不要通过类型转换掩盖设计问题。

- 避免窄化转换。

- 枚举类型优先使用 `enum class`。

示例：

```cpp
enum class UserStatus {
  kInactive,
  kActive,
};
```

## 10. 模板与泛型

### 模板必须有明确收益

- 不要为了泛化而泛化。

- 公共模板接口必须有中文注释和使用约束说明。

- 模板错误信息复杂时，必须通过约束、静态断言或注释降低使用成本。

示例：

```cpp
template <typename T>
T MaxValue(const T& left, const T& right) {
  return left < right ? right : left;
}
```

## 11. 错误处理

### 错误必须被处理或向上传递

- 错误处理方式必须与项目约定一致。

- 禁止在禁用异常的项目中引入异常控制流。

- 使用返回值表示错误时，调用方必须检查。

- 使用异常时，异常类型必须表达业务含义。

- 不要吞掉异常。

## 12. 并发

### 共享状态必须有明确同步机制

- 共享可变状态必须使用 mutex、atomic、channel/queue 或项目约定机制保护。

- 不要在没有同步保护的情况下读写共享可变状态。

- 不要在持锁状态下执行不可控耗时操作。

- 后台线程必须有明确退出流程。

## 13. 质量保证

### 代码必须便于检查、维护和定位问题

- 接口必须进行参数合法性检查。

错误示例：

```cpp
UserProfile LoadUserProfile(const std::string& user_id) {
  return repository.Load(user_id);
}
```

正确示例：

```cpp
UserProfile LoadUserProfile(const std::string& user_id) {
  if (user_id.empty()) {
    throw UserError("user_id is empty");
  }

  return repository.Load(user_id);
}
```

- 外部输入、跨模块输入、配置输入、文件输入和网络输入必须进行合法性检查。

- 禁止保留死代码、无用函数、无用变量和无用类。

- 重复代码必须提取为函数、类方法或公共接口。

- 函数嵌套层级超过 3 层时，必须优先通过提前返回或拆分函数降低复杂度。

## 14. 程序效率

### 优先保证清晰，确认瓶颈后再优化

- 不要为了未经验证的性能收益牺牲代码清晰度。

- 循环内不要重复执行不变计算。

- 只读大对象参数优先使用 `const T&`。

- 不要在高频路径中进行不必要的内存分配和释放。

错误示例：

```cpp
for (const auto& user : users) {
  std::regex pattern(regex_text);
  if (std::regex_match(user.name(), pattern)) {
    ProcessUser(user);
  }
}
```

正确示例：

```cpp
std::regex pattern(regex_text);
for (const auto& user : users) {
  if (std::regex_match(user.name(), pattern)) {
    ProcessUser(user);
  }
}
```

## 15. 注释

### 注释说明原因，不重复代码

- 文档内容和代码注释必须使用中文，除非项目已有其他语言规范。

- 所有接口函数、公共类、公共方法必须提供中文注释。

- 接口注释必须说明功能、参数和返回。

- 复杂业务规则、边界条件、外部系统约束必须写注释。

## 16. 控制流

### 表达式必须简单明确

- `if`、`else`、`for`、`while`、`switch` 的代码块必须使用大括号。

- 复杂条件必须拆分为具名变量或独立函数。

错误示例：

```cpp
if (user != nullptr && user->status() == UserStatus::kActive && retry_count < max_retry_count) {
  RetryLogin(user);
}
```

正确示例：

```cpp
bool can_retry = user != nullptr &&
    user->status() == UserStatus::kActive &&
    retry_count < max_retry_count;
if (can_retry) {
  RetryLogin(user);
}
```

- 范围 for 可以用于清晰遍历。

示例：

```cpp
for (const auto& user : users) {
  ProcessUser(user);
}
```

- `switch` 的每个 `case` 必须明确处理 `break`、`return` 或注释说明 fallthrough。

- 避免深层嵌套。嵌套超过 3 层时，优先提前返回或拆分函数。

## 17. 安全与可移植

### 不依赖未定义行为

- 不要依赖整数溢出、越界访问、悬空指针或未初始化变量。

- 不要依赖 `int`、`long`、指针的固定长度，除非平台明确保证。

- 需要固定位宽时使用 `<cstdint>` 中的类型或项目约定类型。

- 外部输入必须校验。

- 不要在日志中记录密码、token、密钥、身份证号等敏感信息。

### 字符串与格式化安全

- 字符串拷贝、拼接和格式化必须检查目标缓冲区大小。

- 禁止将外部输入直接作为格式字符串。

错误示例：

```cpp
printf(user_input.c_str());
```

正确示例：

```cpp
printf("%s", user_input.c_str());
```

### 整数安全

- 整数运算涉及长度、容量、偏移、索引时，必须检查溢出风险。

## 18. 测试与工具

### 新代码必须通过项目工具链

- 新增业务逻辑必须配套测试。

- 修复 bug 时必须增加能复现该 bug 的测试。

- 编译警告必须处理，不要新增明显警告。

- 如果项目使用 clang-format、clang-tidy、cppcheck、cpplint 或 CodeArts Check，新代码必须通过相关规则。

## 19. 代码修改要求

- 修改代码前必须阅读相关上下文。

- 必须优先保持项目现有架构、命名和风格。

- 不要做与当前任务无关的重构。

- 不要使用超出项目 C++ 标准的语言特性。

- 修改完成后必须运行相关编译、静态检查或测试命令；如果无法运行，必须说明原因。
