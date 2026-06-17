# TypeScript 编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 TypeScript 代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

参考来源取向：TypeScript Handbook、TypeScript ESLint 推荐规则、现代前端工程实践、Node.js 与浏览器端 TypeScript 常见工程约定。

## 0. 规范说明

### 0.1 适用范围

本规范适用于使用 TypeScript 编写的业务代码、基础库代码、前端代码、Node.js 代码、构建脚本和测试代码。

如果项目已有更具体的编码规范、`tsconfig.json`、ESLint 配置、格式化配置或运行环境约束，必须优先使用项目已有规范。

### 0.2 总体原则

### 类型先行。

说明：TypeScript 代码必须利用类型系统表达数据结构、函数边界和状态变化，不得把 TypeScript 当作带注释的 JavaScript 使用。

### 清晰第一。

说明：代码首先应让维护者理解输入、输出、副作用和错误边界。不要为了炫技使用复杂泛型、条件类型或过度抽象。

### 边界明确。

说明：外部输入、第三方库返回值、网络响应、文件内容和跨进程数据都必须在边界处校验或转换，内部代码应使用明确可信的类型。

### 风格一致。

说明：新增代码必须符合项目已有目录结构、命名风格、模块拆分、错误处理和测试方式。

## 1. 文件与模块

### 模块职责必须单一

- 每个 `.ts` 文件必须有清晰职责。

- 不要把无关类型、函数、常量和副作用逻辑放进同一个文件。

- 文件名必须使用项目既有命名风格；没有既有约定时，普通模块使用 `kebab-case` 或 `camelCase`，类型文件避免使用含义模糊的 `types.ts`。

- 禁止使用 `utils.ts`、`common.ts`、`helper.ts` 作为长期堆放无关函数的文件名。

错误示例：

```text
utils.ts
common.ts
data.ts
```

正确示例：

```text
date-format.ts
user-session.ts
request-error.ts
```

- 对外导出必须表达模块稳定接口。

- 内部实现不要导出；仅测试需要访问实现细节时，优先通过公开行为测试，不要为了测试扩大导出范围。

### import 与 export 必须清晰

- import 必须放在文件顶部。

- 禁止保留无用 import。

- 禁止使用过宽的命名空间导入，除非第三方库本身推荐这种方式。

错误示例：

```ts
import * as user from './user';
```

正确示例：

```ts
import { loadUserProfile } from './user';
```

- 禁止使用 `export *` 扩散不稳定实现，除非模块就是明确的 barrel 文件。

- barrel 文件不得造成循环依赖。

## 2. 类型声明

### 类型必须表达业务含义

- 公共函数参数、返回值和导出对象必须显式声明类型。

- 内部局部变量可依赖类型推导，但不得牺牲可读性。

- 禁止使用 `any` 绕过类型检查。

错误示例：

```ts
function parseUser(input: any): any {
  return input;
}
```

正确示例：

```ts
interface UserProfile {
  id: string;
  name: string;
}

function parseUser(input: unknown): UserProfile {
  if (!isUserProfile(input)) {
    throw new Error('invalid user profile');
  }

  return input;
}
```

- 外部输入必须先使用 `unknown` 接收，再通过校验函数、schema 或解析函数转换为内部类型。

- 不要用类型断言掩盖不确定数据。

错误示例：

```ts
const user = JSON.parse(text) as UserProfile;
```

正确示例：

```ts
const parsed: unknown = JSON.parse(text);
const user = parseUserProfile(parsed);
```

### interface 与 type 使用必须稳定

- 对象结构优先使用 `interface`，特别是需要被实现、扩展或作为公共 API 的结构。

- union、tuple、函数签名、映射类型和组合类型优先使用 `type`。

示例：

```ts
interface UserProfile {
  id: string;
  name: string;
}

type UserRole = 'admin' | 'operator' | 'viewer';
```

- 不要为了少写代码把不同语义的字段合并为宽泛类型。

错误示例：

```ts
type Status = string;
```

正确示例：

```ts
type RequestStatus = 'idle' | 'loading' | 'success' | 'error';
```

### 可空类型必须显式处理

- 开启 `strictNullChecks` 时，不得使用非空断言 `!` 逃避判断。

错误示例：

```ts
const title = item.title!.trim();
```

正确示例：

```ts
if (item.title == null) {
  return '';
}

const title = item.title.trim();
```

- 可选字段必须在读取前处理缺省值或分支。

- 函数返回空值时，必须明确使用 `null` 或 `undefined`，不要混用。

## 3. 命名

### 命名必须表达语义

- 变量和函数使用 `camelCase`。

- 类型、接口、类和枚举使用 `PascalCase`。

- 常量使用 `SCREAMING_SNAKE_CASE`，仅限真正固定的全局常量。

- 布尔变量和判断函数使用 `is`、`has`、`can`、`should` 等前缀。

示例：

```ts
const MAX_RETRY_COUNT = 3;

interface UserProfile {
  id: string;
}

function isExpired(expiresAt: Date): boolean {
  return expiresAt.getTime() <= Date.now();
}
```

- 不要使用 `data`、`info`、`obj`、`tmp`、`res` 这类含义模糊的名称，除非上下文非常短且明确。

- 异步函数名应体现动作，不必强制添加 `Async` 后缀，除非项目已有约定。

### 接口、类型和 API 命名必须稳定

- `interface` 名称必须使用名词或名词短语，不要使用 `I` 前缀。

错误示例：

```ts
interface IUserInfo {
  id: string;
}
```

正确示例：

```ts
interface UserProfile {
  id: string;
}
```

- 表示请求参数的类型使用 `Input`、`Params` 或 `Request` 后缀，必须与项目既有风格一致。

- 表示返回结果的类型使用 `Result`、`Response` 或明确的领域名，必须与项目既有风格一致。

示例：

```ts
interface CreateUserInput {
  name: string;
  email: string;
}

interface CreateUserResult {
  id: string;
}
```

- 表示配置的类型使用 `Options` 或 `Config` 后缀，不要混用。

- 表示事件载荷的类型使用 `Payload`、`Event` 或明确领域名。

- API、service、repository 方法名必须使用“动词 + 名词”结构。

示例：

```ts
function loadUserProfile(userId: string): Promise<UserProfile> {
  // ...
}

function updateUserProfile(input: UpdateUserProfileInput): Promise<UpdateUserProfileResult> {
  // ...
}
```

- 获取单个对象使用 `get` 或 `load`，查询列表使用 `list`、`search` 或 `query`，创建使用 `create`，修改使用 `update`，删除使用 `delete` 或 `remove`，校验使用 `validate`。

- `handle` 只用于事件处理函数，不要作为普通业务函数前缀。

### 变量和局部变量必须具体

- 局部变量必须使用具体名称表达内容和用途。

错误示例：

```ts
const data = await loadUserProfile(userId);
const list = data.items;
```

正确示例：

```ts
const userProfile = await loadUserProfile(userId);
const permissionItems = userProfile.permissions;
```

- 临时变量允许使用短名称，但作用域必须很小。

示例：

```ts
for (const item of items) {
  result.push(item.id);
}
```

- 数组变量使用复数或集合语义命名。

示例：

```ts
const users: UserProfile[] = [];
const selectedIds = new Set<string>();
```

- Map、Set、Record 变量名必须体现键和值的含义。

示例：

```ts
const userById = new Map<string, UserProfile>();
const permissionsByRole: Record<RoleName, Permission[]> = {};
```

- 布尔变量必须表达真假语义。

错误示例：

```ts
const status = user.enabled;
```

正确示例：

```ts
const isEnabled = user.enabled;
```

- 禁止为了省字母使用无意义缩写。

错误示例：

```ts
const usr = loadUser();
const cfg = loadConfig();
```

正确示例：

```ts
const user = loadUser();
const config = loadConfig();
```

## 4. 语句与控制流

### if 必须表达清晰分支

- `if` 条件必须可读。复杂条件必须提取为具名变量或函数。

错误示例：

```ts
if (user.enabled && user.roles.includes('admin') && !user.locked && user.expiresAt > now) {
  grantAccess();
}
```

正确示例：

```ts
const canAccessAdmin = isAdminUser(user) && isUserAvailable(user, now);

if (canAccessAdmin) {
  grantAccess();
}
```

- 必须使用花括号，不允许省略单行 `if` 的花括号。

错误示例：

```ts
if (isValid) return;
```

正确示例：

```ts
if (isValid) {
  return;
}
```

- 优先使用提前返回减少嵌套。

错误示例：

```ts
function submit(input: FormInput) {
  if (isValid(input)) {
    if (hasPermission(input)) {
      return save(input);
    }
  }

  return null;
}
```

正确示例：

```ts
function submit(input: FormInput) {
  if (!isValid(input)) {
    return null;
  }

  if (!hasPermission(input)) {
    return null;
  }

  return save(input);
}
```

- `else` 分支不得过长。已经提前 `return`、`throw`、`continue` 或 `break` 时，不要再写不必要的 `else`。

### for 与循环必须选择合适形式

- 遍历数组或可迭代对象时，优先使用 `for...of`。

示例：

```ts
for (const user of users) {
  activateUser(user);
}
```

- 需要索引时使用普通 `for` 或 `entries()`。

示例：

```ts
for (const [index, user] of users.entries()) {
  updateUserOrder(user, index);
}
```

- 禁止使用 `for...in` 遍历数组。

错误示例：

```ts
for (const index in users) {
  activateUser(users[index]);
}
```

- 遍历对象属性时，使用 `Object.keys`、`Object.values` 或 `Object.entries`，并处理类型边界。

示例：

```ts
for (const [role, permissions] of Object.entries(permissionsByRole)) {
  registerPermissions(role, permissions);
}
```

- 不要在循环中执行与循环无关的重复计算，应提前提取。

- 循环中存在 `await` 时，必须确认需要串行执行；可以并发时使用 `Promise.all`。

错误示例：

```ts
for (const user of users) {
  await loadUserProfile(user.id);
}
```

正确示例：

```ts
const profiles = await Promise.all(
  users.map((user) => loadUserProfile(user.id)),
);
```

### switch 必须覆盖完整分支

- `switch` 应用于有限枚举、union type 或明确状态分支。

- 每个 `case` 必须使用 `break`、`return` 或 `throw` 明确结束。

- 对 union type 的 `switch` 必须使用 `never` 做穷尽检查。

示例：

```ts
type LoadStatus = 'idle' | 'loading' | 'success' | 'error';

function getStatusText(status: LoadStatus): string {
  switch (status) {
    case 'idle':
      return 'Idle';
    case 'loading':
      return 'Loading';
    case 'success':
      return 'Success';
    case 'error':
      return 'Error';
    default: {
      const exhaustiveCheck: never = status;
      return exhaustiveCheck;
    }
  }
}
```

- `default` 不得吞掉未处理状态。无法处理时必须抛出错误或显式返回安全结果。

### return 必须保持函数出口清晰

- 函数返回值类型必须稳定。

- 不要在同一函数中混合返回 `null`、`undefined`、空对象和错误对象表达失败。

- 多个提前返回必须对应明确的边界条件。

- 不要在 `finally` 中 `return`，避免吞掉异常或覆盖前面的返回值。

错误示例：

```ts
try {
  return await loadConfig();
} finally {
  return getDefaultConfig();
}
```

### 条件表达式必须保持简单

- 三元表达式只用于简单赋值或简单渲染判断。

错误示例：

```ts
const result = enabled ? hasPermission ? save(input) : reject(input) : skip(input);
```

正确示例：

```ts
if (!enabled) {
  return skip(input);
}

if (!hasPermission) {
  return reject(input);
}

return save(input);
```

- 空值合并使用 `??`，不要用 `||` 处理可能为 `0`、`false` 或空字符串的有效值。

错误示例：

```ts
const retryCount = input.retryCount || 3;
```

正确示例：

```ts
const retryCount = input.retryCount ?? 3;
```

## 5. 函数

### 函数必须短小且边界清晰

- 一个函数只完成一个明确功能。

- 函数名必须准确表达函数行为。

- 函数一般不超过 50 行。超过时必须检查是否可以拆分。

- 参数超过 3 个时，必须考虑使用对象参数。

错误示例：

```ts
function createUser(name: string, age: number, email: string, enabled: boolean) {
  // ...
}
```

正确示例：

```ts
interface CreateUserInput {
  name: string;
  age: number;
  email: string;
  enabled: boolean;
}

function createUser(input: CreateUserInput) {
  // ...
}
```

- 函数不得隐式修改输入对象，除非函数名和类型明确表达该副作用。

- 纯函数和有副作用函数应分开组织。

### 返回值必须稳定

- 函数不得在不同分支返回结构不一致的对象。

错误示例：

```ts
function loadResult(ok: boolean) {
  if (ok) {
    return { data: [] };
  }

  return { error: 'failed' };
}
```

正确示例：

```ts
type LoadResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: Error };
```

- 可预期失败优先使用明确的结果类型或领域错误；不可恢复异常才抛出异常。

## 6. 类与对象

### 类只用于需要状态和行为绑定的场景

- 不要为了组织函数而创建无状态类。

错误示例：

```ts
class DateUtils {
  static format(value: Date): string {
    return value.toISOString();
  }
}
```

正确示例：

```ts
function formatDate(value: Date): string {
  return value.toISOString();
}
```

- 类字段必须声明可见性和只读性。

示例：

```ts
class SessionStore {
  private readonly sessions = new Map<string, Session>();
}
```

- 构造函数不得执行网络请求、文件写入、定时器启动等复杂副作用。

- 继承必须谨慎使用，优先组合。

## 7. 异步代码

### Promise 必须被处理

- 调用返回 `Promise` 的函数必须使用 `await`、`return` 或显式处理错误。

错误示例：

```ts
saveUser(user);
```

正确示例：

```ts
await saveUser(user);
```

- 禁止混用 `async/await` 和长链式 `.then()`，除非链式写法明显更清晰。

- 并发任务必须明确表达并发关系。

示例：

```ts
const [profile, permissions] = await Promise.all([
  loadProfile(userId),
  loadPermissions(userId),
]);
```

- `setTimeout`、事件监听、订阅和轮询必须有清理机制。

## 8. 错误处理

### 错误必须可理解、可定位

- 捕获异常后不得静默吞掉。

错误示例：

```ts
try {
  await saveConfig(config);
} catch {
  // ignore
}
```

正确示例：

```ts
try {
  await saveConfig(config);
} catch (error) {
  throw new ConfigSaveError('failed to save config', { cause: error });
}
```

- `catch` 中的错误类型必须按 `unknown` 处理。

示例：

```ts
catch (error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  logger.error(message);
}
```

- 不要抛出字符串。

错误示例：

```ts
throw 'failed';
```

正确示例：

```ts
throw new Error('failed');
```

- 错误转换应保留原始 `cause`。

## 9. 数据与集合

### 数据转换必须显式

- 不要在业务逻辑中散落重复的字段转换。

- 外部 DTO 和内部模型应在边界处转换。

- 数组转换优先使用 `map`、`filter`、`reduce`，但复杂逻辑应拆成命名函数。

- 不要在 `map`、`filter`、`reduce` 回调中写大量副作用逻辑。

- 对象合并时必须注意字段覆盖顺序。

示例：

```ts
const nextUser = {
  ...currentUser,
  displayName,
};
```

## 10. 注释与文档

### 注释解释原因，不重复代码

- 公共 API、复杂类型、非显然边界和兼容性处理必须有注释。

- 不要写重复代码含义的注释。

错误示例：

```ts
// increment count by one
count += 1;
```

正确示例：

```ts
// Keep this branch compatible with older serialized payloads.
if (input.version == null) {
  return migrateLegacyPayload(input);
}
```

- TODO 必须说明后续动作或阻塞原因。

## 11. 测试代码

### 测试必须覆盖可观察行为

- 测试文件命名必须符合项目约定，常见形式为 `*.test.ts` 或 `*.spec.ts`。

- 测试名称必须描述输入、动作和期望结果。

- 不要只测试实现细节。

- 外部输入解析、错误处理、异步失败、边界值和类型保护函数必须测试。

- mock 必须局部、明确，测试结束后恢复。

示例：

```ts
it('returns error result when payload is invalid', () => {
  const result = parsePayload({ id: 1 });

  expect(result.ok).toBe(false);
});
```

## 12. 禁止写法

- 禁止使用 `any` 绕过类型检查。

- 禁止滥用 `as` 类型断言。

- 禁止使用非空断言 `!` 逃避空值判断。

- 禁止保留无用代码、无用 import 和调试输出。

- 禁止在模块顶层执行不可控副作用。

- 禁止通过全局可变变量保存业务状态，除非项目已有明确架构约定。

- 禁止把外部输入直接当作内部可信类型使用。

- 禁止为了复用而提前设计复杂泛型抽象。
