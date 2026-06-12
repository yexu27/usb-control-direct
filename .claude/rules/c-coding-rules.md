# C 语言编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 C 语言代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

## 0. 规范说明

### 0.1 适用范围

本规范适用于使用 C 语言编写的业务代码、基础库代码、嵌入式代码、系统代码、工具代码和测试代码。

如果项目已有更具体的编码规范、技术实现文档、编译器约束、静态检查规则或安全规范，必须优先使用项目已有规范。

### 0.2 总体原则

### 清晰第一。

说明：代码首先是给人阅读和维护的，其次才是给机器执行的。新增代码必须让后续维护者能够直接理解主要意图。

### 简洁为美。

说明：不写不必要的代码，不引入不必要的抽象，不保留无用函数、无用变量、无用宏和死代码。重复逻辑必须提取为函数或公共接口。

### 风格一致。

说明：修改已有代码时，必须保持项目现有命名、排版、注释、错误处理和构建方式一致。不要因为个人偏好改变项目风格。

### 安全可靠。

说明：C 语言代码必须重点防止空指针、越界、内存泄漏、重复释放、未初始化变量、整数溢出和缓冲区溢出。

## 1. 头文件

### 头文件只暴露接口，不暴露实现细节

说明：头文件是模块对外提供能力的边界。头文件越复杂，编译依赖越复杂，维护成本越高。

- 每个对外提供接口的 `.c` 文件必须有对应的 `.h` 文件。

说明：公共函数、公共结构体、公共枚举、公共宏和公共常量必须通过头文件暴露。调用方只能通过包含头文件使用公共接口。

- 不需要对外暴露的函数必须定义为 `static`。

错误示例：

```c
int parse_config_value(const char *text)
{
    return 0;
}
```

正确示例：

```c
static int parse_config_value(const char *text)
{
    return 0;
}
```

- 禁止在 `.c` 文件中手写 `extern` 声明来使用其他模块接口。

说明：外部接口必须由提供方头文件声明，调用方通过 `#include` 使用。手写 `extern` 容易导致声明和定义不一致。

- 头文件必须自包含。

说明：任意源文件只包含该头文件时，该头文件必须能够独立通过编译。不要要求调用方先包含其他头文件。

- 禁止头文件循环依赖。

说明：循环依赖会导致编译成本上升，并使模块边界不清晰。

- 头文件依赖必须保持方向清晰。业务模块可以依赖基础模块，基础模块不得反向依赖业务模块。

- 能通过前置声明解决的依赖，不要在头文件中包含完整定义。

示例：

```c
typedef struct UserProfile UserProfile;

int load_user_profile(const char *user_id, UserProfile *profile);
```

- 头文件禁止包含用不到的头文件。

说明：头文件包含会传递编译依赖。能用前置声明解决的场景，不要包含完整头文件。

- 头文件必须使用 include guard 或项目约定的防重复包含方式。

示例：

```c
#ifndef USER_PROFILE_H
#define USER_PROFILE_H

typedef struct UserProfile UserProfile;

int load_user_profile(const char *user_id, UserProfile *profile);

#endif
```

- include guard 必须使用唯一名称。项目没有既有约定时，使用 `项目名_路径_文件名_H` 或 `模块名_文件名_H` 格式。

示例：

```c
#ifndef ACCOUNT_USER_PROFILE_H
#define ACCOUNT_USER_PROFILE_H

typedef struct UserProfile UserProfile;

int load_user_profile(const char *user_id, UserProfile *profile);

#endif
```

- include guard 的宏名前后不要放置会参与编译的代码。

- 禁止在头文件中定义变量。

错误示例：

```c
int g_user_count;
```

正确示例：

```c
extern int g_user_count;
```

说明：变量定义必须放在 `.c` 文件中。能不用全局变量时，不要使用全局变量。

- C/C++ 混编头文件必须正确使用 `extern "C"`。

示例：

```c
#ifdef __cplusplus
extern "C" {
#endif

int load_user_profile(const char *user_id, UserProfile *profile);

#ifdef __cplusplus
}
#endif
```

- 禁止在 `extern "C"` 块内部包含头文件。

## 2. 函数

### 一个函数只完成一个功能

说明：函数越短、职责越单一，越容易测试、复用和维护。

- 函数名必须准确表达函数功能。

错误示例：

```c
int handle_data(const char *text);
```

正确示例：

```c
int parse_user_profile(const char *text, UserProfile *profile);
```

- 新增函数一般不超过 80 行。

说明：超过 80 行时，必须检查是否可以拆分。只有拆分会降低可读性时才保留。

- 函数参数一般不超过 5 个。

说明：参数过多时，必须优先使用结构体组织参数。

- 指针参数必须说明是否允许为空。

- 数组、字符串和缓冲区参数必须同时传入长度。

错误示例：

```c
int copy_name(char *dst, const char *src);
```

正确示例：

```c
int copy_name(char *dst, size_t dst_size, const char *src);
```

- 输出参数必须在接口注释中明确说明。

- 函数返回值必须表达成功、失败或计算结果。

- 调用有意义返回值的函数时，必须检查返回值。

错误示例：

```c
load_user_profile(user_id, &profile);
```

正确示例：

```c
ret = load_user_profile(user_id, &profile);
if (ret != 0) {
    return ret;
}
```

- 重复出现 2 次及以上的业务逻辑必须提取为独立函数。

- 只有一个调用点且没有稳定复用价值的逻辑，不要过早提取为公共接口。

### 函数命名必须体现动作和对象

- 函数名必须优先使用“动词 + 名词”结构。

示例：

```c
int load_user_profile(const char *user_id, UserProfile *profile);
int parse_config_file(const char *path, Config *config);
```

- 判断类函数必须使用 `is_`、`has_`、`can_`、`should_` 等表达判断含义的前缀。

示例：

```c
int is_user_active(const User *user);
int has_write_permission(const User *user);
```

- 初始化和清理函数必须成对命名。项目没有既有约定时，使用 `xxx_init` 和 `xxx_deinit`。

示例：

```c
int user_cache_init(UserCache *cache);
void user_cache_deinit(UserCache *cache);
```

- 创建和销毁函数必须成对命名。项目没有既有约定时，使用 `xxx_create` 和 `xxx_destroy`。

示例：

```c
UserSession *user_session_create(const char *user_id);
void user_session_destroy(UserSession *session);
```

## 3. 接口

### 接口必须表达稳定契约

说明：接口是模块之间、文件之间或函数之间的调用边界。接口必须让调用方不用阅读内部实现就能正确使用。

- 被多个模块、多个调用点或多个业务流程复用的能力，必须提取为公共接口。

- 公共接口必须放在语义清晰的头文件中。

- 所有接口必须有中文注释。

- 接口注释必须说明功能、参数和返回。

- 参数注释必须说明输入参数、输出参数和输入输出参数。

- 返回注释必须说明返回值含义。使用错误码时，必须说明成功和失败的返回约定。

- 修改公共接口时，必须同步检查调用方、测试和文档。

接口注释示例：

```c
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
int load_user_profile(const char *user_id, UserProfile *profile);
```

## 4. 标识符命名

### 命名必须清晰表达含义

说明：命名不清晰会迫使维护者阅读实现细节，增加理解成本。

- 命名必须使用英文单词或业界通用缩写。

- 不要使用汉语拼音命名。

- 不要使用无意义名称，例如 `data`、`info`、`tmp`、`obj`，除非上下文非常明确。

- 布尔变量或布尔函数必须表达判断语义。

示例：

```c
int is_ready;
int has_error;
int should_retry;
```

- 全局变量必须使用统一前缀。项目没有既有约定时，使用 `g_` 前缀。

示例：

```c
int g_user_count;
static int g_retry_count;
```

- 全局变量命名必须表达业务含义，不要只写 `g_data`、`g_info`、`g_value` 这类模糊名称。

- 宏名和常量名必须表达含义。

- 不要使用容易混淆的名称，例如 `l`、`O`、`I`。

- 不要使用以下划线加大写字母开头的保留标识符。

### 文件命名必须体现模块职责

- 文件名必须使用小写英文单词。

- 多个单词组成的文件名使用下划线连接。

示例：

```text
user_profile.c
user_profile.h
config_loader.c
config_loader.h
```

- 文件名必须与模块职责一致。

- 对外接口头文件名必须与模块名保持一致。

- 不要使用 `common.c`、`utils.c`、`misc.c` 这类职责模糊的文件名，除非项目已有明确约定。

错误示例：

```text
utils.c
common.h
data.c
```

正确示例：

```text
user_cache.c
user_cache.h
order_validator.c
order_validator.h
```

### 类型命名必须体现数据含义

- `struct`、`enum`、`typedef` 名称必须表达业务含义。

- 枚举值必须使用统一前缀，避免不同枚举之间命名冲突。

示例：

```c
typedef enum {
    USER_STATUS_INACTIVE = 0,
    USER_STATUS_ACTIVE = 1
} UserStatus;
```

- 结构体成员命名必须表达字段含义。

示例：

```c
typedef struct UserProfile {
    int user_id;
    int login_count;
    int status;
} UserProfile;
```

- 不要为了隐藏指针而滥用 `typedef`。

错误示例：

```c
typedef UserProfile *UserProfileHandle;
```

说明：如果使用句柄类型，必须在接口注释中说明所有权和释放方式。

- 不要使用匿名结构体作为公共接口类型。

错误示例：

```c
typedef struct {
    int id;
    int status;
} UserProfile;
```

正确示例：

```c
typedef struct UserProfile {
    int id;
    int status;
} UserProfile;
```

## 5. 变量

### 变量作用域越小越好

说明：变量作用域越大，越难判断其状态变化。

- 变量必须在使用前定义，并在定义时初始化。

错误示例：

```c
int count;
printf("%d\n", count);
```

正确示例：

```c
int count = 0;
printf("%d\n", count);
```

- 局部变量不得与全局变量同名。

- 不要随意使用全局变量。

- 必须使用全局状态时，必须限制可见范围，并说明原因。

- 只在单个 `.c` 文件内使用的全局变量必须定义为 `static`。

- 文件作用域静态变量必须使用统一前缀。项目没有既有约定时，使用 `s_` 前缀。

示例：

```c
static int s_retry_count;
```

- 对外可见的全局变量必须使用 `g_` 前缀。

- 全局常量必须使用统一命名风格。项目没有既有约定时，使用全大写加下划线。

示例：

```c
const int MAX_RETRY_COUNT = 3;
```

- 禁止跨模块直接读写全局变量；必须优先通过接口访问。

错误示例：

```c
g_user_count++;
```

正确示例：

```c
increase_user_count();
```

- 指针变量必须明确是否可以为空。

- 数组和缓冲区必须明确容量。

## 6. 宏与常量

### 优先使用清晰的常量，不使用魔法值

- 禁止在代码中直接出现含义不明的魔法数字。

错误示例：

```c
if (retry_count > 3) {
    return -1;
}
```

正确示例：

```c
#define MAX_RETRY_COUNT 3

if (retry_count > MAX_RETRY_COUNT) {
    return -1;
}
```

- 宏参数必须加括号。

错误示例：

```c
#define SQUARE(x) x * x
```

正确示例：

```c
#define SQUARE(x) ((x) * (x))
```

- 多语句宏必须使用 `do { ... } while (0)` 包裹。

- 宏参数不要传入带副作用的表达式。

- 能使用函数或 `static inline` 函数时，不要使用复杂宏。

## 7. 内存与指针

### 谁申请，谁释放；谁转移，谁说明

- 动态分配的内存必须有明确释放路径。

- `malloc`、`calloc`、`realloc` 的返回值必须检查。

- `realloc` 不要直接赋值给原指针。

错误示例：

```c
buffer = realloc(buffer, new_size);
```

正确示例：

```c
new_buffer = realloc(buffer, new_size);
if (new_buffer == NULL) {
    return ERROR_NO_MEMORY;
}
buffer = new_buffer;
```

- 释放指针后，如果指针仍处于可见作用域，必须置为 `NULL`。

- 不要访问已释放内存。

- 不要重复释放同一指针。

- 不要返回局部变量地址。

- 解引用指针前必须确认指针有效。

- 指针运算必须保证不会越界。

## 8. 字符串与缓冲区

### 所有缓冲区访问必须有边界

- 字符串操作必须检查目标缓冲区大小。

- 不要使用不安全字符串函数，除非项目明确允许并有边界保护。

- 不要假设外部输入一定以 `\0` 结尾。

- 解析协议、文件、网络数据时，必须校验长度、范围、类型和格式。

- 整数运算涉及长度、容量、偏移、索引时，必须防止溢出。

## 9. 错误处理

### 错误必须被处理或向上传递

- 不要吞掉错误。

- 错误码必须有清晰含义。

- 错误路径必须释放已获得资源。

- 多层调用传递错误时，必须保留足够上下文。

- 公共接口必须明确错误返回值和错误码语义。

- 错误日志必须包含排查问题所需的关键信息。

- 不要在日志中记录密码、token、密钥、身份证号等敏感信息。

## 10. 质量保证

### 代码必须便于检查、维护和定位问题

- 接口必须进行参数合法性检查。

错误示例：

```c
int load_user_profile(const char *user_id, UserProfile *profile)
{
    return repository_load(user_id, profile);
}
```

正确示例：

```c
int load_user_profile(const char *user_id, UserProfile *profile)
{
    if ((user_id == NULL) || (profile == NULL)) {
        return ERROR_INVALID_PARAM;
    }

    return repository_load(user_id, profile);
}
```

- 外部输入、跨模块输入、配置输入、文件输入和网络输入必须进行合法性检查。

- 不要使用断言代替运行时错误处理。

- 断言只用于检查程序内部不变量，不用于处理外部输入错误。

- 所有错误分支必须有明确处理方式。

- 申请资源后的每个错误分支必须释放已申请资源。

示例：

```c
buffer = malloc(buffer_size);
if (buffer == NULL) {
    return ERROR_NO_MEMORY;
}

ret = read_config(buffer, buffer_size);
if (ret != 0) {
    free(buffer);
    return ret;
}
```

- 必须检查有意义的函数返回值。

- 禁止保留死代码、无用函数、无用变量和无用宏。

- 重复代码必须提取为函数或公共接口。

- 函数嵌套层级超过 3 层时，必须优先通过提前返回或拆分函数降低复杂度。

- 复杂条件必须拆分为具名变量或独立函数。

## 11. 程序效率

### 优先保证清晰，确认瓶颈后再优化

- 不要为了未经验证的性能收益牺牲代码清晰度。

- 只有确认性能是瓶颈时，才进行针对性优化。

- 循环内不要重复计算不变表达式。

错误示例：

```c
for (i = 0; i < strlen(name); i++) {
    process_char(name[i]);
}
```

正确示例：

```c
name_len = strlen(name);
for (i = 0; i < name_len; i++) {
    process_char(name[i]);
}
```

- 大结构体作为只读参数传递时，优先使用 `const` 指针。

错误示例：

```c
int validate_user_profile(UserProfile profile);
```

正确示例：

```c
int validate_user_profile(const UserProfile *profile);
```

- 频繁调用路径中不要进行不必要的内存分配和释放。

- IO、网络、文件和锁操作不得放在无必要的高频循环中。

- 字符串拼接必须考虑目标缓冲区大小和重复拷贝成本。

示例：

```c
ret = snprintf(path, path_size, "%s/%s", base_dir, file_name);
if ((ret < 0) || ((size_t)ret >= path_size)) {
    return ERROR_BUFFER_TOO_SMALL;
}
```

## 12. 注释

### 注释说明原因，不重复代码

- 文档内容和代码注释必须使用中文，除非项目已有其他语言规范。

- 所有接口函数必须提供中文注释。

- 接口函数注释必须说明功能、参数和返回。

- 公共头文件、公共结构体、公共枚举必须提供中文注释。

- 文件头注释用于说明文件职责。项目要求文件头注释时，必须按项目模板填写。

示例：

```c
/*
 * 文件功能:
 *   用户资料加载与缓存管理。
 */
```

- 模块注释必须说明模块职责、核心数据结构和对外接口。

- 结构体注释必须说明结构体用途。

示例：

```c
/*
 * 功能:
 *   保存用户资料的基础信息。
 */
typedef struct UserProfile {
    int user_id;
    int status;
} UserProfile;
```

- 枚举注释必须说明枚举用途，必要时说明枚举值含义。

- 宏注释必须说明宏用途、参数和使用限制。

示例：

```c
/*
 * 功能:
 *   判断错误码是否表示成功。
 *
 * 参数:
 *   code: 待判断的错误码。
 *
 * 返回:
 *   非 0 表示成功；0 表示失败。
 */
#define IS_SUCCESS(code) ((code) == 0)
```

- 全局变量注释必须说明用途、可见范围和修改约束。

- 复杂业务规则、边界条件、外部系统约束必须写注释。

- 不要留下无效注释、过期注释或无说明的 TODO。

- TODO 必须说明待办原因、影响范围和后续处理方向。

## 13. 排版与格式

### 排版必须服务于阅读

- 缩进必须使用 4 个空格，禁止使用 Tab，除非项目已有不同约定。

- 单行代码长度默认不超过 120 字符；如果项目已有行宽配置，按项目配置执行。

- 不要在一行写多个语句。

- 复杂表达式必须拆分为多行或中间变量。

- 删除行尾空格。

- 文件末尾必须保留一个换行符。

## 14. 表达式与控制流

### 表达式必须简单明确

- 不要在一个表达式中混合多个副作用。

- 不要依赖运算符优先级表达复杂逻辑，必须使用括号明确意图。

- `if`、`else if`、`else`、`for`、`while`、`do while`、`switch` 的代码块必须使用大括号。

错误示例：

```c
if (is_ready)
    start_service();
```

正确示例：

```c
if (is_ready) {
    start_service();
}
```

- `if` 条件必须表达清晰，复杂条件必须拆分为具名变量或独立函数。

错误示例：

```c
if ((user != NULL) && (user->status == STATUS_ACTIVE) && (user->retry_count < MAX_RETRY_COUNT)) {
    retry_login(user);
}
```

正确示例：

```c
can_retry = (user != NULL) &&
    (user->status == STATUS_ACTIVE) &&
    (user->retry_count < MAX_RETRY_COUNT);
if (can_retry) {
    retry_login(user);
}
```

- `if` 和 `else` 的分支都必须保持清晰。分支逻辑较长时，必须拆分为独立函数。
- 不要写空的 `if`、`else`、`for`、`while` 代码块；确实需要空块时，必须写注释说明原因。
- `for` 循环必须明确初始化条件、循环条件和迭代表达式。
- `for` 循环变量只用于控制循环，不要在循环体内随意修改循环变量。
- 遍历数组时，循环边界必须使用数组长度或明确的容量变量。

示例：

```c
for (i = 0; i < user_count; i++) {
    process_user(&users[i]);
}
```

- `while` 循环必须有明确退出条件。
- `while` 循环中影响退出条件的变量必须在循环体内有清晰更新。
- `do while` 只用于必须至少执行一次的场景。
- 避免深层嵌套。嵌套超过 3 层时，必须优先使用提前返回、拆分函数或重构条件。
- `switch` 表达式必须是枚举、整数或项目允许的明确类型。
- `switch` 的每个 `case` 必须明确处理 `break`、`return` 或注释说明 fallthrough。
- `case` 分支中如果定义局部变量，必须使用大括号限定作用域。

示例：

```c
switch (state) {
case STATE_READY:
    start_service();
    break;
case STATE_STOPPED:
    stop_service();
    break;
default:
    return ERROR_INVALID_STATE;
}
```

- `default` 分支必须处理非法值或说明为什么不需要处理。
- 不要使用 `goto`，除非用于集中资源清理，并且必须保持结构清晰。

## 15. 可测性与单元测试

### 代码必须可测试

- 新增业务逻辑必须配套测试。

- 修复 bug 时必须增加能复现该 bug 的测试。

- 接口必须有对应测试。

- 测试必须覆盖正常路径、边界条件和错误路径。

- 外部 API、文件系统、网络、数据库等副作用必须使用 mock、stub、fake 或测试夹具隔离。

- 测试不得依赖执行顺序。

- 测试不得依赖生产环境数据。

## 16. 安全性

### 所有外部输入都不可信

- 所有外部输入必须校验。

- 所有数组、字符串、缓冲区访问必须保证边界安全。

- 不要使用未初始化内存。

- 不要访问越界内存。

- 不要使用释放后的内存。

- 不要重复释放同一资源。

- 不要通过断言代替运行时安全检查。

- 接口必须校验非法参数。

### 字符串操作安全

- 字符串拷贝、拼接和格式化必须检查目标缓冲区大小。

错误示例：

```c
strcpy(dst, src);
```

正确示例：

```c
ret = snprintf(dst, dst_size, "%s", src);
if ((ret < 0) || ((size_t)ret >= dst_size)) {
    return ERROR_BUFFER_TOO_SMALL;
}
```

- 不要使用没有边界保护的字符串函数。

- 外部输入字符串不得假设一定以 `\0` 结尾。

### 整数安全

- 整数运算涉及长度、容量、偏移、索引时，必须检查溢出风险。

示例：

```c
#include <stdint.h>

if (item_count > (SIZE_MAX / sizeof(UserProfile))) {
    return ERROR_SIZE_OVERFLOW;
}

buffer_size = item_count * sizeof(UserProfile);
```

- 有符号整数和无符号整数混用时，必须确认比较和转换结果符合预期。

- 不同宽度整数转换时，必须确认不会截断有效数据。

### 格式化输出安全

- `printf`、`snprintf`、日志格式化等格式字符串必须使用固定格式。

- 禁止将外部输入直接作为格式字符串。

错误示例：

```c
printf(user_input);
```

正确示例：

```c
printf("%s", user_input);
```

### 文件 I/O 安全

- 文件路径来自外部输入时，必须校验路径合法性。

示例：

```c
if (!is_valid_config_path(path)) {
    return ERROR_INVALID_PATH;
}
```

- 打开文件后必须检查返回值。

- 文件读写必须检查实际读写长度。

- 文件句柄必须在所有路径上关闭。

## 17. 代码编辑与编译

### 新代码必须能在项目环境中编译和检查

- 构建命令必须以项目现有构建系统为准。

- 新增源文件必须加入项目构建配置。

- 编译警告必须处理；不要新增明显警告。

- 如果项目启用 `-Wall`、`-Wextra`、`-Werror` 或等价选项，新代码必须通过这些检查。

- 如果项目使用 clang-format、clang-tidy、cppcheck、CodeArts Check 或其他静态检查工具，新代码必须通过相关规则。

- 不要把编译产物提交到源码目录，除非项目已有明确约定。

- 头文件必须能独立编译检查。

示例：

```c
#include "user_profile.h"
```

说明：单独包含该头文件时必须可以编译通过。

- 新增宏、头文件和源文件必须通过项目支持的目标平台编译。

- 平台相关代码必须通过条件编译隔离，并保持条件编译结构清晰。

## 18. 可移植性

### 平台相关假设必须显式表达

- 不要依赖 `int`、`long`、指针等类型的固定长度，除非项目平台明确保证。

错误示例：

```c
long user_id;
```

正确示例：

```c
#include <stdint.h>

int64_t user_id;
```

- 需要固定位宽时，优先使用项目约定的定长类型；项目允许时使用 `stdint.h` 中的类型。

- 不要隐式依赖大小端。涉及网络协议、文件格式或跨平台数据时，必须明确字节序。

示例：

```c
packet_size = to_network_u32(host_size);
```

- 不要隐式依赖结构体内存布局。

- 结构体用于持久化、网络传输或跨语言交互时，必须显式定义字段序列化方式。

- 路径分隔符、换行符、文件打开模式必须考虑目标平台差异。

- 编译器扩展语法必须限制在平台适配层或项目允许范围内。

- 平台相关代码必须集中隔离，不要散落在业务逻辑中。

## 19. 代码修改要求

- 修改代码前必须阅读相关上下文。

- 必须优先保持项目现有架构、命名和风格。

- 不要做与当前任务无关的重构。

- 不要删除用户已有改动，除非用户明确要求。

- 新增功能时必须同时添加或更新相关测试；如果无法添加测试，必须说明原因。

- 修改 bug 时必须先理解根因，再改代码。

- 修改完成后必须运行相关编译、测试或静态检查命令；如果无法运行，必须说明原因。

- 生成代码必须能直接编译，不要留下伪代码。
