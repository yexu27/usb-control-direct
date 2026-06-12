# Python 编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 Python 代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

## 0. 规范说明

### 0.1 适用范围

本规范适用于使用 Python 编写的业务代码、基础库代码、工具脚本、自动化脚本、服务端代码和测试代码。

如果项目已有更具体的编码规范、技术实现文档、运行环境约束、格式化配置、静态检查规则或安全规范，必须优先使用项目已有规范。

### 0.2 总体原则

### 清晰第一。

说明：代码首先是给人阅读和维护的，其次才是给解释器执行的。新增代码必须让后续维护者能够直接理解主要意图。

### 简洁为美。

说明：不写不必要的代码，不引入不必要的抽象，不保留无用函数、无用变量、无用类和死代码。重复逻辑必须提取为函数、类方法或公共接口。

### 风格一致。

说明：修改已有代码时，必须保持项目现有命名、排版、注释、异常处理、测试和依赖管理方式一致。不要因为个人偏好改变项目风格。

### 版本兼容。

说明：Python 版本必须以项目技术实现文档、运行环境、依赖配置或现有代码为准。不要在通用编码规范中强制指定 Python 版本。

## 1. 文件与模块

### 模块职责必须清晰

说明：Python 文件就是模块。模块边界不清晰会导致依赖混乱、测试困难和后续维护成本上升。

- 每个 `.py` 文件必须职责清晰。

- 不要把无关功能放进同一个模块。

- 模块名必须使用小写英文单词。

- 多个单词组成的模块名使用下划线连接。

示例：

```text
user_profile.py
config_loader.py
order_validator.py
```

- 不要使用 `common.py`、`utils.py`、`misc.py` 这类职责模糊的文件名，除非项目已有明确约定。

错误示例：

```text
utils.py
common.py
data.py
```

正确示例：

```text
user_cache.py
order_validator.py
config_loader.py
```

- 模块中只对外暴露调用方需要使用的接口。

- 模块内部实现细节必须使用单下划线前缀或放在内部模块中。

示例：

```python
def load_user_profile(user_id):
    return _load_from_repository(user_id)


def _load_from_repository(user_id):
    ...
```

## 2. import

### import 必须清晰表达依赖关系

说明：import 是模块依赖的入口。依赖越混乱，越容易出现循环导入和启动副作用。

- import 必须放在文件顶部，除非用于避免循环依赖、降低启动成本或处理可选依赖。

- import 顺序必须清晰：
  1. 标准库
  2. 第三方库
  3. 项目内部模块

- 每组 import 之间保留一个空行。

示例：

```python
import json
from pathlib import Path

import requests

from account.user_profile import load_user_profile
```

- 禁止使用 `from module import *`。

错误示例：

```python
from user_profile import *
```

正确示例：

```python
from user_profile import load_user_profile
```

- 禁止保留无用 import。

- 不要在 import 阶段执行耗时操作、网络请求、数据库连接或文件写入。

- 项目内部模块之间必须保持依赖方向清晰。基础模块不得反向依赖业务模块。

## 3. 函数

### 一个函数只完成一个功能

说明：函数越短、职责越单一，越容易测试、复用和维护。

- 函数名必须准确表达函数功能。

错误示例：

```python
def handle_data(data):
    ...
```

正确示例：

```python
def parse_user_profile(text):
    ...
```

- 函数名必须优先使用“动词 + 名词”结构。

示例：

```python
def load_user_profile(user_id):
    ...


def parse_config_file(path):
    ...
```

- 判断类函数必须使用 `is_`、`has_`、`can_`、`should_` 等表达判断含义的前缀。

示例：

```python
def is_user_active(user):
    ...


def has_write_permission(user):
    ...
```

- 新增函数一般不超过 50 行。

说明：超过 50 行时，必须检查是否可以拆分。只有拆分会降低可读性时才保留。

- 函数参数一般不超过 4 个。

说明：参数过多时，必须优先使用 dataclass、配置对象、参数对象或关键字参数组织参数。

错误示例：

```python
def create_user(name, age, email, phone, address, level):
    ...
```

正确示例：

```python
def create_user(request):
    ...
```

- 不要使用可变对象作为默认参数。

错误示例：

```python
def add_user(name, roles=[]):
    roles.append("user")
    return roles
```

正确示例：

```python
def add_user(name, roles=None):
    if roles is None:
        roles = []
    roles.append("user")
    return roles
```

- 函数如果会写文件、访问网络、修改数据库、修改全局状态或触发外部系统行为，函数名或注释必须清楚表达。

- 重复出现 2 次及以上的业务逻辑必须提取为独立函数。

- 只有一个调用点且没有稳定复用价值的逻辑，不要过早提取为公共接口。

## 4. 接口

### 接口必须表达稳定契约

说明：接口是模块之间、文件之间、类之间或函数之间的调用边界。接口必须让调用方不用阅读内部实现就能正确使用。

- 被多个模块、多个调用点或多个业务流程复用的能力，必须提取为公共接口。

- 公共接口可以是公共函数、公共类、公共方法、公共模块或稳定的数据模型。

- 公共接口必须放在语义清晰的位置，不要放在临时脚本、测试文件或流程函数内部。

- 所有接口必须有中文 docstring。

- 接口 docstring 必须说明功能、参数和返回。

- 参数说明必须明确参数含义和是否允许为空。

- 返回说明必须明确返回值含义。

- 修改公共接口时，必须同步检查调用方、测试和文档。

接口 docstring 示例：

```python
def load_user_profile(user_id):
    """根据用户 ID 加载用户资料。

    参数:
        user_id: 用户唯一 ID，不能为空。

    返回:
        用户资料数据。
    """
    ...
```

## 5. 命名

### 命名必须清晰表达含义

说明：命名不清晰会迫使维护者阅读实现细节，增加理解成本。

- 命名必须使用英文单词或业界通用缩写。

- 不要使用汉语拼音命名。

- 不要使用无意义名称，例如 `data`、`info`、`tmp`、`obj`，除非上下文非常明确。

- 模块名、包名、函数名、变量名必须使用 `snake_case`。

- 类名必须使用 `PascalCase`。

- 常量名必须使用全大写加下划线。

示例：

```python
MAX_RETRY_COUNT = 3


class UserProfile:
    ...


def load_user_profile(user_id):
    ...
```

- 私有函数、私有方法、私有属性必须使用单下划线前缀。

示例：

```python
def _load_from_repository(user_id):
    ...
```

- 布尔变量或布尔函数必须表达判断语义。

示例：

```python
is_ready = True
has_error = False
should_retry = True
```

- 全局变量必须使用统一前缀。项目没有既有约定时，使用 `g_` 前缀。

示例：

```python
g_user_count = 0
```

- 全局变量命名必须表达业务含义，不要只写 `g_data`、`g_info`、`g_value` 这类模糊名称。

## 6. 变量与常量

### 变量作用域越小越好

说明：变量作用域越大，越难判断其状态变化。

- 变量必须在使用前赋值。

- 不要依赖未初始化变量或条件分支中可能不存在的变量。

错误示例：

```python
if enabled:
    result = load_user_profile(user_id)

return result
```

正确示例：

```python
result = None
if enabled:
    result = load_user_profile(user_id)

return result
```

- 局部变量不得与全局变量同名。

- 不要随意使用全局变量。

- 必须使用全局状态时，必须限制修改入口，并说明原因。

- 禁止跨模块直接修改全局变量，必须优先通过接口访问。

错误示例：

```python
user_cache.g_user_count += 1
```

正确示例：

```python
user_cache.increase_user_count()
```

- 常量必须表达业务含义，不要使用魔法值。

错误示例：

```python
if retry_count > 3:
    return False
```

正确示例：

```python
MAX_RETRY_COUNT = 3

if retry_count > MAX_RETRY_COUNT:
    return False
```

## 7. 类型与数据模型

### 类型信息必须与项目 Python 版本兼容

说明：类型信息用于表达接口契约和数据结构。类型写法必须以项目 Python 版本为准。

- Python 3 项目中，新增函数必须添加参数类型标注和返回值类型标注。

- Python 2.7 项目或不支持函数注解的旧项目中，不要使用 Python 3 函数注解；必须通过 docstring、类型注释或 `.pyi` stub 文件表达接口类型。

- Python 3 项目中，返回空值时使用 `-> None`。

- Python 2.7 项目或不支持函数注解的旧项目中，返回空值必须在 docstring 或类型注释中说明。

- Python 3.9+ 项目可以使用内建泛型，例如 `list[str]`、`dict[str, int]`。

- Python 3.10+ 项目可以使用联合类型语法，例如 `str | None`。

- Python 3.8 及以下项目必须使用 `typing.List`、`typing.Dict`、`typing.Optional`、`typing.Union` 等兼容写法。

- 避免使用 `Any`。只有在第三方库边界或动态数据确实无法准确描述时才允许使用。

- 复杂结构必须使用 dataclass、TypedDict、NamedTuple、Pydantic 模型或项目已有数据模型表达，不要长期使用裸 `dict`。

错误示例：

```python
user = {
    "id": 1,
    "name": "Alice",
    "status": "active",
}
```

正确示例：

```python
from dataclasses import dataclass


@dataclass
class UserProfile:
    user_id: int
    name: str
    status: str
```

## 8. 类

### 类必须表示明确的业务概念或状态封装

说明：类用于表达对象、状态、资源或行为边界。不要把无状态工具函数强行放进类里。

- 类名必须表达业务概念。

- 类的公开方法必须保持少而清晰。

- 私有方法只用于降低类内部复杂度，不得被外部调用。

- 类如果只是保存数据，优先使用 dataclass 或项目已有数据模型。

- 优先组合，谨慎继承。

- 继承必须表达稳定的 “is-a” 关系。

- 构造函数必须让对象进入有效状态。

- 不要在对象初始化后留下半初始化状态。

错误示例：

```python
class Helper:
    def handle(self, data):
        ...
```

正确示例：

```python
class UserProfileLoader:
    def load(self, user_id):
        ...
```

## 9. 异常处理

### 错误必须被处理或向上传递

说明：异常处理必须保留问题上下文，不能吞掉错误。

- 只捕获能够处理的异常。

- 禁止裸 `except:`。

- 禁止吞掉异常而不记录、不处理。

错误示例：

```python
try:
    result = parse_config(path)
except Exception:
    pass
```

正确示例：

```python
try:
    result = parse_config(path)
except IOError as exc:
    raise ConfigLoadError("Config file not found: {}".format(path))
```

- 捕获异常时必须使用具体异常类型。

- Python 3 项目中，重新抛出异常优先使用 `raise NewError(...) from exc` 保留异常链。

- Python 2.7 项目或不支持异常链的旧项目中，必须通过日志或错误消息保留原始异常信息。

- 面向业务的错误必须定义明确的自定义异常。

- 日志中不得记录密码、token、密钥、身份证号等敏感信息。

## 10. 资源与文件

### 资源必须明确获取和释放

说明：文件、网络连接、数据库连接、锁和临时资源必须有清晰生命周期。

- 文件读写必须使用上下文管理器。

- 打开文本文件时必须显式指定编码，默认使用 `encoding="utf-8"`。

- Python 2.7 项目中，文本文件读写必须使用 `io.open(..., encoding="utf-8")`。

示例：

```python
import io


def read_text(path):
    with io.open(path, "r", encoding="utf-8") as file:
        return file.read()
```

- 项目 Python 版本支持 pathlib 时，路径处理优先使用 `pathlib.Path`。

- 不要手动拼接路径字符串。

错误示例：

```python
path = base_dir + "/" + file_name
```

正确示例：

```python
path = base_dir / file_name
```

- 网络连接、数据库连接、锁和临时目录必须使用上下文管理器或明确释放。

## 11. 质量保证

### 代码必须便于检查、维护和定位问题

- 接口必须进行参数合法性检查。

错误示例：

```python
def load_user_profile(user_id):
    return repository.load(user_id)
```

正确示例：

```python
def load_user_profile(user_id):
    if not user_id:
        raise ValueError("user_id must not be empty")

    return repository.load(user_id)
```

- 外部输入、跨模块输入、配置输入、文件输入和网络输入必须进行合法性检查。

- 禁止保留死代码、无用函数、无用变量和无用类。

- 重复代码必须提取为函数、类方法或公共接口。

- 函数嵌套层级超过 3 层时，必须优先通过提前返回或拆分函数降低复杂度。

- 复杂条件必须拆分为具名变量或独立函数。

## 12. 程序效率

### 优先保证清晰，确认瓶颈后再优化

- 不要为了未经验证的性能收益牺牲代码清晰度。

- 只有确认性能是瓶颈时，才进行针对性优化。

- 循环内不要重复执行不变计算、重复 IO、重复查询或重复正则编译。

错误示例：

```python
for name in names:
    if re.match(pattern, name):
        process_name(name)
```

正确示例：

```python
import re


compiled_pattern = re.compile(pattern)
for name in names:
    if compiled_pattern.match(name):
        process_name(name)
```

- 大量字符串拼接必须使用 `str.join` 或项目约定方式。

错误示例：

```python
result = ""
for item in items:
    result += item
```

正确示例：

```python
result = "".join(items)
```

- 不要在高频循环中执行无必要的网络请求、数据库查询或文件读写。

## 13. 注释与文档

### 注释说明原因，不重复代码

- 文档内容和代码注释必须使用中文，除非项目已有其他语言规范。

- 所有接口函数、接口方法和公共类必须提供中文 docstring。

- 接口 docstring 必须说明功能、参数和返回。

- 模块注释必须说明模块职责。

- 类注释必须说明类的用途。

- 复杂业务规则、边界条件、外部系统约束必须写注释。

- 不要留下无效注释、过期注释或无说明的 TODO。

- TODO 必须说明待办原因、影响范围和后续处理方向。

示例：

```python
def calculate_discount(price, customer_level):
    """根据客户等级计算折扣金额。

    参数:
        price: 原始价格。
        customer_level: 客户等级。

    返回:
        折扣金额。
    """
    ...
```

## 14. 排版与控制流

### 排版必须服务于阅读

- 缩进必须使用 4 个空格，禁止使用 Tab。

- 行宽默认不超过 88 字符；如果项目已有行宽配置，按项目配置执行。

- 顶层函数和类之间保留 2 个空行。

- 类内部方法之间保留 1 个空行。

- 删除行尾空格。

- 文件末尾必须保留一个换行符。

- `if`、`elif`、`else` 的条件必须表达清晰。

- 复杂条件必须拆分为具名变量或独立函数。

错误示例：

```python
if user and user.status == "active" and user.retry_count < MAX_RETRY_COUNT:
    retry_login(user)
```

正确示例：

```python
can_retry = (
    user is not None
    and user.status == "active"
    and user.retry_count < MAX_RETRY_COUNT
)
if can_retry:
    retry_login(user)
```

- `for` 循环必须明确遍历对象。

- 不要在遍历列表时直接修改同一个列表。

错误示例：

```python
for user in users:
    if not user.is_active:
        users.remove(user)
```

正确示例：

```python
active_users = [user for user in users if user.is_active]
```

- `while` 循环必须有明确退出条件。

- 避免深层嵌套。嵌套超过 3 层时，必须优先使用提前返回、拆分函数或重构条件。

- 简单推导式可以使用，复杂逻辑不要塞进推导式。

## 15. 测试

### 代码必须可测试

- 新增业务逻辑必须配套测试。

- 修复 bug 时必须增加能复现该 bug 的测试。

- 接口必须有对应测试。

- 默认使用 pytest，除非项目已有其他测试框架。

- 测试文件命名使用 `test_*.py`。

- 测试函数命名使用 `test_<行为>_<场景>_<预期结果>`。

- 测试必须覆盖正常路径、边界条件和错误路径。

- 外部 API、文件系统、网络、数据库等副作用必须使用 mock、fixture、fake 或临时目录隔离。

示例：

```python
import pytest


def test_load_user_profile_missing_user_raises_error():
    with pytest.raises(UserNotFoundError):
        load_user_profile("missing-user")
```

## 16. 安全性

### 所有外部输入都不可信

- 所有外部输入必须校验。

- 不要把外部输入直接拼接到 SQL、Shell 命令、文件路径或模板中。

错误示例：

```python
query = "select * from users where id = '{}'".format(user_id)
```

正确示例：

```python
query = "select * from users where id = ?"
cursor.execute(query, (user_id,))
```

- 使用 `subprocess` 时必须传入参数列表，不要拼接命令字符串。

错误示例：

```python
import subprocess


subprocess.call("cat " + file_name, shell=True)
```

正确示例：

```python
import subprocess


subprocess.check_call(["cat", file_name])
```

- 不要在日志中记录密码、token、密钥、身份证号等敏感信息。

- 文件路径来自外部输入时，必须校验路径合法性。

- 反序列化、动态执行、模板渲染等高风险操作必须限制输入来源和执行范围。

## 17. 依赖与配置

### 依赖必须可维护、可复现

- 项目配置优先集中在 `pyproject.toml` 或项目已有配置文件中。

- 不要随意新增依赖。

- 新增依赖前必须确认标准库无法合理完成、依赖维护活跃、不会显著增加项目复杂度。

- 依赖版本必须可复现。

- 配置、密钥、环境差异必须通过环境变量或配置文件管理。

- 禁止在代码中硬编码密钥、token、密码、私钥、访问凭证或生产环境地址。

## 18. 工具与检查

### 新代码必须能在项目环境中检查和运行

- 格式化、lint、类型检查和测试命令必须以项目现有配置为准。

- 如果项目使用 ruff，新代码必须通过相关规则。

- 如果项目使用 black 或 ruff format，代码必须按项目配置格式化。

- 如果项目使用 mypy 或 pyright，新代码必须通过项目要求的类型检查。

- 如果项目没有现成配置，可以使用以下基础配置作为起点。

```toml
[tool.black]
line-length = 88

[tool.ruff]
line-length = 88

[tool.ruff.lint]
select = ["E", "F", "I", "B", "UP", "SIM"]

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
python_functions = ["test_*"]
```

## 19. 兼容性

### 版本相关假设必须显式表达

- Python 版本必须以项目技术实现文档、运行环境、依赖配置或现有代码为准。

- 不要使用超出项目 Python 版本支持范围的语法、标准库 API 或类型标注写法。

- Python 2.7 文件中包含中文注释、中文 docstring 或非 ASCII 字符时，文件头必须添加 `# -*- coding: utf-8 -*-`。

- 跨平台路径处理必须使用 pathlib、os.path 或项目约定方式。

- 平台相关逻辑必须集中隔离，不要散落在业务逻辑中。

## 20. 代码修改要求

- 修改代码前必须阅读相关上下文。

- 必须优先保持项目现有架构、命名和风格。

- 不要做与当前任务无关的重构。

- 不要删除用户已有改动，除非用户明确要求。

- 新增功能时必须同时添加或更新相关测试；如果无法添加测试，必须说明原因。

- 修改 bug 时必须先理解根因，再改代码。

- 修改完成后必须运行相关格式化、lint、类型检查或测试命令；如果无法运行，必须说明原因。

- 生成代码必须能直接运行，不要留下伪代码。
