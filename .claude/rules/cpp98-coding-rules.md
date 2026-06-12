# C++98 编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 C++98 代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

参考来源取向：Google C++ Style Guide、C++ Core Guidelines 中适用于传统 C++ 的部分、华为 C/C++ 编码规范风格，以及 C++98 工程实践。

## 0. 规范说明

### 0.1 适用范围

本规范适用于必须兼容 C++98 的业务代码、基础库代码、嵌入式代码、系统代码和测试代码。

如果项目已有更具体的编码规范、编译器约束或平台约束，必须优先使用项目已有规范。

### 0.2 总体原则

### 清晰第一。

说明：C++98 缺少现代 C++ 的很多安全设施，代码必须通过清晰接口、RAII 和严格资源管理降低风险。

### 不使用超出 C++98 的特性。

说明：禁止在 C++98 项目中使用 `auto`、lambda、`nullptr`、右值引用、范围 for、智能指针标准库等 C++11+ 特性。

## 1. 头文件

### 头文件只暴露接口

- 每个对外提供接口的 `.cc`、`.cpp` 文件必须有对应头文件。

- 头文件必须自包含。

- 头文件必须使用 include guard。

示例：

```cpp
#ifndef ACCOUNT_USER_PROFILE_H_
#define ACCOUNT_USER_PROFILE_H_

#include <string>

class UserProfile;

int LoadUserProfile(const std::string& user_id, UserProfile* profile);

#endif
```

- 能使用前置声明时，不要包含完整头文件。

- 禁止在头文件中定义非 inline 的函数和变量。

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

#include <string>
#include <vector>

#include "base/status.h"
```

- `if`、`else`、`for`、`while`、`switch` 的代码块必须使用大括号。

- 指针和引用符号的位置必须与项目既有风格一致。

- 不要在一行写多个语句。

- 删除行尾空格。

- 文件末尾必须保留一个换行符。

## 3. 命名

### 命名必须表达含义

- 文件名使用小写英文单词，多个单词使用下划线连接。

- 类型名、类名使用项目约定风格；没有约定时使用 `PascalCase`。

- 函数名必须表达动作和对象。

- 常量名必须表达含义。

- 全局变量必须使用统一前缀。项目没有既有约定时，使用 `g_`。

示例：

```cpp
int g_user_count = 0;
```

- 文件作用域静态变量必须使用统一前缀。项目没有既有约定时，使用 `s_`。

- 文件名必须使用小写英文单词，多个单词使用下划线连接。

示例：

```text
user_profile.cpp
user_profile.h
config_loader.cpp
```

- 类名、结构体名、枚举名必须表达业务含义。

- 枚举值必须使用统一前缀，避免命名冲突。

示例：

```cpp
enum UserStatus {
    USER_STATUS_INACTIVE = 0,
    USER_STATUS_ACTIVE = 1
};
```

- 不要使用 `common.cpp`、`utils.cpp`、`misc.cpp` 这类职责模糊的文件名，除非项目已有明确约定。

## 4. 函数

### 一个函数只完成一个功能

- 函数名必须准确表达功能。

- 函数一般不超过 80 行。

- 参数过多时，必须使用结构体组织参数。

- 指针参数必须说明是否允许为 `NULL`。

- 输出参数必须在接口注释中说明。

- 函数名必须优先使用“动词 + 名词”结构。

- 初始化和清理函数必须成对命名。项目没有既有约定时，使用 `Init` 和 `Deinit`。

示例：

```cpp
int InitUserCache(UserCache* cache);
void DeinitUserCache(UserCache* cache);
```

错误示例：

```cpp
int HandleData(const char* text);
```

正确示例：

```cpp
int ParseUserProfile(const char* text, UserProfile* profile);
```

## 5. 接口

### 接口必须表达稳定契约

- 被多个模块、多个调用点复用的能力，必须提取为公共接口。

- 所有接口必须有中文注释。

- 接口注释必须说明功能、参数和返回。

- 接口命名必须表达业务语义，不要使用 `HandleData`、`ProcessInfo` 这类模糊名称。

示例：

```cpp
/*
 * 功能:
 *   根据用户 ID 加载用户资料。
 *
 * 参数:
 *   user_id: 用户唯一 ID，不能为空。
 *   profile: 输出参数，用于保存用户资料，不能为空。
 *
 * 返回:
 *   0 表示成功；非 0 表示失败。
 */
int LoadUserProfile(const std::string& user_id, UserProfile* profile);
```

## 6. 类

### 类必须维护明确不变量

- 类必须表示明确业务概念、资源所有权或状态封装。

- 构造函数必须让对象进入有效状态。

- 析构函数不得抛出异常。

- 需要多态删除的基类析构函数必须为 virtual。

- 不要在构造函数或析构函数中调用依赖派生类行为的虚函数。

示例：

```cpp
class UserProfileLoader {
public:
    UserProfileLoader();
    virtual ~UserProfileLoader();

    int Load(const std::string& user_id, UserProfile* profile);
};
```

## 7. 资源管理

### 使用 RAII 管理资源

- 每个资源获取动作必须有对应释放动作。

- 资源必须封装到类中，通过析构函数释放。

错误示例：

```cpp
FILE* file = fopen(path, "r");
ReadFile(file);
fclose(file);
```

正确示例：

```cpp
#include <cstdio>

class FileHandle {
public:
    explicit FileHandle(FILE* file) : file_(file) {}
    ~FileHandle() {
        if (file_ != NULL) {
            fclose(file_);
        }
    }

private:
    FileHandle(const FileHandle&);
    FileHandle& operator=(const FileHandle&);

    FILE* file_;
};
```

- 不要让调用方猜测谁负责释放资源。

- 禁止在同一表达式中执行多个资源分配。

## 8. 指针与内存

### 指针所有权必须清晰

- `T*` 参数必须说明是否允许为 `NULL`。

- 返回裸指针时必须说明所有权。

- 动态分配内存后必须有明确释放路径。

- `new` 和 `delete` 必须成对出现。

- 数组使用 `new[]` 和 `delete[]` 成对释放。

- 优先使用标准容器管理动态数组。

错误示例：

```cpp
int* values = new int[count];
delete values;
```

正确示例：

```cpp
std::vector<int> values(count);
```

## 9. 错误处理

### 错误必须被处理或向上传递

- 错误处理方式必须与项目约定一致。

- 使用返回值表示错误时，调用方必须检查。

- 使用异常时，异常类型必须表达业务含义。

- 禁止吞掉异常。

错误示例：

```cpp
try {
    LoadConfig(path);
} catch (...) {
}
```

正确示例：

```cpp
try {
    LoadConfig(path);
} catch (const ConfigError& error) {
    return ERROR_LOAD_CONFIG;
}
```

## 10. 质量保证

### 代码必须便于检查、维护和定位问题

- 接口必须进行参数合法性检查。

错误示例：

```cpp
int LoadUserProfile(const std::string& user_id, UserProfile* profile) {
    return repository.Load(user_id, profile);
}
```

正确示例：

```cpp
int LoadUserProfile(const std::string& user_id, UserProfile* profile) {
    if (user_id.empty() || profile == NULL) {
        return ERROR_INVALID_PARAM;
    }

    return repository.Load(user_id, profile);
}
```

- 外部输入、跨模块输入、配置输入、文件输入和网络输入必须进行合法性检查。

- 禁止保留死代码、无用函数、无用变量和无用类。

- 重复代码必须提取为函数、类方法或公共接口。

- 函数嵌套层级超过 3 层时，必须优先通过提前返回或拆分函数降低复杂度。

## 11. 程序效率

### 优先保证清晰，确认瓶颈后再优化

- 不要为了未经验证的性能收益牺牲代码清晰度。

- 循环内不要重复执行不变计算。

错误示例：

```cpp
for (size_t i = 0; i < users.size(); ++i) {
    if (NormalizeName(users[i].name()).size() > 0) {
        ProcessUser(users[i]);
    }
}
```

正确示例：

```cpp
for (size_t i = 0; i < users.size(); ++i) {
    std::string name = NormalizeName(users[i].name());
    if (!name.empty()) {
        ProcessUser(users[i]);
    }
}
```

- 只读大对象参数优先使用 `const T&`。

错误示例：

```cpp
bool ValidateUser(UserProfile profile);
```

正确示例：

```cpp
bool ValidateUser(const UserProfile& profile);
```

## 12. 注释

### 注释说明原因，不重复代码

- 文档内容和代码注释必须使用中文，除非项目已有其他语言规范。

- 所有接口函数、公共类、公共方法必须提供中文注释。

- 接口注释必须说明功能、参数和返回。

- 复杂业务规则、边界条件、外部系统约束必须写注释。

## 13. 控制流

### 表达式必须简单明确

- `if`、`else`、`for`、`while`、`switch` 的代码块必须使用大括号。

- 复杂条件必须拆分为具名变量或独立函数。

错误示例：

```cpp
if (user != NULL && user->status() == USER_STATUS_ACTIVE && retry_count < max_retry_count) {
    RetryLogin(user);
}
```

正确示例：

```cpp
bool can_retry = user != NULL &&
    user->status() == USER_STATUS_ACTIVE &&
    retry_count < max_retry_count;
if (can_retry) {
    RetryLogin(user);
}
```

- `switch` 的每个 `case` 必须明确处理 `break`、`return` 或注释说明 fallthrough。

- 避免深层嵌套。嵌套超过 3 层时，优先提前返回或拆分函数。

## 14. 安全与可移植

### 不依赖未定义行为

- 不要依赖整数溢出、越界访问、悬空指针或未初始化变量。

- 不要依赖 `int`、`long`、指针的固定长度，除非平台明确保证。

- 不要使用编译器扩展语法，除非项目明确允许。

- 外部输入必须校验。

### 字符串与格式化安全

- 字符串拷贝、拼接和格式化必须检查目标缓冲区大小。

错误示例：

```cpp
strcpy(dst, src);
```

正确示例：

```cpp
snprintf(dst, dst_size, "%s", src);
```

- 禁止将外部输入直接作为格式字符串。

错误示例：

```cpp
printf(user_input);
```

正确示例：

```cpp
printf("%s", user_input);
```

### 整数安全

- 整数运算涉及长度、容量、偏移、索引时，必须检查溢出风险。

## 15. 测试与工具

### 新代码必须通过项目工具链

- 新增业务逻辑必须配套测试。

- 修复 bug 时必须增加能复现该 bug 的测试。

- 编译警告必须处理，不要新增明显警告。

- 如果项目使用 cpplint、clang-tidy、cppcheck 或 CodeArts Check，新代码必须通过相关规则。

## 16. 代码修改要求

- 修改代码前必须阅读相关上下文。

- 必须优先保持项目现有架构、命名和风格。

- 不要使用 C++11 或以上特性。

- 修改完成后必须运行相关编译、静态检查或测试命令；如果无法运行，必须说明原因。
