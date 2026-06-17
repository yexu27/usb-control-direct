# Vue 3 编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 Vue 3 代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

参考来源取向：Vue 3 官方文档、Vue Style Guide、Composition API 推荐实践、Pinia 文档、Vue Router 文档、Element Plus 常见工程实践。

## 0. 规范说明

### 0.1 适用范围

本规范适用于 Vue 3 单文件组件、组合式函数、Pinia store、Vue Router 配置、组件库封装和 Vue 组件测试代码。

如果项目已有更具体的组件规范、目录结构、ESLint 配置、格式化配置、UI 规范或测试规范，必须优先使用项目已有规范。

### 0.2 总体原则

### 组件边界清晰。

说明：组件必须有明确职责。页面组件负责组织流程和数据，可复用组件负责展示和交互，组合式函数负责复用状态逻辑。

### 状态来源单一。

说明：同一份状态只能有一个权威来源。不要在 props、store、本地 ref 和路由参数之间重复保存同一状态。

### 模板保持简单。

说明：模板应表达结构和绑定，不承载复杂业务计算。复杂逻辑必须放入计算属性、函数或组合式函数。

### 类型完整。

说明：Vue 代码必须使用 TypeScript 明确声明 props、emits、store state、路由 meta 和组件公开接口。

## 1. 单文件组件

### 统一使用 Vue 3 组件写法

- 新增组件必须使用 Vue 3 语法。

- 优先使用 `<script setup lang="ts">`。

- 禁止在同一组件中混用 Options API 和 Composition API，除非维护历史代码且项目已有约定。

示例：

```vue
<script setup lang="ts">
interface Props {
  title: string;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
});
</script>

<template>
  <button :disabled="props.disabled">
    {{ props.title }}
  </button>
</template>
```

- 单文件组件推荐顺序为 `<script setup>`、`<template>`、`<style scoped>`，具体顺序以项目既有风格为准。

- 样式必须限制作用域，优先使用 `scoped`、CSS Modules 或项目约定的样式隔离方案。

### 组件命名必须表达用途

- 组件文件名使用 `PascalCase.vue`，除非项目已有其它约定。

示例：

```text
UserTable.vue
ConfirmDialog.vue
```

- 组件名称必须使用多单词命名，根组件 `App.vue` 除外，避免与原生 HTML 元素冲突。

- 基础组件、业务组件、页面组件必须通过目录或命名体现层级。

- 不要使用 `Base.vue`、`Common.vue`、`Panel.vue` 这类含义过宽的组件名。

### 组件内变量和函数命名必须清晰

- 响应式变量必须表达状态含义，不要只用 `data`、`state`、`list`、`obj`。

错误示例：

```ts
const data = ref<UserProfile[]>([]);
const status = ref(false);
```

正确示例：

```ts
const userProfiles = ref<UserProfile[]>([]);
const isSubmitting = ref(false);
```

- `ref` 变量名称不需要添加 `Ref` 后缀，除非用于区分 DOM ref。

示例：

```ts
const username = ref('');
const submitButtonRef = ref<HTMLButtonElement | null>(null);
```

- `computed` 变量必须使用名词、形容词或判断语义命名。

示例：

```ts
const activeUsers = computed(() => users.value.filter((user) => user.active));
const canSubmit = computed(() => username.value.trim().length > 0);
```

- 事件处理函数使用 `handleXxx` 命名，只用于组件交互事件。

示例：

```ts
function handleSubmit() {
  emit('submit', formModel.value);
}
```

- 普通业务函数不得滥用 `handle` 前缀，应使用 `load`、`create`、`update`、`reset`、`validate` 等动词。

示例：

```ts
function validateForm(): boolean {
  return username.value.trim().length > 0;
}
```

- 表单模型使用 `xxxForm`、`xxxModel` 或项目既有约定，避免与提交参数、接口返回值混淆。

### 组件脚本控制流必须简单

- 组件方法中的 `if` 必须使用花括号，不允许省略。

错误示例：

```ts
if (isSubmitting.value) return;
```

正确示例：

```ts
if (isSubmitting.value) {
  return;
}
```

- 组件方法应优先使用提前返回减少嵌套。

示例：

```ts
async function handleSubmit() {
  if (isSubmitting.value) {
    return;
  }

  if (!validateForm()) {
    return;
  }

  isSubmitting.value = true;

  try {
    await submitForm(formModel.value);
  } finally {
    isSubmitting.value = false;
  }
}
```

- `for` 循环中修改响应式数组时，必须保证意图清晰。普通转换优先使用 `map`、`filter` 或计算属性。

错误示例：

```ts
for (const user of users.value) {
  activeUsers.value.push(user);
}
```

正确示例：

```ts
const activeUsers = computed(() => users.value.filter((user) => user.active));
```

- 需要顺序执行异步操作时可以在 `for...of` 中使用 `await`；可以并发时必须明确使用并发写法。

- `switch` 处理组件状态时，必须覆盖所有状态；状态较多时应抽成函数，不要把复杂分支写进模板。

## 2. props 与 emits

### props 必须声明类型

- 所有 props 必须声明 TypeScript 类型。

- 可选 props 必须提供合理默认值，或在模板和逻辑中显式处理空值。

- props 必须只读，禁止直接修改 props。

错误示例：

```vue
<script setup lang="ts">
const props = defineProps<{ modelValue: string }>();

props.modelValue = 'changed';
</script>
```

正确示例：

```vue
<script setup lang="ts">
const props = defineProps<{ modelValue: string }>();
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

function updateValue(value: string) {
  emit('update:modelValue', value);
}
</script>
```

- props 名称必须表达语义，不要使用 `data`、`item`、`value` 等模糊名称，除非组件语义非常明确。

### emits 必须声明事件契约

- 所有 emit 事件必须声明类型。

- 事件名使用项目既有风格；没有约定时，组件事件使用 kebab-case。

- 事件应表达已经发生的事实或请求父组件执行的动作。

示例：

```ts
const emit = defineEmits<{
  (event: 'submit', payload: FormPayload): void;
  (event: 'cancel'): void;
}>();
```

- 不要通过 emit 传递过大的对象或不稳定内部状态。

## 3. 模板

### 模板必须保持可读

- 模板中禁止写复杂表达式。

错误示例：

```vue
<template>
  <span>{{ users.filter((user) => user.active).map((user) => user.name).join(', ') }}</span>
</template>
```

正确示例：

```vue
<script setup lang="ts">
const activeUserNames = computed(() =>
  users.value.filter((user) => user.active).map((user) => user.name).join(', '),
);
</script>

<template>
  <span>{{ activeUserNames }}</span>
</template>
```

- `v-for` 必须提供稳定且唯一的 `key`。

- 禁止同时在同一元素上使用 `v-if` 和 `v-for`，应提前计算过滤结果。

- 条件渲染分支较多时，应拆分为子组件或计算属性。

- 模板中的事件处理器应调用命名函数，不要写复杂内联逻辑。

## 4. 响应式状态

### ref 与 reactive 必须按场景使用

- 基础类型和可替换对象优先使用 `ref`。

- 需要整体响应式访问的对象可使用 `reactive`。

- 不要在 Vue 3.4 及以下版本直接解构 props 后继续把解构变量当作响应式数据使用；需要解构时使用 `toRefs`。

- Vue 3.5 及以上版本支持 `<script setup>` 中的响应式 props 解构；使用前必须确认项目 Vue 版本和编译器配置支持该能力。

- 计算值使用 `computed`，不要用 watch 手工同步可推导状态。

错误示例：

```ts
const fullName = ref('');

watch([firstName, lastName], () => {
  fullName.value = `${firstName.value} ${lastName.value}`;
});
```

正确示例：

```ts
const fullName = computed(() => `${firstName.value} ${lastName.value}`);
```

### watch 必须谨慎使用

- `watch` 只用于处理副作用，不用于替代计算属性。

- `watch` 必须有明确监听源。

- 异步 `watch` 回调必须处理竞态、取消或过期结果。

- 组件卸载后仍可能执行的订阅、定时器、事件监听必须清理。

## 5. 组合式函数

### 可复用逻辑必须抽成 composable

- 组合式函数文件名使用 `useXxx.ts`。

- 组合式函数必须只暴露调用方需要的状态和方法。

- 组合式函数不得隐式依赖具体页面组件。

- 组合式函数如创建事件监听、订阅、定时器，必须在作用域释放时清理。

示例：

```ts
export function useOnlineStatus() {
  const isOnline = ref(navigator.onLine);

  function updateOnline() {
    isOnline.value = navigator.onLine;
  }

  window.addEventListener('online', updateOnline);
  window.addEventListener('offline', updateOnline);

  onScopeDispose(() => {
    window.removeEventListener('online', updateOnline);
    window.removeEventListener('offline', updateOnline);
  });

  return { isOnline };
}
```

- 组合式函数中不应直接操作全局 store，除非该函数就是为 store 集成设计。

## 6. 组件拆分

### 页面组件不得过大

- 页面组件负责装配数据、调用服务、组织子组件。

- 展示组件只负责渲染 props 和发出事件。

- 表单、表格、筛选区、弹窗等复杂区域应拆成子组件。

- 单个组件一般不超过 300 行。超过时必须检查是否可以拆分。

- 不要为了复用一次性逻辑而过早抽象组件。

### 组件通信必须简单

- 父子组件使用 props 和 emits。

- 跨多层共享状态使用 Pinia 或 provide/inject。

- provide/inject 必须使用 typed key，禁止使用裸字符串作为复杂依赖 key。

示例：

```ts
export const formContextKey: InjectionKey<FormContext> = Symbol('formContext');
```

## 7. Pinia

### store 必须按业务域或状态域拆分

- 不要创建一个全局巨型 store。

- store state 必须可序列化，除非项目明确允许保存运行时对象。

- store 中的 action 必须表达业务动作，不要只做简单 setter 堆叠。

- getter 必须保持纯计算，不得产生副作用。

示例：

```ts
export const useUserStore = defineStore('user', () => {
  const profile = ref<UserProfile | null>(null);

  const isLoggedIn = computed(() => profile.value != null);

  function setProfile(nextProfile: UserProfile | null) {
    profile.value = nextProfile;
  }

  return {
    profile,
    isLoggedIn,
    setProfile,
  };
});
```

- store 之间依赖必须保持方向清晰，避免循环调用。

- 组件中不要复制 store 状态到本地 ref，除非需要编辑草稿或临时状态。

## 8. Vue Router

### 路由配置必须类型清晰

- 路由记录必须集中定义，避免在多个文件中隐式拼接。

- 路由名称必须稳定，适合在代码中引用。

- route meta 必须声明类型。

示例：

```ts
declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean;
  }
}
```

- 路由守卫必须保持短小，复杂判断应抽成命名函数。

- 路由守卫不得直接操作 DOM。

- 页面跳转必须处理重复导航和异步失败。

## 9. 组件库使用

### 组件库封装必须一致

- 优先使用项目选定的组件库，不要混用多个 UI 组件库实现同一类控件。

- 表单组件必须声明校验规则和触发时机。

- 弹窗、抽屉、通知、消息提示等全局反馈必须通过统一封装或项目约定调用。

- 表格列定义、分页、筛选和排序逻辑应保持一致。

- 不要通过深层选择器大面积覆盖组件库内部样式；需要定制时优先使用组件库暴露的属性、插槽和主题变量。

## 10. 样式

### 样式必须可维护

- 组件私有样式使用 `scoped`、CSS Modules 或项目约定方案。

- class 命名必须表达结构和语义。

- 不要在模板中大量写内联样式。

- 不要依赖脆弱的 DOM 层级选择器。

- 响应式布局应使用清晰的 CSS 规则，不要通过 JavaScript 手工计算普通布局。

## 11. 性能

### 避免无意义渲染和计算

- 大列表必须使用稳定 key。

- 昂贵计算必须使用 `computed` 或缓存策略。

- 频繁触发的输入、滚动、窗口变化等事件应使用节流或防抖。

- 不要在模板中创建新的对象、数组或函数作为频繁变化的 props。

- 组件卸载时必须清理事件监听、定时器和订阅。

## 12. 测试代码

### 组件测试必须关注行为

- 组件测试应覆盖渲染状态、用户交互、props 变化、emit 事件和错误状态。

- 不要测试 Vue 内部实现细节。

- 选择元素时优先使用可访问语义、文本、label 或稳定测试属性。

- Pinia store 必须覆盖 state 初始化、getter 和 action。

- 路由相关逻辑必须覆盖守卫分支和跳转结果。

示例：

```ts
it('emits submit when the form is valid', async () => {
  const wrapper = mount(UserForm);

  await wrapper.get('form').trigger('submit');

  expect(wrapper.emitted('submit')).toHaveLength(1);
});
```

## 13. 禁止写法

- 禁止新增 Vue 2 写法。

- 禁止在同一组件中混用 Options API 和 Composition API，除非维护历史代码。

- 禁止直接修改 props。

- 禁止模板中出现复杂业务表达式。

- 禁止 `v-for` 缺少稳定 key。

- 禁止使用 watch 替代 computed。

- 禁止组件隐式依赖父组件内部实现。

- 禁止在组件卸载后遗留事件监听、定时器或订阅。

- 禁止把页面组件写成包含所有逻辑的巨型组件。
