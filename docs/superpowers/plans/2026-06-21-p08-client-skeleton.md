# P08 管理端工程骨架与通信层 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 交付完整的 `client/` Electron + Vue 3 工程骨架，冻结 F5 通信层契约（TLS 客户端、帧编解码、请求分发、心跳、连接状态机、IPC 桥接、Pinia stores、路由守卫、Services 层、公共组件、主框架布局）。

**Architecture:** electron-vite 统一驱动 main/preload/renderer 三入口。主进程负责 TLS 通信（tls/ 模块分层），preload 暴露最小 `desktopApi` 桥接，renderer 使用 Vue 3 + Pinia + Vue Router + Element Plus。所有协议交互通过 services/ 封装，组件不直接调 IPC。Protobuf 使用 protobufjs + pbts 静态生成。

**Tech Stack:** Electron 32.x, Vue 3.4.x, Vite 5.x, TypeScript 5.x, Element Plus 2.8.x, Pinia 2.x, Vue Router 4.x, protobufjs 7.x, electron-vite, Vitest, electron-builder

## Global Constraints

- **零持久化：** 禁止 localStorage / IndexedDB / 文件落盘业务数据，token/username/role 仅进程内存保存
- **TLS 在主进程：** renderer 通过 IPC 调用，不直接接触 Node.js tls 模块
- **Services 封装：** 所有协议交互通过 `renderer/services/` 封装，组件不直接调 IPC
- **安全默认：** `nodeIntegration: false`, `contextIsolation: true`, `sandbox: true`
- **preload 最小暴露：** 不暴露 `ipcRenderer`/`require`/`process`，只暴露 `desktopApi` 受控接口
- **结果码映射：** 已知 result_code 按统一结果码表映射中文文案；UNAUTHENTICATED(0x0001) 无弹窗静默清会话跳登录页；未知码展示 error_message 原文
- **心跳不重置无操作计时器**
- **断连保留当前页面，不跳登录页；** 主动登出才跳登录页
- **协议帧：** 20 字节固定头（Magic 0x55534243 + MsgType + SeqID + PayloadLen + CRC32），Big Endian
- **ESLint + Prettier + TypeScript strict 全开**
- **Windows 安装包不依赖目标机器预装 Node.js**
- **proto/ 指向项目根 proto/usb_control.proto**（Windows 用拷贝替代符号链接）

## File Structure

```
client/
├── package.json
├── electron-builder.yml
├── electron.vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── tsconfig.web.json
├── .eslintrc.cjs
├── .prettierrc
├── proto/usb_control.proto          # 从项目根拷贝
├── scripts/
│   └── gen-proto.ts
├── src/
│   ├── main/
│   │   ├── index.ts                 # app 生命周期
│   │   ├── window.ts                # BrowserWindow 创建
│   │   ├── tls-client.ts            # TLS 门面
│   │   ├── tls/
│   │   │   ├── tls-transport.ts
│   │   │   ├── frame-codec.ts
│   │   │   ├── request-dispatcher.ts
│   │   │   ├── heartbeat.ts
│   │   │   └── connection-state.ts
│   │   └── ipc/
│   │       └── tls-ipc.ts
│   ├── preload/
│   │   └── index.ts
│   ├── renderer/
│   │   ├── App.vue
│   │   ├── main.ts
│   │   ├── router/
│   │   │   ├── index.ts
│   │   │   └── routes.ts
│   │   ├── stores/
│   │   │   ├── session.ts
│   │   │   ├── connection.ts
│   │   │   └── ui.ts
│   │   ├── services/
│   │   │   ├── send-command.ts       # 通用发送封装
│   │   │   ├── auth-service.ts
│   │   │   ├── device-service.ts
│   │   │   ├── whitelist-service.ts
│   │   │   ├── file-policy-service.ts
│   │   │   ├── log-service.ts
│   │   │   ├── system-service.ts
│   │   │   ├── user-service.ts
│   │   │   └── policy-service.ts
│   │   ├── components/
│   │   │   ├── DataTable.vue
│   │   │   ├── ConfirmDialog.vue
│   │   │   ├── ProgressDialog.vue
│   │   │   ├── ConnectionAlert.vue
│   │   │   └── ChangePasswordDialog.vue
│   │   ├── layouts/
│   │   │   └── MainLayout.vue
│   │   ├── pages/
│   │   │   ├── LoginPage.vue
│   │   │   ├── LicensePage.vue
│   │   │   ├── FileAccessPage.vue
│   │   │   ├── UsbDevicesPage.vue
│   │   │   ├── PoliciesPage.vue
│   │   │   ├── LogsPage.vue
│   │   │   ├── SystemPage.vue
│   │   │   └── UsersPage.vue
│   │   ├── styles/
│   │   │   └── variables.scss
│   │   └── utils/
│   │       ├── result-code.ts
│   │       └── password-validator.ts
│   └── shared/
│       ├── ipc-channels.ts
│       ├── connection-state.ts
│       ├── result-codes.ts
│       └── proto/                    # gen-proto 输出
│           ├── usb_control.js
│           └── usb_control.d.ts
└── tests/
    └── unit/
        ├── tls/
        │   ├── frame-codec.test.ts
        │   ├── connection-state.test.ts
        │   ├── request-dispatcher.test.ts
        │   └── heartbeat.test.ts
        └── stores/
            ├── session.test.ts
            ├── connection.test.ts
            └── ui.test.ts
```

---

### Task 1: 工程脚手架与构建配置（Part A）

**Files:**
- Create: `client/package.json`
- Create: `client/electron.vite.config.ts`
- Create: `client/electron-builder.yml`
- Create: `client/tsconfig.json`
- Create: `client/tsconfig.node.json`
- Create: `client/tsconfig.web.json`
- Create: `client/.eslintrc.cjs`
- Create: `client/.prettierrc`
- Create: `client/src/main/index.ts` (最小启动)
- Create: `client/src/preload/index.ts` (空壳)
- Create: `client/src/renderer/main.ts` (Vue 入口)
- Create: `client/src/renderer/App.vue` (最小组件)
- Create: `client/src/renderer/index.html`

**Interfaces:**
- Produces: 可运行的 electron-vite 开发环境，`npm run dev` 启动后展示空白窗口

- [ ] **Step 1: 初始化 package.json**

在 `client/` 目录创建 `package.json`：

```json
{
  "name": "usb-control-client",
  "version": "0.1.0",
  "private": true,
  "description": "USB安全管理系统管理端",
  "main": "./out/main/index.js",
  "scripts": {
    "dev": "electron-vite dev",
    "build": "electron-vite build",
    "preview": "electron-vite preview",
    "gen-proto": "ts-node scripts/gen-proto.ts",
    "prebuild": "npm run gen-proto",
    "lint": "eslint . --ext .ts,.tsx,.vue",
    "format": "prettier --write \"src/**/*.{ts,tsx,vue,scss}\"",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "dependencies": {
    "vue": "^3.4.0",
    "vue-router": "^4.0.0",
    "pinia": "^2.0.0",
    "element-plus": "^2.8.0",
    "protobufjs": "^7.0.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "electron": "^32.0.0",
    "electron-vite": "^2.0.0",
    "electron-builder": "^24.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "vitest": "^2.0.0",
    "sass": "^1.70.0",
    "@types/node": "^20.0.0",
    "eslint": "^8.0.0",
    "@typescript-eslint/eslint-plugin": "^7.0.0",
    "@typescript-eslint/parser": "^7.0.0",
    "eslint-plugin-vue": "^9.0.0",
    "prettier": "^3.0.0",
    "ts-node": "^10.0.0",
    "protobufjs-cli": "^1.1.0",
    "@vue/test-utils": "^2.0.0",
    "happy-dom": "^14.0.0",
    "unplugin-auto-import": "^0.18.0",
    "unplugin-vue-components": "^0.27.0"
  }
}
```

- [ ] **Step 2: 创建 electron.vite.config.ts**

```typescript
import { resolve } from 'path'
import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        external: ['protobufjs']
      }
    }
  },
  preload: {
    plugins: [externalizeDepsPlugin()]
  },
  renderer: {
    resolve: {
      alias: {
        '@': resolve('src/renderer')
      }
    },
    plugins: [
      vue(),
      AutoImport({
        resolvers: [ElementPlusResolver()]
      }),
      Components({
        resolvers: [ElementPlusResolver()]
      })
    ],
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@use "@/styles/variables" as *;`
        }
      }
    }
  }
})
```

- [ ] **Step 3: 创建 tsconfig 三件套**

`client/tsconfig.json`：
```json
{
  "files": [],
  "references": [
    { "path": "./tsconfig.node.json" },
    { "path": "./tsconfig.web.json" }
  ]
}
```

`client/tsconfig.node.json`：
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "composite": true,
    "outDir": "./out",
    "declaration": true,
    "declarationMap": true,
    "lib": ["ES2022"]
  },
  "include": [
    "src/main/**/*.ts",
    "src/preload/**/*.ts",
    "src/shared/**/*.ts",
    "scripts/**/*.ts",
    "electron.vite.config.ts"
  ]
}
```

`client/tsconfig.web.json`：
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "composite": true,
    "outDir": "./out",
    "declaration": true,
    "declarationMap": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "paths": {
      "@/*": ["./src/renderer/*"]
    }
  },
  "include": [
    "src/renderer/**/*.ts",
    "src/renderer/**/*.vue",
    "src/shared/**/*.ts"
  ]
}
```

- [ ] **Step 4: 创建 .eslintrc.cjs 和 .prettierrc**

`client/.eslintrc.cjs`：
```javascript
module.exports = {
  root: true,
  env: {
    node: true,
    browser: true
  },
  parser: 'vue-eslint-parser',
  parserOptions: {
    parser: '@typescript-eslint/parser',
    ecmaVersion: 'latest',
    sourceType: 'module'
  },
  plugins: ['@typescript-eslint'],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:vue/vue3-recommended'
  ],
  rules: {
    'vue/multi-word-component-names': 'off',
    '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }]
  }
}
```

`client/.prettierrc`：
```json
{
  "semi": false,
  "singleQuote": true,
  "printWidth": 100,
  "trailingComma": "all"
}
```

- [ ] **Step 5: 创建 electron-builder.yml**

```yaml
appId: com.andi.usb-control
productName: USB安全管理系统
win:
  target:
    - target: nsis
      arch: [x64]
  icon: resources/icon.ico
nsis:
  oneClick: false
  allowToChangeInstallationDirectory: true
  installerLanguages:
    - zh_CN
directories:
  output: dist
  buildResources: resources
```

- [ ] **Step 6: 创建最小主进程入口 src/main/index.ts**

```typescript
import { app, BrowserWindow } from 'electron'
import { join } from 'path'
import { is } from '@electron-toolkit/utils'

let mainWindow: BrowserWindow | null = null

function createMainWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 1024,
    minHeight: 700,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
      preload: join(__dirname, '../preload/index.js'),
    },
  })

  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
  }

  mainWindow.on('closed', () => {
    mainWindow = null
  })
}

app.whenReady().then(() => {
  createMainWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createMainWindow()
    }
  })
})

app.on('window-all-closed', () => {
  app.quit()
})
```

注意：`@electron-toolkit/utils` 需加入 devDependencies。如果不想引入额外依赖，也可以用 `app.isPackaged` 替代 `is.dev`：

```typescript
const isDev = !app.isPackaged
```

按项目简洁原则，优先使用 `!app.isPackaged`，不引入 `@electron-toolkit/utils`。

- [ ] **Step 7: 创建最小 preload 和 renderer 入口**

`client/src/preload/index.ts`：
```typescript
// preload 桥接入口，后续 Task 补充 contextBridge 暴露
```

`client/src/renderer/index.html`：
```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <title>USB安全管理系统</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
```

`client/src/renderer/main.ts`：
```typescript
import { createApp } from 'vue'
import App from './App.vue'

const app = createApp(App)
app.mount('#app')
```

`client/src/renderer/App.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div>USB安全管理系统</div>
</template>
```

- [ ] **Step 8: 安装依赖并验证启动**

```bash
cd client
npm install
npm run dev
```

Expected: Electron 窗口启动，显示"USB安全管理系统"文字。

- [ ] **Step 9: 提交**

```bash
git add client/
git commit -m "feat(client): 初始化 Electron + Vue 3 工程骨架

- 使用 electron-vite 统一驱动 main/preload/renderer 三入口
- 配置 TypeScript strict 模式、ESLint、Prettier
- 配置 Element Plus 按需导入
- 配置 electron-builder NSIS 安装包
- 最小主进程入口，安全默认配置（nodeIntegration: false, contextIsolation: true, sandbox: true）
"
```

---

### Task 2: 共享类型与常量定义

**Files:**
- Create: `client/src/shared/ipc-channels.ts`
- Create: `client/src/shared/connection-state.ts`
- Create: `client/src/shared/result-codes.ts`
- Create: `client/src/renderer/utils/result-code.ts`

**Interfaces:**
- Produces:
  - `IpcChannels` 常量对象（`tls:connect`, `tls:disconnect`, `tls:send`, `connection:state-changed`）
  - `ConnectionStatus` 类型（8 状态联合类型）
  - `ConnectionEvent` 类型（14 事件联合类型）
  - `ResultCode` 常量对象（全量结果码数字常量）
  - `getResultMessage(resultCode: number, errorMessage: string): string | null` 函数

- [ ] **Step 1: 创建 IPC Channel 常量**

`client/src/shared/ipc-channels.ts`：
```typescript
export const IpcChannels = {
  tlsConnect: 'tls:connect',
  tlsDisconnect: 'tls:disconnect',
  tlsSend: 'tls:send',
  connectionStateChanged: 'connection:state-changed',
  dialogOpenFile: 'dialog:open-file',
  dialogSaveFile: 'dialog:save-file',
} as const
```

- [ ] **Step 2: 创建连接状态类型**

`client/src/shared/connection-state.ts`：
```typescript
export type ConnectionStatus =
  | 'DISCONNECTED'
  | 'CONNECTING'
  | 'AUTHENTICATING'
  | 'CHECK_LICENSE'
  | 'AUTH_REQUIRED'
  | 'LICENSE_EXPIRED'
  | 'LOADING_CONFIG'
  | 'CONNECTED'

export type ConnectionEvent =
  | 'CONNECT_START'
  | 'CONNECT_SUCCESS'
  | 'CONNECT_FAIL'
  | 'AUTH_SUCCESS'
  | 'AUTH_FAIL'
  | 'LICENSE_AUTHORIZED'
  | 'LICENSE_UNAUTHORIZED'
  | 'LICENSE_EXPIRED'
  | 'CONFIG_LOADED'
  | 'CONFIG_FAILED'
  | 'HEARTBEAT_TIMEOUT'
  | 'NETWORK_ERROR'
  | 'LOGOUT'
  | 'LICENSE_UPLOAD_SUCCESS'

export type UserRole = 'admin' | 'operator' | 'auditor'

export type AuthStatus = 'authorized' | 'unauthorized' | 'expired' | 'failed' | ''
```

- [ ] **Step 3: 创建统一结果码常量**

`client/src/shared/result-codes.ts`：
```typescript
export const ResultCode = {
  SUCCESS: 0x0000,
  // 全局 0x00xx
  UNAUTHENTICATED: 0x0001,
  UNAUTHORIZED: 0x0002,
  PERMISSION_DENIED: 0x0003,
  ACCOUNT_LOCKED: 0x0004,
  VALIDATION_FAILED: 0x0005,
  DEVICE_BUSY: 0x0006,
  INTERNAL_ERROR: 0x0007,
  // 登录鉴权 0x01xx
  USER_OR_PASSWORD_ERROR: 0x0101,
  DEVICE_UNAUTHORIZED: 0x0102,
  LICENSE_FORMAT_ERROR: 0x0103,
  LICENSE_VERIFY_FAILED: 0x0104,
  LICENSE_EXPIRED: 0x0105,
  // 白名单 0x02xx
  SERIAL_NUMBER_EMPTY: 0x0201,
  ALREADY_EXISTS: 0x0202,
  NOT_FOUND: 0x0203,
  DEVICE_NOT_STORAGE: 0x0204,
  DEVICE_SPOOF_SUSPECTED: 0x0205,
  DEVICE_UNSUPPORTED: 0x0206,
  // 文件策略 0x03xx
  POLICY_KEY_INVALID: 0x0301,
  EXTENSION_FORMAT_ERROR: 0x0302,
  EXTENSION_EXISTS: 0x0303,
  EXTENSION_NOT_FOUND: 0x0304,
  DEFAULT_EXTENSION_NO_DELETE: 0x0305,
  // 策略导入导出 0x04xx
  POLICY_EXPORT_FAILED: 0x0401,
  FORMAT_ERROR_POLICY: 0x0402,
  POLICY_SIGNATURE_ERROR: 0x0403,
  POLICY_DIGEST_ERROR: 0x0404,
  POLICY_DECRYPT_ERROR: 0x0405,
  VERSION_INCOMPATIBLE: 0x0406,
  POLICY_IMPORT_FAILED: 0x0407,
  // 日志 0x05xx
  LOG_RETENTION_VIOLATION: 0x0501,
  LOG_TYPE_INVALID: 0x0502,
  LOG_QUERY_FAILED: 0x0503,
  LOG_EXPORT_FAILED: 0x0504,
  // 系统管理 0x06xx
  VERSION_TOO_LOW: 0x0601,
  FORMAT_ERROR_UPGRADE: 0x0602,
  UPGRADE_CHECKSUM_ERROR: 0x0603,
  UPGRADE_APPLY_FAILED: 0x0604,
  VERSION_NUMBER_FORBIDDEN: 0x0605,
  VIRUSDB_INTEGRITY_ERROR: 0x0606,
  CLAMD_RELOAD_FAILED: 0x0607,
  VIRUSDB_APPLY_FAILED: 0x0608,
  DEVICE_DESC_FORMAT_ERROR: 0x0609,
  // 用户管理 0x07xx
  USERNAME_EXISTS: 0x0701,
  USERNAME_DELETED_REUSE: 0x0702,
  USER_NOT_FOUND: 0x0703,
  PASSWORD_COMPLEXITY_ERROR: 0x0704,
  PASSWORD_CONFIRM_MISMATCH: 0x0705,
  OLD_PASSWORD_ERROR: 0x0706,
  BUILTIN_USER_NO_DELETE: 0x0707,
  SELF_DELETE_FORBIDDEN: 0x0708,
} as const
```

- [ ] **Step 4: 创建结果码文案映射**

`client/src/renderer/utils/result-code.ts`：
```typescript
import { ResultCode } from '../../shared/result-codes'

const RESULT_CODE_MESSAGES: Record<number, string | null> = {
  [ResultCode.UNAUTHENTICATED]: null,
  [ResultCode.UNAUTHORIZED]: '装置未授权，请先完成授权',
  [ResultCode.PERMISSION_DENIED]: '当前用户无权执行此操作',
  [ResultCode.ACCOUNT_LOCKED]: '用户已被锁定，请5分钟后重试',
  [ResultCode.VALIDATION_FAILED]: null,
  [ResultCode.DEVICE_BUSY]: '装置正在处理其他操作，请稍后重试',
  [ResultCode.INTERNAL_ERROR]: '装置内部错误，请联系管理员',
  [ResultCode.USER_OR_PASSWORD_ERROR]: '用户名或密码错误',
  [ResultCode.DEVICE_UNAUTHORIZED]: null,
  [ResultCode.LICENSE_FORMAT_ERROR]: '授权文件格式错误，请检查文件',
  [ResultCode.LICENSE_VERIFY_FAILED]: '授权文件校验失败，请确认文件与本装置匹配',
  [ResultCode.LICENSE_EXPIRED]: '授权文件已过有效期，请重新获取授权文件',
  [ResultCode.SERIAL_NUMBER_EMPTY]: '序列号不能为空',
  [ResultCode.ALREADY_EXISTS]: '该设备已在白名单中',
  [ResultCode.NOT_FOUND]: '目标设备不存在',
  [ResultCode.DEVICE_NOT_STORAGE]: '仅支持添加大容量存储设备',
  [ResultCode.DEVICE_SPOOF_SUSPECTED]: '设备描述符异常，疑似伪装设备，禁止添加',
  [ResultCode.DEVICE_UNSUPPORTED]: '不支持的USB设备类型，无法添加',
  [ResultCode.POLICY_KEY_INVALID]: '策略开关标识无效',
  [ResultCode.EXTENSION_FORMAT_ERROR]: '文件后缀格式错误',
  [ResultCode.EXTENSION_EXISTS]: '该文件后缀已在黑名单中',
  [ResultCode.EXTENSION_NOT_FOUND]: '该文件后缀不在黑名单中',
  [ResultCode.DEFAULT_EXTENSION_NO_DELETE]: '内置默认后缀不可删除',
  [ResultCode.POLICY_EXPORT_FAILED]: '策略导出失败，请重试',
  [ResultCode.FORMAT_ERROR_POLICY]: '策略文件格式错误',
  [ResultCode.POLICY_SIGNATURE_ERROR]: '策略文件签名校验失败',
  [ResultCode.POLICY_DIGEST_ERROR]: '策略文件完整性校验失败',
  [ResultCode.POLICY_DECRYPT_ERROR]: '策略文件解密失败',
  [ResultCode.VERSION_INCOMPATIBLE]: '策略文件版本不兼容，请检查文件',
  [ResultCode.POLICY_IMPORT_FAILED]: '策略导入失败，原策略未变更',
  [ResultCode.LOG_RETENTION_VIOLATION]: '半年内的日志不可清理',
  [ResultCode.LOG_TYPE_INVALID]: null,
  [ResultCode.LOG_QUERY_FAILED]: '日志查询失败，请重试',
  [ResultCode.LOG_EXPORT_FAILED]: '日志导出失败，请重试',
  [ResultCode.VERSION_TOO_LOW]: '升级包版本低于当前版本，不允许降版本',
  [ResultCode.FORMAT_ERROR_UPGRADE]: '升级包格式错误',
  [ResultCode.UPGRADE_CHECKSUM_ERROR]: '升级包完整性校验失败',
  [ResultCode.UPGRADE_APPLY_FAILED]: '系统升级失败，已回滚至原版本',
  [ResultCode.VERSION_NUMBER_FORBIDDEN]: '病毒库版本号不合法',
  [ResultCode.VIRUSDB_INTEGRITY_ERROR]: '病毒库文件完整性校验失败',
  [ResultCode.CLAMD_RELOAD_FAILED]: '病毒库更新成功但加载失败，已回滚至旧版本',
  [ResultCode.VIRUSDB_APPLY_FAILED]: '病毒库升级失败，原病毒库未变更',
  [ResultCode.DEVICE_DESC_FORMAT_ERROR]: '设备描述仅允许字母、数字、下划线，最多32位',
  [ResultCode.USERNAME_EXISTS]: '用户名已存在',
  [ResultCode.USERNAME_DELETED_REUSE]: '该用户名已被使用过，不可重复创建',
  [ResultCode.USER_NOT_FOUND]: '目标用户不存在',
  [ResultCode.PASSWORD_COMPLEXITY_ERROR]: null,
  [ResultCode.PASSWORD_CONFIRM_MISMATCH]: '两次输入的密码不一致',
  [ResultCode.OLD_PASSWORD_ERROR]: '旧密码错误',
  [ResultCode.BUILTIN_USER_NO_DELETE]: '内置用户不可删除',
  [ResultCode.SELF_DELETE_FORBIDDEN]: '不允许删除当前登录用户',
}

export function getResultMessage(resultCode: number, errorMessage: string): string | null {
  if (resultCode === ResultCode.SUCCESS) {
    return null
  }

  if (resultCode === ResultCode.UNAUTHENTICATED) {
    return null
  }

  if (resultCode === ResultCode.DEVICE_UNAUTHORIZED) {
    return null
  }

  if (resultCode in RESULT_CODE_MESSAGES) {
    const mapped = RESULT_CODE_MESSAGES[resultCode]
    if (mapped === null) {
      return errorMessage || '操作失败'
    }
    return mapped
  }

  return errorMessage || '未知错误'
}
```

- [ ] **Step 5: 提交**

```bash
git add client/src/shared/ client/src/renderer/utils/
git commit -m "feat(client): 添加共享类型定义与统一结果码映射

- 定义 IPC Channel 常量（tls:connect/disconnect/send, connection:state-changed, dialog）
- 定义 ConnectionStatus 8 状态联合类型和 ConnectionEvent 14 事件联合类型
- 定义 UserRole、AuthStatus 类型
- 定义全量 ResultCode 常量（全局 + 7 个业务模块共 46 个结果码）
- 实现 getResultMessage 文案映射函数，UNAUTHENTICATED 返回 null 表示静默清会话
"
```

---

### Task 3: Protobuf 生成管线（Part B）

**Files:**
- Create: `client/proto/usb_control.proto` (从项目根拷贝)
- Create: `client/scripts/gen-proto.ts`
- Create: `client/src/shared/proto/` (生成输出目录)

**Interfaces:**
- Consumes: `proto/usb_control.proto`（项目根）
- Produces: `client/src/shared/proto/usb_control.js` 和 `usb_control.d.ts`（protobufjs 静态模块，ES6 格式），供 services/ 和 tls/ 层 import 使用

- [ ] **Step 1: 拷贝 proto 文件到 client/proto/**

```bash
cd client
mkdir -p proto
cp ../proto/usb_control.proto proto/usb_control.proto
```

Windows 环境使用拷贝替代符号链接，后续 proto 更新时需手动同步。

- [ ] **Step 2: 创建生成脚本 scripts/gen-proto.ts**

`client/scripts/gen-proto.ts`：
```typescript
import { execSync } from 'child_process'
import { resolve } from 'path'
import { mkdirSync } from 'fs'

const root = resolve(__dirname, '..')
const protoFile = resolve(root, 'proto/usb_control.proto')
const outDir = resolve(root, 'src/shared/proto')
const outJs = resolve(outDir, 'usb_control.js')
const outDts = resolve(outDir, 'usb_control.d.ts')

mkdirSync(outDir, { recursive: true })

execSync(
  `npx pbjs -t static-module -w es6 --no-create --no-verify -o "${outJs}" "${protoFile}"`,
  { stdio: 'inherit', cwd: root },
)

execSync(`npx pbts -o "${outDts}" "${outJs}"`, {
  stdio: 'inherit',
  cwd: root,
})

console.log('Proto generation complete.')
```

- [ ] **Step 3: 运行生成脚本并验证输出**

```bash
cd client
npm run gen-proto
```

Expected: `src/shared/proto/usb_control.js` 和 `usb_control.d.ts` 生成成功。验证 `usb_control.d.ts` 中包含 `CmdLogin`、`RspLogin`、`RspCommon`、`CmdHeartbeat` 等类型定义。

- [ ] **Step 4: 验证 TypeScript 可以 import 生成的类型**

在 `src/main/index.ts` 顶部临时添加：
```typescript
import { usb_control } from '../shared/proto/usb_control'
console.log(usb_control.CmdLogin)
```
运行 `npm run dev` 确认无编译错误后删除临时代码。

- [ ] **Step 5: 提交**

```bash
git add client/proto/ client/scripts/ client/src/shared/proto/
git commit -m "feat(client): 添加 Protobuf 生成管线

- 从项目根拷贝 usb_control.proto 到 client/proto/
- 实现 gen-proto.ts 脚本调用 pbjs + pbts 生成静态模块
- 生成 ES6 格式 usb_control.js 和 usb_control.d.ts
- prebuild 钩子确保每次构建前重新生成
"
```

---

### Task 4: 帧编解码与连接状态机（Part C 基础层）

**Files:**
- Create: `client/src/main/tls/frame-codec.ts`
- Create: `client/src/main/tls/connection-state.ts`
- Test: `client/tests/unit/tls/frame-codec.test.ts`
- Test: `client/tests/unit/tls/connection-state.test.ts`

**Interfaces:**
- Produces:
  - `FRAME_MAGIC = 0x55534243`, `FRAME_HEADER_SIZE = 20`, `MAX_PAYLOAD_SIZE = 128 * 1024 * 1024`
  - `FrameHeader { magic, msgType, seqId, payloadLen, crc32 }`
  - `encodeFrame(msgType: number, seqId: number, payload: Uint8Array): Buffer`
  - `decodeFrameHeader(buffer: Buffer): FrameHeader | null`（需至少 20 字节）
  - `calculateCrc32(data: Uint8Array): number`（IEEE 802.3, 多项式 0xEDB88320）
  - `FrameStreamParser` 类：`feed(chunk: Buffer)` + `onFrame` 回调
  - `ConnectionStateMachine` 类：`current`, `transition(event)`, `onStateChange` 回调

- [ ] **Step 1: 编写 frame-codec 测试**

`client/tests/unit/tls/frame-codec.test.ts`：
```typescript
import { describe, it, expect, vi } from 'vitest'
import {
  FRAME_MAGIC,
  FRAME_HEADER_SIZE,
  MAX_PAYLOAD_SIZE,
  encodeFrame,
  decodeFrameHeader,
  calculateCrc32,
  FrameStreamParser,
} from '../../../src/main/tls/frame-codec'

describe('calculateCrc32', () => {
  it('returns 0x00000000 for empty data', () => {
    expect(calculateCrc32(new Uint8Array(0))).toBe(0x00000000)
  })

  it('computes correct CRC32 for known input', () => {
    const data = new TextEncoder().encode('123456789')
    expect(calculateCrc32(data)).toBe(0xcbf43926)
  })
})

describe('encodeFrame', () => {
  it('produces 20-byte header + payload', () => {
    const payload = new Uint8Array([0x01, 0x02, 0x03])
    const frame = encodeFrame(0x0001, 1, payload)
    expect(frame.length).toBe(FRAME_HEADER_SIZE + payload.length)
  })

  it('writes magic number at offset 0 as big-endian', () => {
    const frame = encodeFrame(0x0001, 1, new Uint8Array(0))
    expect(frame.readUInt32BE(0)).toBe(FRAME_MAGIC)
  })

  it('writes message type at offset 4', () => {
    const frame = encodeFrame(0x0102, 1, new Uint8Array(0))
    expect(frame.readUInt32BE(4)).toBe(0x0102)
  })

  it('writes sequence id at offset 8', () => {
    const frame = encodeFrame(0x0001, 42, new Uint8Array(0))
    expect(frame.readUInt32BE(8)).toBe(42)
  })

  it('writes payload length at offset 12', () => {
    const payload = new Uint8Array(100)
    const frame = encodeFrame(0x0001, 1, payload)
    expect(frame.readUInt32BE(12)).toBe(100)
  })

  it('writes CRC32 at offset 16 as 0 for empty payload', () => {
    const frame = encodeFrame(0x0001, 1, new Uint8Array(0))
    expect(frame.readUInt32BE(16)).toBe(0x00000000)
  })

  it('writes correct CRC32 for non-empty payload', () => {
    const payload = new TextEncoder().encode('123456789')
    const frame = encodeFrame(0x0001, 1, payload)
    expect(frame.readUInt32BE(16)).toBe(0xcbf43926)
  })
})

describe('decodeFrameHeader', () => {
  it('returns null for buffer shorter than 20 bytes', () => {
    expect(decodeFrameHeader(Buffer.alloc(19))).toBeNull()
  })

  it('decodes header from a valid frame', () => {
    const payload = new Uint8Array([0xaa, 0xbb])
    const frame = encodeFrame(0x0200, 7, payload)
    const header = decodeFrameHeader(frame)
    expect(header).not.toBeNull()
    expect(header!.magic).toBe(FRAME_MAGIC)
    expect(header!.msgType).toBe(0x0200)
    expect(header!.seqId).toBe(7)
    expect(header!.payloadLen).toBe(2)
  })
})

describe('FrameStreamParser', () => {
  it('parses a complete frame delivered in one chunk', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const payload = new Uint8Array([0x01, 0x02])
    const frame = encodeFrame(0x0001, 1, payload)
    parser.feed(frame)

    expect(onFrame).toHaveBeenCalledTimes(1)
    const [header, parsedPayload] = onFrame.mock.calls[0]
    expect(header.msgType).toBe(0x0001)
    expect(header.seqId).toBe(1)
    expect(parsedPayload).toEqual(payload)
  })

  it('handles TCP fragmentation (split delivery)', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const payload = new Uint8Array([0x0a, 0x0b, 0x0c])
    const frame = encodeFrame(0x0002, 2, payload)

    parser.feed(frame.subarray(0, 10))
    expect(onFrame).not.toHaveBeenCalled()

    parser.feed(frame.subarray(10))
    expect(onFrame).toHaveBeenCalledTimes(1)
  })

  it('handles multiple frames in one chunk (TCP coalescing)', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const frame1 = encodeFrame(0x0001, 1, new Uint8Array([0x01]))
    const frame2 = encodeFrame(0x0002, 2, new Uint8Array([0x02]))
    const combined = Buffer.concat([frame1, frame2])

    parser.feed(combined)
    expect(onFrame).toHaveBeenCalledTimes(2)
  })

  it('discards frame with bad magic and continues', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const badFrame = Buffer.alloc(20)
    badFrame.writeUInt32BE(0xdeadbeef, 0)
    badFrame.writeUInt32BE(0, 12)

    const goodFrame = encodeFrame(0x0001, 1, new Uint8Array(0))
    parser.feed(Buffer.concat([badFrame, goodFrame]))

    expect(onFrame).toHaveBeenCalledTimes(1)
    expect(onFrame.mock.calls[0][0].msgType).toBe(0x0001)
  })

  it('discards frame with bad CRC32', () => {
    const parser = new FrameStreamParser()
    const onFrame = vi.fn()
    parser.onFrame = onFrame

    const payload = new Uint8Array([0x01, 0x02])
    const frame = encodeFrame(0x0001, 1, payload)
    frame.writeUInt32BE(0xffffffff, 16)

    parser.feed(frame)
    expect(onFrame).not.toHaveBeenCalled()
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cd client
npx vitest run tests/unit/tls/frame-codec.test.ts
```

Expected: FAIL — 模块不存在。

先创建 `client/vitest.config.ts`（如果 electron-vite 不自带）：
```typescript
import { defineConfig } from 'vitest/config'
import { resolve } from 'path'

export default defineConfig({
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src/renderer'),
    },
  },
})
```

- [ ] **Step 3: 实现 frame-codec.ts**

`client/src/main/tls/frame-codec.ts`：
```typescript
export const FRAME_MAGIC = 0x55534243
export const FRAME_HEADER_SIZE = 20
export const MAX_PAYLOAD_SIZE = 128 * 1024 * 1024

export interface FrameHeader {
  magic: number
  msgType: number
  seqId: number
  payloadLen: number
  crc32: number
}

const CRC32_TABLE = buildCrc32Table()

function buildCrc32Table(): Uint32Array {
  const table = new Uint32Array(256)
  for (let i = 0; i < 256; i++) {
    let crc = i
    for (let j = 0; j < 8; j++) {
      crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1
    }
    table[i] = crc >>> 0
  }
  return table
}

export function calculateCrc32(data: Uint8Array): number {
  if (data.length === 0) {
    return 0x00000000
  }
  let crc = 0xffffffff
  for (let i = 0; i < data.length; i++) {
    crc = (crc >>> 8) ^ CRC32_TABLE[(crc ^ data[i]) & 0xff]
  }
  return (crc ^ 0xffffffff) >>> 0
}

export function encodeFrame(msgType: number, seqId: number, payload: Uint8Array): Buffer {
  const frame = Buffer.alloc(FRAME_HEADER_SIZE + payload.length)
  frame.writeUInt32BE(FRAME_MAGIC, 0)
  frame.writeUInt32BE(msgType, 4)
  frame.writeUInt32BE(seqId, 8)
  frame.writeUInt32BE(payload.length, 12)
  frame.writeUInt32BE(calculateCrc32(payload), 16)
  if (payload.length > 0) {
    frame.set(payload, FRAME_HEADER_SIZE)
  }
  return frame
}

export function decodeFrameHeader(buffer: Buffer): FrameHeader | null {
  if (buffer.length < FRAME_HEADER_SIZE) {
    return null
  }
  return {
    magic: buffer.readUInt32BE(0),
    msgType: buffer.readUInt32BE(4),
    seqId: buffer.readUInt32BE(8),
    payloadLen: buffer.readUInt32BE(12),
    crc32: buffer.readUInt32BE(16),
  }
}

export class FrameStreamParser {
  private buffer: Buffer = Buffer.alloc(0)
  onFrame: ((header: FrameHeader, payload: Uint8Array) => void) | null = null

  feed(chunk: Buffer): void {
    this.buffer = Buffer.concat([this.buffer, chunk])
    this.tryParse()
  }

  private tryParse(): void {
    while (this.buffer.length >= FRAME_HEADER_SIZE) {
      const header = decodeFrameHeader(this.buffer)
      if (header == null) {
        break
      }

      if (header.magic !== FRAME_MAGIC) {
        this.buffer = this.buffer.subarray(4)
        continue
      }

      if (header.payloadLen > MAX_PAYLOAD_SIZE) {
        this.buffer = this.buffer.subarray(4)
        continue
      }

      const totalLen = FRAME_HEADER_SIZE + header.payloadLen
      if (this.buffer.length < totalLen) {
        break
      }

      const payload = new Uint8Array(this.buffer.subarray(FRAME_HEADER_SIZE, totalLen))

      const expectedCrc = calculateCrc32(payload)
      if (header.crc32 !== expectedCrc) {
        this.buffer = this.buffer.subarray(totalLen)
        continue
      }

      this.buffer = this.buffer.subarray(totalLen)

      if (this.onFrame != null) {
        this.onFrame(header, payload)
      }
    }
  }
}
```

- [ ] **Step 4: 运行 frame-codec 测试确认全部通过**

```bash
npx vitest run tests/unit/tls/frame-codec.test.ts
```

Expected: 全部 PASS。

- [ ] **Step 5: 编写 connection-state 测试**

`client/tests/unit/tls/connection-state.test.ts`：
```typescript
import { describe, it, expect, vi } from 'vitest'
import { ConnectionStateMachine } from '../../../src/main/tls/connection-state'

describe('ConnectionStateMachine', () => {
  it('starts in DISCONNECTED state', () => {
    const sm = new ConnectionStateMachine()
    expect(sm.current).toBe('DISCONNECTED')
  })

  it('transitions DISCONNECTED → CONNECTING on CONNECT_START', () => {
    const sm = new ConnectionStateMachine()
    expect(sm.transition('CONNECT_START')).toBe('CONNECTING')
  })

  it('transitions CONNECTING → AUTHENTICATING on CONNECT_SUCCESS', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    expect(sm.transition('CONNECT_SUCCESS')).toBe('AUTHENTICATING')
  })

  it('transitions CONNECTING → DISCONNECTED on CONNECT_FAIL', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    expect(sm.transition('CONNECT_FAIL')).toBe('DISCONNECTED')
  })

  it('transitions AUTHENTICATING → CHECK_LICENSE on AUTH_SUCCESS', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    expect(sm.transition('AUTH_SUCCESS')).toBe('CHECK_LICENSE')
  })

  it('stays AUTHENTICATING on AUTH_FAIL', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    expect(sm.transition('AUTH_FAIL')).toBe('AUTHENTICATING')
  })

  it('transitions CHECK_LICENSE → LOADING_CONFIG on LICENSE_AUTHORIZED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    expect(sm.transition('LICENSE_AUTHORIZED')).toBe('LOADING_CONFIG')
  })

  it('transitions CHECK_LICENSE → AUTH_REQUIRED on LICENSE_UNAUTHORIZED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    expect(sm.transition('LICENSE_UNAUTHORIZED')).toBe('AUTH_REQUIRED')
  })

  it('transitions CHECK_LICENSE → LICENSE_EXPIRED on LICENSE_EXPIRED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    expect(sm.transition('LICENSE_EXPIRED')).toBe('LICENSE_EXPIRED')
  })

  it('transitions AUTH_REQUIRED → DISCONNECTED on LICENSE_UPLOAD_SUCCESS', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_UNAUTHORIZED')
    expect(sm.transition('LICENSE_UPLOAD_SUCCESS')).toBe('DISCONNECTED')
  })

  it('transitions LOADING_CONFIG → CONNECTED on CONFIG_LOADED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    expect(sm.transition('CONFIG_LOADED')).toBe('CONNECTED')
  })

  it('transitions LOADING_CONFIG → DISCONNECTED on CONFIG_FAILED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    expect(sm.transition('CONFIG_FAILED')).toBe('DISCONNECTED')
  })

  it('transitions CONNECTED → DISCONNECTED on HEARTBEAT_TIMEOUT', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    sm.transition('CONFIG_LOADED')
    expect(sm.transition('HEARTBEAT_TIMEOUT')).toBe('DISCONNECTED')
  })

  it('transitions CONNECTED → DISCONNECTED on NETWORK_ERROR', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    sm.transition('CONFIG_LOADED')
    expect(sm.transition('NETWORK_ERROR')).toBe('DISCONNECTED')
  })

  it('transitions CONNECTED → DISCONNECTED on LOGOUT', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    sm.transition('AUTH_SUCCESS')
    sm.transition('LICENSE_AUTHORIZED')
    sm.transition('CONFIG_LOADED')
    expect(sm.transition('LOGOUT')).toBe('DISCONNECTED')
  })

  it('NETWORK_ERROR transitions any state to DISCONNECTED', () => {
    const sm = new ConnectionStateMachine()
    sm.transition('CONNECT_START')
    sm.transition('CONNECT_SUCCESS')
    expect(sm.transition('NETWORK_ERROR')).toBe('DISCONNECTED')
  })

  it('throws on invalid transition', () => {
    const sm = new ConnectionStateMachine()
    expect(() => sm.transition('AUTH_SUCCESS')).toThrow()
  })

  it('calls onStateChange callback on valid transition', () => {
    const sm = new ConnectionStateMachine()
    const callback = vi.fn()
    sm.onStateChange = callback

    sm.transition('CONNECT_START')
    expect(callback).toHaveBeenCalledWith('DISCONNECTED', 'CONNECTING', 'CONNECT_START')
  })
})
```

- [ ] **Step 6: 实现 connection-state.ts**

`client/src/main/tls/connection-state.ts`：
```typescript
import type { ConnectionStatus, ConnectionEvent } from '../../shared/connection-state'

type TransitionMap = Partial<Record<ConnectionEvent, ConnectionStatus>>
type StateTransitions = Record<ConnectionStatus, TransitionMap>

const TRANSITIONS: StateTransitions = {
  DISCONNECTED: {
    CONNECT_START: 'CONNECTING',
  },
  CONNECTING: {
    CONNECT_SUCCESS: 'AUTHENTICATING',
    CONNECT_FAIL: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  AUTHENTICATING: {
    AUTH_SUCCESS: 'CHECK_LICENSE',
    AUTH_FAIL: 'AUTHENTICATING',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  CHECK_LICENSE: {
    LICENSE_AUTHORIZED: 'LOADING_CONFIG',
    LICENSE_UNAUTHORIZED: 'AUTH_REQUIRED',
    LICENSE_EXPIRED: 'LICENSE_EXPIRED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  AUTH_REQUIRED: {
    LICENSE_UPLOAD_SUCCESS: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  LICENSE_EXPIRED: {
    NETWORK_ERROR: 'DISCONNECTED',
  },
  LOADING_CONFIG: {
    CONFIG_LOADED: 'CONNECTED',
    CONFIG_FAILED: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
  },
  CONNECTED: {
    HEARTBEAT_TIMEOUT: 'DISCONNECTED',
    NETWORK_ERROR: 'DISCONNECTED',
    LOGOUT: 'DISCONNECTED',
  },
}

export class ConnectionStateMachine {
  private state: ConnectionStatus = 'DISCONNECTED'
  onStateChange:
    | ((from: ConnectionStatus, to: ConnectionStatus, event: ConnectionEvent) => void)
    | null = null

  get current(): ConnectionStatus {
    return this.state
  }

  transition(event: ConnectionEvent): ConnectionStatus {
    const transitions = TRANSITIONS[this.state]
    const nextState = transitions[event]

    if (nextState == null) {
      throw new Error(
        `Invalid transition: state=${this.state}, event=${event}`,
      )
    }

    const prevState = this.state
    this.state = nextState

    if (this.onStateChange != null) {
      this.onStateChange(prevState, nextState, event)
    }

    return nextState
  }
}
```

- [ ] **Step 7: 运行 connection-state 测试确认全部通过**

```bash
npx vitest run tests/unit/tls/connection-state.test.ts
```

Expected: 全部 PASS。

- [ ] **Step 8: 提交**

```bash
git add client/src/main/tls/frame-codec.ts client/src/main/tls/connection-state.ts \
       client/tests/unit/tls/
git commit -m "feat(client): 实现帧编解码和连接状态机

- 实现 CRC32 IEEE 802.3 算法，空载荷返回 0x00000000
- 实现 encodeFrame/decodeFrameHeader 20 字节固定头编解码（Big Endian）
- 实现 FrameStreamParser 处理 TCP 粘包/拆包，Magic 不匹配和 CRC 失败丢弃帧
- 实现 ConnectionStateMachine 8 状态 14 事件转换表
- 非法转换抛出错误，NETWORK_ERROR 任何状态均回到 DISCONNECTED
- 单元测试覆盖编解码、粘包拆包、状态转换全路径
"
```

---

### Task 5: 请求分发器与心跳管理

**Files:**
- Create: `client/src/main/tls/request-dispatcher.ts`
- Create: `client/src/main/tls/heartbeat.ts`
- Test: `client/tests/unit/tls/request-dispatcher.test.ts`
- Test: `client/tests/unit/tls/heartbeat.test.ts`

**Interfaces:**
- Consumes: `encodeFrame()` from Task 4
- Produces:
  - `RequestDispatcher` 类：`dispatch(msgType, payload, timeout?): Promise<Uint8Array>`, `handleResponse(seqId, msgType, payload)`, `rejectAll(reason)`
  - `HeartbeatManager` 类：`start(sendFn)`, `stop()`, `onHeartbeatResponse()`, `onTimeout` 回调

- [ ] **Step 1: 编写 request-dispatcher 测试**

`client/tests/unit/tls/request-dispatcher.test.ts`：
```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { RequestDispatcher } from '../../../src/main/tls/request-dispatcher'

describe('RequestDispatcher', () => {
  let dispatcher: RequestDispatcher
  let writtenFrames: Buffer[]

  beforeEach(() => {
    vi.useFakeTimers()
    writtenFrames = []
    dispatcher = new RequestDispatcher((frame: Buffer) => {
      writtenFrames.push(frame)
    })
  })

  afterEach(() => {
    dispatcher.rejectAll(new Error('cleanup'))
    vi.useRealTimers()
  })

  it('assigns incrementing sequence IDs', async () => {
    const p1 = dispatcher.dispatch(0x0001, new Uint8Array(0))
    const p2 = dispatcher.dispatch(0x0002, new Uint8Array(0))

    dispatcher.handleResponse(1, 0x0002, new Uint8Array(0))
    dispatcher.handleResponse(2, 0x0003, new Uint8Array(0))

    await p1
    await p2
    expect(writtenFrames.length).toBe(2)
  })

  it('resolves when matching response arrives', async () => {
    const promise = dispatcher.dispatch(0x0001, new Uint8Array([0x01]))
    dispatcher.handleResponse(1, 0x0002, new Uint8Array([0x02]))
    const result = await promise
    expect(result).toEqual(new Uint8Array([0x02]))
  })

  it('rejects on timeout for non-retryable command', async () => {
    const promise = dispatcher.dispatch(0x0104, new Uint8Array(0), 1000)
    vi.advanceTimersByTime(1001)
    await expect(promise).rejects.toThrow('timeout')
  })

  it('retries once for read-only query command on timeout', async () => {
    const promise = dispatcher.dispatch(0x0100, new Uint8Array(0), 1000)

    vi.advanceTimersByTime(1001)
    expect(writtenFrames.length).toBe(2)

    dispatcher.handleResponse(2, 0x0101, new Uint8Array([0x0a]))
    const result = await promise
    expect(result).toEqual(new Uint8Array([0x0a]))
  })

  it('rejects after retry also times out', async () => {
    const promise = dispatcher.dispatch(0x0100, new Uint8Array(0), 1000)

    vi.advanceTimersByTime(1001)
    vi.advanceTimersByTime(1001)

    await expect(promise).rejects.toThrow('timeout')
  })

  it('rejectAll rejects all pending requests', async () => {
    const p1 = dispatcher.dispatch(0x0001, new Uint8Array(0))
    const p2 = dispatcher.dispatch(0x0002, new Uint8Array(0))

    dispatcher.rejectAll(new Error('disconnected'))

    await expect(p1).rejects.toThrow('disconnected')
    await expect(p2).rejects.toThrow('disconnected')
  })

  it('ignores late response after timeout', () => {
    dispatcher.dispatch(0x0104, new Uint8Array(0), 1000).catch(() => {})
    vi.advanceTimersByTime(1001)

    expect(() => {
      dispatcher.handleResponse(1, 0xff00, new Uint8Array(0))
    }).not.toThrow()
  })
})
```

- [ ] **Step 2: 实现 request-dispatcher.ts**

`client/src/main/tls/request-dispatcher.ts`：
```typescript
import { encodeFrame } from './frame-codec'

const RETRYABLE_COMMANDS = new Set([
  0x0100, 0x0102, 0x0200, 0x0400, 0x0500, 0x0600, 0x0003,
])

const FILE_TRANSFER_COMMANDS = new Set([
  0x0007, 0x0300, 0x0302, 0x0402, 0x0502, 0x0503,
])

const DEFAULT_TIMEOUT = 15_000
const FILE_TRANSFER_TIMEOUT = 300_000

interface PendingRequest {
  resolve: (payload: Uint8Array) => void
  reject: (error: Error) => void
  timer: ReturnType<typeof setTimeout>
  msgType: number
  payload: Uint8Array
  retried: boolean
}

export class RequestDispatcher {
  private nextSeqId = 1
  private pending = new Map<number, PendingRequest>()
  private writeFn: (frame: Buffer) => void

  constructor(writeFn: (frame: Buffer) => void) {
    this.writeFn = writeFn
  }

  dispatch(msgType: number, payload: Uint8Array, timeout?: number): Promise<Uint8Array> {
    const resolvedTimeout =
      timeout ?? (FILE_TRANSFER_COMMANDS.has(msgType) ? FILE_TRANSFER_TIMEOUT : DEFAULT_TIMEOUT)

    return new Promise<Uint8Array>((resolve, reject) => {
      const seqId = this.nextSeqId++
      this.sendAndTrack(seqId, msgType, payload, resolvedTimeout, resolve, reject, false)
    })
  }

  private sendAndTrack(
    seqId: number,
    msgType: number,
    payload: Uint8Array,
    timeout: number,
    resolve: (payload: Uint8Array) => void,
    reject: (error: Error) => void,
    retried: boolean,
  ): void {
    const timer = setTimeout(() => {
      this.pending.delete(seqId)

      if (!retried && RETRYABLE_COMMANDS.has(msgType)) {
        const retrySeqId = this.nextSeqId++
        this.sendAndTrack(retrySeqId, msgType, payload, timeout, resolve, reject, true)
        return
      }

      reject(new Error(`Request timeout: msgType=0x${msgType.toString(16)}, seqId=${seqId}`))
    }, timeout)

    this.pending.set(seqId, { resolve, reject, timer, msgType, payload, retried })

    const frame = encodeFrame(msgType, seqId, payload)
    this.writeFn(frame)
  }

  handleResponse(seqId: number, _msgType: number, payload: Uint8Array): void {
    const request = this.pending.get(seqId)
    if (request == null) {
      return
    }

    clearTimeout(request.timer)
    this.pending.delete(seqId)
    request.resolve(payload)
  }

  rejectAll(reason: Error): void {
    for (const [seqId, request] of this.pending) {
      clearTimeout(request.timer)
      request.reject(reason)
    }
    this.pending.clear()
  }
}
```

- [ ] **Step 3: 运行 request-dispatcher 测试**

```bash
npx vitest run tests/unit/tls/request-dispatcher.test.ts
```

Expected: 全部 PASS。

- [ ] **Step 4: 编写 heartbeat 测试**

`client/tests/unit/tls/heartbeat.test.ts`：
```typescript
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { HeartbeatManager } from '../../../src/main/tls/heartbeat'

describe('HeartbeatManager', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('calls sendFn every 30 seconds after start', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const hb = new HeartbeatManager()
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    expect(sendFn).toHaveBeenCalledTimes(1)

    vi.advanceTimersByTime(30_000)
    expect(sendFn).toHaveBeenCalledTimes(2)

    hb.stop()
  })

  it('resets miss counter on heartbeat response', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const onTimeout = vi.fn()
    const hb = new HeartbeatManager()
    hb.onTimeout = onTimeout
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)
    hb.onHeartbeatResponse()

    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)
    hb.onHeartbeatResponse()

    expect(onTimeout).not.toHaveBeenCalled()
    hb.stop()
  })

  it('triggers onTimeout after 3 consecutive misses', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const onTimeout = vi.fn()
    const hb = new HeartbeatManager()
    hb.onTimeout = onTimeout
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)

    expect(onTimeout).toHaveBeenCalledTimes(1)
    hb.stop()
  })

  it('stops sending after stop is called', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const hb = new HeartbeatManager()
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    expect(sendFn).toHaveBeenCalledTimes(1)

    hb.stop()
    vi.advanceTimersByTime(60_000)
    expect(sendFn).toHaveBeenCalledTimes(1)
  })
})
```

- [ ] **Step 5: 实现 heartbeat.ts**

`client/src/main/tls/heartbeat.ts`：
```typescript
const HEARTBEAT_INTERVAL = 30_000
const MAX_MISS_COUNT = 3

export class HeartbeatManager {
  private intervalId: ReturnType<typeof setInterval> | null = null
  private missCount = 0
  onTimeout: (() => void) | null = null

  start(sendFn: () => Promise<void>): void {
    this.stop()
    this.missCount = 0

    this.intervalId = setInterval(() => {
      this.missCount++

      if (this.missCount >= MAX_MISS_COUNT) {
        this.stop()
        if (this.onTimeout != null) {
          this.onTimeout()
        }
        return
      }

      sendFn().catch(() => {})
    }, HEARTBEAT_INTERVAL)
  }

  stop(): void {
    if (this.intervalId != null) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }
    this.missCount = 0
  }

  onHeartbeatResponse(): void {
    this.missCount = 0
  }
}
```

- [ ] **Step 6: 运行 heartbeat 测试**

```bash
npx vitest run tests/unit/tls/heartbeat.test.ts
```

Expected: 全部 PASS。

- [ ] **Step 7: 提交**

```bash
git add client/src/main/tls/request-dispatcher.ts client/src/main/tls/heartbeat.ts \
       client/tests/unit/tls/
git commit -m "feat(client): 实现请求分发器和心跳管理

- RequestDispatcher: SeqID 单调递增分配，请求/响应 SeqID 匹配
- 超时分档：普通指令 15s，文件传输 300s
- 7 个只读查询命令超时后自动重试 1 次（新 SeqID），写入类不重试
- 超时后 SeqID 废弃，迟到响应丢弃
- rejectAll() 释放所有 pending 请求
- HeartbeatManager: 30s 间隔发送心跳，连续 3 次无响应触发 onTimeout
- 收到响应重置计数器，stop() 清理定时器
"
```

---

### Task 6: TLS 传输层、门面与 IPC 桥接

**Files:**
- Create: `client/src/main/tls/tls-transport.ts`
- Create: `client/src/main/tls-client.ts`
- Create: `client/src/main/ipc/tls-ipc.ts`
- Create: `client/src/main/window.ts`
- Modify: `client/src/main/index.ts` (集成 TLS client + IPC)

**Interfaces:**
- Consumes: `FrameStreamParser`, `encodeFrame` (Task 4), `RequestDispatcher` (Task 5), `HeartbeatManager` (Task 5), `ConnectionStateMachine` (Task 4), `IpcChannels` (Task 2)
- Produces:
  - `TlsTransport` 类：`connect(host, port): Promise<void>`, `disconnect()`, `write(data: Buffer)`, `isConnected()`, events: `data` | `close` | `error`
  - `TlsClient` 类：`connect(host, port): Promise<void>`, `disconnect()`, `send(msgType, payload, timeout?): Promise<Uint8Array>`, `getConnectionStatus()`, events: `state-change`
  - `registerTlsIpc(tlsClient, getMainWindow)` 函数
  - `createMainWindow()` 函数（从 index.ts 提取）

- [ ] **Step 1: 创建 tls-transport.ts**

`client/src/main/tls/tls-transport.ts`：
```typescript
import * as tls from 'tls'
import { EventEmitter } from 'events'

const CONNECT_TIMEOUT = 15_000

export class TlsTransport extends EventEmitter {
  private socket: tls.TLSSocket | null = null

  async connect(host: string, port: number): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        if (this.socket != null) {
          this.socket.destroy()
          this.socket = null
        }
        reject(new Error('连接超时'))
      }, CONNECT_TIMEOUT)

      this.socket = tls.connect(
        {
          host,
          port,
          rejectUnauthorized: false,
        },
        () => {
          clearTimeout(timer)

          const cert = this.socket!.getPeerCertificate()
          const fingerprint = cert.fingerprint256?.replace(/:/g, '').toLowerCase()
          const expectedFingerprint = process.env['VITE_CERT_FINGERPRINT']

          if (
            expectedFingerprint != null &&
            expectedFingerprint !== '' &&
            fingerprint !== expectedFingerprint.toLowerCase()
          ) {
            this.socket!.destroy()
            this.socket = null
            reject(new Error('版本不兼容，请升级管理端'))
            return
          }

          resolve()
        },
      )

      this.socket.on('data', (chunk: Buffer) => {
        this.emit('data', chunk)
      })

      this.socket.on('close', () => {
        this.socket = null
        this.emit('close')
      })

      this.socket.on('error', (err: Error) => {
        clearTimeout(timer)
        this.socket = null
        this.emit('error', err)
        reject(err)
      })
    })
  }

  disconnect(): void {
    if (this.socket != null) {
      this.socket.destroy()
      this.socket = null
    }
  }

  write(data: Buffer): void {
    if (this.socket != null && !this.socket.destroyed) {
      this.socket.write(data)
    }
  }

  isConnected(): boolean {
    return this.socket != null && !this.socket.destroyed
  }
}
```

- [ ] **Step 2: 创建 tls-client.ts 门面**

`client/src/main/tls-client.ts`：
```typescript
import { EventEmitter } from 'events'
import type { ConnectionStatus } from '../shared/connection-state'
import { TlsTransport } from './tls/tls-transport'
import { FrameStreamParser } from './tls/frame-codec'
import { RequestDispatcher } from './tls/request-dispatcher'
import { HeartbeatManager } from './tls/heartbeat'
import { ConnectionStateMachine } from './tls/connection-state'
import { encodeFrame } from './tls/frame-codec'

const MSG_HEARTBEAT_CMD = 0xff01
const MSG_HEARTBEAT_RSP = 0xff02

export class TlsClient extends EventEmitter {
  private transport = new TlsTransport()
  private parser = new FrameStreamParser()
  private dispatcher: RequestDispatcher
  private heartbeat = new HeartbeatManager()
  private stateMachine = new ConnectionStateMachine()

  constructor() {
    super()

    this.dispatcher = new RequestDispatcher((frame: Buffer) => {
      this.transport.write(frame)
    })

    this.parser.onFrame = (header, payload) => {
      if (header.msgType === MSG_HEARTBEAT_RSP) {
        this.heartbeat.onHeartbeatResponse()
        return
      }
      this.dispatcher.handleResponse(header.seqId, header.msgType, payload)
    }

    this.transport.on('data', (chunk: Buffer) => {
      this.parser.feed(chunk)
    })

    this.transport.on('close', () => {
      this.handleDisconnect('NETWORK_ERROR')
    })

    this.transport.on('error', () => {
      this.handleDisconnect('NETWORK_ERROR')
    })

    this.heartbeat.onTimeout = () => {
      this.handleDisconnect('HEARTBEAT_TIMEOUT')
    }

    this.stateMachine.onStateChange = (from, to, event) => {
      this.emit('state-change', from, to, event)
    }
  }

  async connect(host: string, port: number): Promise<void> {
    this.stateMachine.transition('CONNECT_START')

    try {
      await this.transport.connect(host, port)
      this.stateMachine.transition('CONNECT_SUCCESS')
    } catch (err) {
      try {
        this.stateMachine.transition('CONNECT_FAIL')
      } catch {
        // 状态机可能已经被 NETWORK_ERROR 推到 DISCONNECTED
      }
      throw err
    }
  }

  disconnect(): void {
    this.heartbeat.stop()
    this.transport.disconnect()
    this.dispatcher.rejectAll(new Error('主动断开连接'))

    if (this.stateMachine.current !== 'DISCONNECTED') {
      try {
        this.stateMachine.transition('LOGOUT')
      } catch {
        // 某些状态下 LOGOUT 不合法，忽略
      }
    }
  }

  async send(msgType: number, payload: Uint8Array, timeout?: number): Promise<Uint8Array> {
    return this.dispatcher.dispatch(msgType, payload, timeout)
  }

  getConnectionStatus(): ConnectionStatus {
    return this.stateMachine.current
  }

  transitionState(event: import('../shared/connection-state').ConnectionEvent): void {
    this.stateMachine.transition(event)
  }

  startHeartbeat(): void {
    this.heartbeat.start(async () => {
      const frame = encodeFrame(MSG_HEARTBEAT_CMD, 0, new Uint8Array(0))
      this.transport.write(frame)
    })
  }

  stopHeartbeat(): void {
    this.heartbeat.stop()
  }

  private handleDisconnect(
    event: 'NETWORK_ERROR' | 'HEARTBEAT_TIMEOUT',
  ): void {
    this.heartbeat.stop()
    this.dispatcher.rejectAll(new Error(event === 'HEARTBEAT_TIMEOUT' ? '心跳超时' : '网络断开'))

    if (this.stateMachine.current !== 'DISCONNECTED') {
      try {
        this.stateMachine.transition(event)
      } catch {
        // 已经 DISCONNECTED
      }
    }
  }
}
```

- [ ] **Step 3: 创建 tls-ipc.ts**

`client/src/main/ipc/tls-ipc.ts`：
```typescript
import { ipcMain, type BrowserWindow } from 'electron'
import { IpcChannels } from '../../shared/ipc-channels'
import type { TlsClient } from '../tls-client'

const DEFAULT_PORT = 9600

export function registerTlsIpc(
  tlsClient: TlsClient,
  getMainWindow: () => BrowserWindow | null,
): void {
  ipcMain.handle(IpcChannels.tlsConnect, async (_event, ip: string) => {
    await tlsClient.connect(ip, DEFAULT_PORT)
  })

  ipcMain.handle(IpcChannels.tlsDisconnect, () => {
    tlsClient.disconnect()
  })

  ipcMain.handle(
    IpcChannels.tlsSend,
    async (_event, msgType: number, payload: Uint8Array, timeout?: number) => {
      return tlsClient.send(msgType, payload, timeout)
    },
  )

  tlsClient.on('state-change', (from, to, event) => {
    const mainWindow = getMainWindow()
    if (mainWindow != null && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send(IpcChannels.connectionStateChanged, to)
    }
  })
}
```

- [ ] **Step 4: 提取 window.ts**

`client/src/main/window.ts`：
```typescript
import { BrowserWindow } from 'electron'
import { join } from 'path'

let mainWindow: BrowserWindow | null = null

export function createMainWindow(): BrowserWindow {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 1024,
    minHeight: 700,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
      preload: join(__dirname, '../preload/index.js'),
    },
  })

  const isDev = !require('electron').app.isPackaged
  if (isDev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
  }

  mainWindow.on('closed', () => {
    mainWindow = null
  })

  return mainWindow
}

export function getMainWindow(): BrowserWindow | null {
  return mainWindow
}
```

- [ ] **Step 5: 更新 index.ts 集成 TLS client + IPC**

`client/src/main/index.ts` 替换为：
```typescript
import { app, BrowserWindow } from 'electron'
import { createMainWindow, getMainWindow } from './window'
import { TlsClient } from './tls-client'
import { registerTlsIpc } from './ipc/tls-ipc'

const tlsClient = new TlsClient()

app.whenReady().then(() => {
  registerTlsIpc(tlsClient, getMainWindow)
  createMainWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createMainWindow()
    }
  })
})

app.on('window-all-closed', () => {
  tlsClient.disconnect()
  app.quit()
})
```

- [ ] **Step 6: 验证应用启动**

```bash
cd client
npm run dev
```

Expected: Electron 窗口正常启动，控制台无报错。

- [ ] **Step 7: 提交**

```bash
git add client/src/main/
git commit -m "feat(client): 实现 TLS 传输层、客户端门面和 IPC 桥接

- TlsTransport: Node.js tls.connect，15s 连接超时，证书指纹校验
- TlsClient 门面: 组装 transport/parser/dispatcher/heartbeat/stateMachine
- connect 时触发状态机转换 CONNECT_START → CONNECT_SUCCESS/FAIL
- 心跳超时和网络断开自动转换到 DISCONNECTED，rejectAll pending 请求
- registerTlsIpc: 注册 tls:connect/disconnect/send handler
- 状态变更通过 webContents.send 推送到 renderer
- 提取 window.ts 管理 BrowserWindow 创建
"
```

---

### Task 7: Preload 桥接层

**Files:**
- Modify: `client/src/preload/index.ts`

**Interfaces:**
- Consumes: `IpcChannels` (Task 2), `ConnectionStatus` (Task 2)
- Produces: `window.desktopApi` 对象暴露给 renderer，包含 `tls.connect()`, `tls.disconnect()`, `tls.send()`, `tls.onStateChanged()`, `dialog.openFile()`, `dialog.saveFile()`

- [ ] **Step 1: 实现 preload/index.ts**

`client/src/preload/index.ts`：
```typescript
import { contextBridge, ipcRenderer, type IpcRendererEvent } from 'electron'
import type { ConnectionStatus } from '../shared/connection-state'

export interface OpenFileOptions {
  title?: string
  filters?: Array<{ name: string; extensions: string[] }>
}

export interface SaveFileOptions {
  title?: string
  defaultPath?: string
  filters?: Array<{ name: string; extensions: string[] }>
}

const desktopApi = {
  tls: {
    connect: (ip: string): Promise<void> => ipcRenderer.invoke('tls:connect', ip),

    disconnect: (): Promise<void> => ipcRenderer.invoke('tls:disconnect'),

    send: (
      msgType: number,
      payload: Uint8Array,
      timeout?: number,
    ): Promise<Uint8Array> => ipcRenderer.invoke('tls:send', msgType, payload, timeout),

    onStateChanged: (callback: (status: ConnectionStatus) => void): (() => void) => {
      const handler = (_event: IpcRendererEvent, status: ConnectionStatus) => callback(status)
      ipcRenderer.on('connection:state-changed', handler)
      return () => {
        ipcRenderer.removeListener('connection:state-changed', handler)
      }
    },
  },

  dialog: {
    openFile: (options: OpenFileOptions): Promise<{ canceled: boolean; filePaths: string[] }> =>
      ipcRenderer.invoke('dialog:open-file', options),

    saveFile: (options: SaveFileOptions): Promise<{ canceled: boolean; filePath?: string }> =>
      ipcRenderer.invoke('dialog:save-file', options),
  },
}

contextBridge.exposeInMainWorld('desktopApi', desktopApi)

export type DesktopApi = typeof desktopApi
```

- [ ] **Step 2: 创建 renderer 端类型声明**

`client/src/renderer/env.d.ts`：
```typescript
/// <reference types="vite/client" />

import type { DesktopApi } from '../preload/index'

declare global {
  interface Window {
    desktopApi: DesktopApi
  }
}
```

- [ ] **Step 3: 注册 dialog IPC handler**

在 `client/src/main/ipc/` 下新建 `dialog-ipc.ts`：
```typescript
import { ipcMain, dialog, type BrowserWindow } from 'electron'
import { IpcChannels } from '../../shared/ipc-channels'

export function registerDialogIpc(getMainWindow: () => BrowserWindow | null): void {
  ipcMain.handle(IpcChannels.dialogOpenFile, async (_event, options: unknown) => {
    const mainWindow = getMainWindow()
    if (mainWindow == null) {
      return { canceled: true, filePaths: [] }
    }
    const parsed = options as { title?: string; filters?: Array<{ name: string; extensions: string[] }> }
    return dialog.showOpenDialog(mainWindow, {
      title: parsed?.title,
      filters: parsed?.filters,
      properties: ['openFile'],
    })
  })

  ipcMain.handle(IpcChannels.dialogSaveFile, async (_event, options: unknown) => {
    const mainWindow = getMainWindow()
    if (mainWindow == null) {
      return { canceled: true }
    }
    const parsed = options as { title?: string; defaultPath?: string; filters?: Array<{ name: string; extensions: string[] }> }
    return dialog.showSaveDialog(mainWindow, {
      title: parsed?.title,
      defaultPath: parsed?.defaultPath,
      filters: parsed?.filters,
    })
  })
}
```

更新 `client/src/main/index.ts`，在 `app.whenReady` 中添加：
```typescript
import { registerDialogIpc } from './ipc/dialog-ipc'

// 在 app.whenReady().then 内：
registerDialogIpc(getMainWindow)
```

- [ ] **Step 4: 提交**

```bash
git add client/src/preload/ client/src/renderer/env.d.ts client/src/main/ipc/dialog-ipc.ts client/src/main/index.ts
git commit -m "feat(client): 实现 preload 桥接层和 dialog IPC

- contextBridge 暴露 desktopApi（tls + dialog），不暴露 ipcRenderer/require/process
- tls.onStateChanged 返回取消订阅函数
- 注册 dialog:open-file 和 dialog:save-file IPC handler
- 创建 renderer 端 Window.desktopApi 类型声明
"
```

---

### Task 8: Pinia Stores

**Files:**
- Create: `client/src/renderer/stores/connection.ts`
- Create: `client/src/renderer/stores/session.ts`
- Create: `client/src/renderer/stores/ui.ts`
- Modify: `client/src/renderer/main.ts` (注册 Pinia)
- Test: `client/tests/unit/stores/connection.test.ts`
- Test: `client/tests/unit/stores/session.test.ts`
- Test: `client/tests/unit/stores/ui.test.ts`

**Interfaces:**
- Consumes: `ConnectionStatus`, `UserRole`, `AuthStatus` (Task 2), `window.desktopApi` (Task 7)
- Produces:
  - `useConnectionStore()`: `status`, `deviceIp`, `wasConnected`, `isConnected`, `isDisconnected`, `connect(ip)`, `disconnect()`, `updateStatus(status)`
  - `useSessionStore()`: `token`, `username`, `role`, `authStatus`, `isLoggedIn`, `isAuthorized`, `login()`, `logout()`, `validateSession()`, `clearSession()`, `startInactivityTimer()`, `resetInactivityTimer()`, `stopInactivityTimer()`
  - `useUiStore()`: `isGlobalLoading`, `toastQueue`, `showLoading()`, `hideLoading()`, `showToast()`, `removeToast()`

- [ ] **Step 1: 注册 Pinia 到 Vue 入口**

修改 `client/src/renderer/main.ts`：
```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
```

- [ ] **Step 2: 实现 connection store**

`client/src/renderer/stores/connection.ts`：
```typescript
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { defineStore } from 'pinia'
import type { ConnectionStatus } from '../../shared/connection-state'

export const useConnectionStore = defineStore('connection', () => {
  const status = ref<ConnectionStatus>('DISCONNECTED')
  const deviceIp = ref('')
  const wasConnected = ref(false)

  const isConnected = computed(() => status.value === 'CONNECTED')
  const isDisconnected = computed(() => status.value === 'DISCONNECTED')

  let unsubscribe: (() => void) | null = null

  function setupListener(): void {
    if (window.desktopApi?.tls?.onStateChanged == null) {
      return
    }
    unsubscribe = window.desktopApi.tls.onStateChanged((newStatus: ConnectionStatus) => {
      if (newStatus === 'CONNECTED') {
        wasConnected.value = true
      }
      status.value = newStatus
    })
  }

  function teardownListener(): void {
    if (unsubscribe != null) {
      unsubscribe()
      unsubscribe = null
    }
  }

  async function connect(ip: string): Promise<void> {
    deviceIp.value = ip
    await window.desktopApi.tls.connect(ip)
  }

  function disconnect(): void {
    window.desktopApi.tls.disconnect()
  }

  function updateStatus(newStatus: ConnectionStatus): void {
    status.value = newStatus
  }

  async function reconnect(): Promise<boolean> {
    if (deviceIp.value === '') {
      return false
    }
    try {
      await window.desktopApi.tls.connect(deviceIp.value)
      return true
    } catch {
      return false
    }
  }

  return {
    status,
    deviceIp,
    wasConnected,
    isConnected,
    isDisconnected,
    connect,
    disconnect,
    reconnect,
    updateStatus,
    setupListener,
    teardownListener,
  }
})
```

- [ ] **Step 3: 实现 session store**

`client/src/renderer/stores/session.ts`：
```typescript
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { useRouter } from 'vue-router'
import type { UserRole, AuthStatus } from '../../shared/connection-state'
import { useConnectionStore } from './connection'

const INACTIVITY_TIMEOUT = 5 * 60 * 1000

export interface LoginResult {
  success: boolean
  resultCode: number
  errorMessage: string
}

export const useSessionStore = defineStore('session', () => {
  const token = ref('')
  const username = ref('')
  const role = ref<UserRole | ''>('')
  const authStatus = ref<AuthStatus>('')
  const authExpireTime = ref(0)
  const deviceDescription = ref('')

  const isLoggedIn = computed(() => token.value !== '')
  const isAuthorized = computed(() => authStatus.value === 'authorized')

  let inactivityTimer: ReturnType<typeof setTimeout> | null = null

  function clearSession(): void {
    token.value = ''
    username.value = ''
    role.value = ''
    authStatus.value = ''
    authExpireTime.value = 0
    deviceDescription.value = ''
    stopInactivityTimer()
  }

  function setSession(data: {
    token: string
    username: string
    role: UserRole
    authStatus: AuthStatus
    authExpireTime: number
    deviceDescription: string
  }): void {
    token.value = data.token
    username.value = data.username
    role.value = data.role
    authStatus.value = data.authStatus
    authExpireTime.value = data.authExpireTime
    deviceDescription.value = data.deviceDescription
  }

  async function login(ip: string, loginUsername: string, password: string): Promise<LoginResult> {
    const connection = useConnectionStore()
    await connection.connect(ip)

    const response = await window.desktopApi.tls.send({
      msgType: 0x0001,
      payload: { username: loginUsername, password },
    })

    if (!response.success) {
      return {
        success: false,
        resultCode: response.resultCode,
        errorMessage: response.errorMessage,
      }
    }

    setSession({
      token: response.sessionToken,
      username: response.username,
      role: response.role as UserRole,
      authStatus: response.authStatus as AuthStatus,
      authExpireTime: response.authExpireTime,
      deviceDescription: response.deviceDescription,
    })
    startInactivityTimer()

    return { success: true, resultCode: 0, errorMessage: '' }
  }

  async function logout(): Promise<void> {
    const connection = useConnectionStore()
    if (token.value !== '' && connection.isConnected) {
      try {
        await window.desktopApi.tls.send({
          msgType: 0x0009,
          payload: { sessionToken: token.value },
        })
      } catch {
        // 异常处理：断开连接时无法通知装置端，装置侧按超时失效
      }
    }
    clearSession()
    connection.disconnect()
  }

  function startInactivityTimer(): void {
    stopInactivityTimer()
    inactivityTimer = setTimeout(() => {
      const connection = useConnectionStore()
      clearSession()
      connection.disconnect()
    }, INACTIVITY_TIMEOUT)
  }

  function resetInactivityTimer(): void {
    if (inactivityTimer != null) {
      startInactivityTimer()
    }
  }

  function stopInactivityTimer(): void {
    if (inactivityTimer != null) {
      clearTimeout(inactivityTimer)
      inactivityTimer = null
    }
  }

  async function validateSession(): Promise<boolean> {
    if (token.value === '') {
      return false
    }
    try {
      const response = await window.desktopApi.tls.send({
        msgType: 0x0003,
        payload: { sessionToken: token.value },
      })
      if (response.authStatus === 'authorized' || response.authStatus === 'expired') {
        authStatus.value = response.authStatus as AuthStatus
        authExpireTime.value = response.expireTime ?? 0
        deviceDescription.value = response.deviceDescription ?? deviceDescription.value
        startInactivityTimer()
        return true
      }
      clearSession()
      return false
    } catch {
      clearSession()
      return false
    }
  }

  return {
    token,
    username,
    role,
    authStatus,
    authExpireTime,
    deviceDescription,
    isLoggedIn,
    isAuthorized,
    login,
    logout,
    validateSession,
    clearSession,
    setSession,
    startInactivityTimer,
    resetInactivityTimer,
    stopInactivityTimer,
  }
})
```

- [ ] **Step 4: 实现 ui store**

`client/src/renderer/stores/ui.ts`：
```typescript
import { ref } from 'vue'
import { defineStore } from 'pinia'

export type ToastType = 'success' | 'warning' | 'error' | 'info'

export interface ToastItem {
  id: string
  message: string
  type: ToastType
}

let toastIdCounter = 0

export const useUiStore = defineStore('ui', () => {
  const isGlobalLoading = ref(false)
  const toastQueue = ref<ToastItem[]>([])

  function showLoading(): void {
    isGlobalLoading.value = true
  }

  function hideLoading(): void {
    isGlobalLoading.value = false
  }

  function showToast(message: string, type: ToastType = 'info'): void {
    const id = `toast-${++toastIdCounter}`
    toastQueue.value.push({ id, message, type })
  }

  function removeToast(id: string): void {
    const index = toastQueue.value.findIndex((t) => t.id === id)
    if (index !== -1) {
      toastQueue.value.splice(index, 1)
    }
  }

  return {
    isGlobalLoading,
    toastQueue,
    showLoading,
    hideLoading,
    showToast,
    removeToast,
  }
})
```

- [ ] **Step 5: 编写 stores 测试**

`client/tests/unit/stores/connection.test.ts`：
```typescript
import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useConnectionStore } from '../../../src/renderer/stores/connection'

describe('useConnectionStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('starts with DISCONNECTED status', () => {
    const store = useConnectionStore()
    expect(store.status).toBe('DISCONNECTED')
    expect(store.isDisconnected).toBe(true)
    expect(store.isConnected).toBe(false)
  })

  it('updateStatus changes status and computed values', () => {
    const store = useConnectionStore()
    store.updateStatus('CONNECTED')
    expect(store.status).toBe('CONNECTED')
    expect(store.isConnected).toBe(true)
    expect(store.isDisconnected).toBe(false)
  })
})
```

`client/tests/unit/stores/session.test.ts`：
```typescript
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSessionStore } from '../../../src/renderer/stores/session'

describe('useSessionStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('starts with empty session', () => {
    const store = useSessionStore()
    expect(store.isLoggedIn).toBe(false)
    expect(store.token).toBe('')
  })

  it('setSession populates all fields', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 1234567890,
      deviceDescription: 'test-device',
    })
    expect(store.isLoggedIn).toBe(true)
    expect(store.isAuthorized).toBe(true)
    expect(store.username).toBe('admin')
    expect(store.role).toBe('admin')
  })

  it('clearSession resets all fields', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 1234567890,
      deviceDescription: 'test-device',
    })
    store.clearSession()
    expect(store.isLoggedIn).toBe(false)
    expect(store.token).toBe('')
    expect(store.username).toBe('')
  })

  it('inactivity timer clears session after 5 minutes', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    store.startInactivityTimer()

    vi.advanceTimersByTime(5 * 60 * 1000)
    expect(store.isLoggedIn).toBe(false)
  })

  it('resetInactivityTimer postpones timeout', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    store.startInactivityTimer()

    vi.advanceTimersByTime(4 * 60 * 1000)
    store.resetInactivityTimer()

    vi.advanceTimersByTime(4 * 60 * 1000)
    expect(store.isLoggedIn).toBe(true)

    vi.advanceTimersByTime(60 * 1000)
    expect(store.isLoggedIn).toBe(false)
  })

  it('stopInactivityTimer cancels pending timeout', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    store.startInactivityTimer()
    store.stopInactivityTimer()

    vi.advanceTimersByTime(6 * 60 * 1000)
    expect(store.isLoggedIn).toBe(true)
  })
})
```

`client/tests/unit/stores/ui.test.ts`：
```typescript
import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useUiStore } from '../../../src/renderer/stores/ui'

describe('useUiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('toggles global loading', () => {
    const store = useUiStore()
    expect(store.isGlobalLoading).toBe(false)
    store.showLoading()
    expect(store.isGlobalLoading).toBe(true)
    store.hideLoading()
    expect(store.isGlobalLoading).toBe(false)
  })

  it('adds and removes toasts', () => {
    const store = useUiStore()
    store.showToast('操作成功', 'success')
    expect(store.toastQueue.length).toBe(1)
    expect(store.toastQueue[0].message).toBe('操作成功')

    store.removeToast(store.toastQueue[0].id)
    expect(store.toastQueue.length).toBe(0)
  })
})
```

- [ ] **Step 6: 运行 stores 测试**

需要在 `vitest.config.ts` 中配置 stores 测试使用 happy-dom 环境，或在测试文件中标记。更新 `vitest.config.ts`：
```typescript
import { defineConfig } from 'vitest/config'
import { resolve } from 'path'

export default defineConfig({
  test: {
    include: ['tests/**/*.test.ts'],
    environmentMatchGlobs: [
      ['tests/unit/stores/**', 'happy-dom'],
    ],
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src/renderer'),
    },
  },
})
```

```bash
npx vitest run tests/unit/stores/
```

Expected: 全部 PASS。

- [ ] **Step 7: 提交**

```bash
git add client/src/renderer/stores/ client/src/renderer/main.ts \
       client/tests/unit/stores/ client/vitest.config.ts
git commit -m "feat(client): 实现 Pinia stores（connection/session/ui）

- connection store: 管理连接状态和装置 IP，监听主进程状态变更
- session store: 管理 token/username/role/authStatus，仅进程内保存
- session store: 5 分钟无操作自动清空会话，心跳不重置计时器
- ui store: 全局 loading 和 toast 队列管理
- 单元测试覆盖状态初始化、更新、清空、无操作计时器
"
```

---

### Task 9: 路由与权限守卫

**Files:**
- Create: `client/src/renderer/router/routes.ts`
- Create: `client/src/renderer/router/index.ts`
- Modify: `client/src/renderer/main.ts` (注册 router)

**Interfaces:**
- Consumes: `useSessionStore()` (Task 8), `useConnectionStore()` (Task 8), `UserRole` (Task 2)
- Produces: Vue Router 实例，含 8 个路由 + `beforeEach` 守卫逻辑

- [ ] **Step 1: 定义路由配置**

`client/src/renderer/router/routes.ts`：
```typescript
import type { RouteRecordRaw } from 'vue-router'
import type { UserRole } from '../../shared/connection-state'

declare module 'vue-router' {
  interface RouteMeta {
    guest?: boolean
    requiresAuth?: boolean
    roles?: UserRole[]
  }
}

export const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/pages/LoginPage.vue'),
    meta: { guest: true },
  },
  {
    path: '/license',
    name: 'License',
    component: () => import('@/pages/LicensePage.vue'),
    meta: { requiresAuth: false },
  },
  {
    path: '/file-access',
    name: 'FileAccess',
    component: () => import('@/pages/FileAccessPage.vue'),
    meta: { requiresAuth: true, roles: ['operator'] },
  },
  {
    path: '/usb-devices',
    name: 'UsbDevices',
    component: () => import('@/pages/UsbDevicesPage.vue'),
    meta: { requiresAuth: true, roles: ['operator'] },
  },
  {
    path: '/policies',
    name: 'Policies',
    component: () => import('@/pages/PoliciesPage.vue'),
    meta: { requiresAuth: true, roles: ['operator'] },
  },
  {
    path: '/logs',
    name: 'Logs',
    component: () => import('@/pages/LogsPage.vue'),
    meta: { requiresAuth: true, roles: ['auditor'] },
  },
  {
    path: '/system',
    name: 'System',
    component: () => import('@/pages/SystemPage.vue'),
    meta: { requiresAuth: true, roles: ['admin'] },
  },
  {
    path: '/users',
    name: 'Users',
    component: () => import('@/pages/UsersPage.vue'),
    meta: { requiresAuth: true, roles: ['admin'] },
  },
  {
    path: '/',
    redirect: '/login',
  },
]

export const ROLE_DEFAULT_ROUTES: Record<UserRole, string> = {
  admin: '/users',
  operator: '/file-access',
  auditor: '/logs',
}
```

- [ ] **Step 2: 实现路由守卫**

`client/src/renderer/router/index.ts`：
```typescript
import { createRouter, createWebHashHistory } from 'vue-router'
import { routes, ROLE_DEFAULT_ROUTES } from './routes'
import { useSessionStore } from '@/stores/session'
import { useConnectionStore } from '@/stores/connection'
import type { UserRole } from '../../shared/connection-state'

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

router.beforeEach((to, _from) => {
  if (to.meta.guest === true) {
    return true
  }

  const session = useSessionStore()
  if (!session.isLoggedIn) {
    return '/login'
  }

  const connection = useConnectionStore()
  if (
    connection.status === 'AUTH_REQUIRED' ||
    connection.status === 'LICENSE_EXPIRED'
  ) {
    return '/license'
  }

  if (to.meta.requiresAuth === true && to.meta.roles != null) {
    const currentRole = session.role as UserRole
    if (!to.meta.roles.includes(currentRole)) {
      const defaultRoute = ROLE_DEFAULT_ROUTES[currentRole]
      return defaultRoute ?? '/login'
    }
  }

  return true
})

export default router
```

- [ ] **Step 3: 注册 router 到 Vue 入口**

修改 `client/src/renderer/main.ts`：
```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.mount('#app')
```

- [ ] **Step 4: 创建页面占位组件**

为路由中引用的每个页面创建最小占位组件（后续 Task 补充内容）：

`client/src/renderer/pages/LoginPage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="login-page">登录页（占位）</div>
</template>
```

其他 7 个页面文件格式相同（LicensePage.vue、FileAccessPage.vue、UsbDevicesPage.vue、PoliciesPage.vue、LogsPage.vue、SystemPage.vue、UsersPage.vue），只修改 class 名和文字。

例如 `LicensePage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="license-page">授权页（占位）</div>
</template>
```

`FileAccessPage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="file-access-page">文件访问控制（占位）</div>
</template>
```

`UsbDevicesPage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="usb-devices-page">U盘设备控制（占位）</div>
</template>
```

`PoliciesPage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="policies-page">策略管理（占位）</div>
</template>
```

`LogsPage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="logs-page">日志管理（占位）</div>
</template>
```

`SystemPage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="system-page">系统管理（占位）</div>
</template>
```

`UsersPage.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <div class="users-page">用户管理（占位）</div>
</template>
```

- [ ] **Step 5: 更新 App.vue 使用 router-view**

`client/src/renderer/App.vue`：
```vue
<script setup lang="ts">
</script>

<template>
  <router-view />
</template>
```

- [ ] **Step 6: 验证路由启动**

```bash
cd client
npm run dev
```

Expected: 应用启动后自动跳转到 `#/login`，显示"登录页（占位）"。

- [ ] **Step 7: 提交**

```bash
git add client/src/renderer/router/ client/src/renderer/pages/ \
       client/src/renderer/App.vue client/src/renderer/main.ts
git commit -m "feat(client): 实现路由配置与权限守卫

- 定义 8 个路由：login(guest)、license、file-access/usb-devices/policies(operator)、logs(auditor)、system/users(admin)
- beforeEach 守卫：未登录跳 login，AUTH_REQUIRED/LICENSE_EXPIRED 跳 license，角色不匹配跳默认页
- 使用 createWebHashHistory 避免 Electron file:// 协议路由问题
- 创建 8 个页面占位组件
- 角色默认页：admin→/users, operator→/file-access, auditor→/logs
"
```

---

### Task 10: 设计变量与主框架布局

**Files:**
- Create: `client/src/renderer/styles/variables.scss`
- Create: `client/src/renderer/layouts/MainLayout.vue`

**Interfaces:**
- Consumes: `useSessionStore()` (Task 8), `useConnectionStore()` (Task 8), `UserRole` (Task 2), `ROLE_DEFAULT_ROUTES` (Task 9)
- Produces: `MainLayout.vue` 含 Header/Sidebar/Content/Footer，供业务页面使用

- [ ] **Step 1: 创建设计变量文件**

`client/src/renderer/styles/variables.scss`：
```scss
// 品牌色
$brand-primary: #0056b3;
$brand-header-start: #004b9a;
$brand-header-end: #0066cc;

// 布局
$sidebar-width: 190px;
$header-height: 48px;

// 通用色彩
$color-success: #67c23a;
$color-warning: #e6a23c;
$color-danger: #f56c6c;
$color-info: #909399;

// 文字
$text-primary: #303133;
$text-regular: #606266;
$text-secondary: #909399;

// 背景
$bg-sidebar: #f5f7fa;
$bg-page: #f0f2f5;

// 边框/圆角/阴影
$border-color: #dcdfe6;
$border-radius: 4px;
$box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
```

- [ ] **Step 2: 实现 MainLayout**

`client/src/renderer/layouts/MainLayout.vue`：
```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useSessionStore } from '@/stores/session'
import { useConnectionStore } from '@/stores/connection'
import ChangePasswordDialog from '@/components/ChangePasswordDialog.vue'
import type { UserRole } from '../../shared/connection-state'

interface MenuItem {
  label: string
  path: string
  roles: UserRole[]
}

const ALL_MENUS: MenuItem[] = [
  { label: '文件访问控制', path: '/file-access', roles: ['operator'] },
  { label: 'U盘设备控制', path: '/usb-devices', roles: ['operator'] },
  { label: '策略管理', path: '/policies', roles: ['operator'] },
  { label: '日志管理', path: '/logs', roles: ['auditor'] },
  { label: '系统管理', path: '/system', roles: ['admin'] },
  { label: '用户管理', path: '/users', roles: ['admin'] },
]

const session = useSessionStore()
const connection = useConnectionStore()
const router = useRouter()
const route = useRoute()

const changePasswordVisible = ref(false)

const visibleMenus = computed(() => {
  const currentRole = session.role as UserRole
  if (!currentRole) {
    return []
  }
  return ALL_MENUS.filter((menu) => menu.roles.includes(currentRole))
})

function navigateTo(path: string): void {
  router.push(path)
}

function handleUserCommand(command: string): void {
  if (command === 'change-password') {
    changePasswordVisible.value = true
  } else if (command === 'logout') {
    handleLogout()
  }
}

async function handleLogout(): Promise<void> {
  await session.logout()
  router.push('/login')
}
</script>

<template>
  <div class="main-layout">
    <header class="main-header">
      <div class="header-left">
        <span class="brand-name">USB安全管理系统</span>
      </div>
      <div class="header-right">
        <span class="device-ip">{{ connection.deviceIp }}</span>
        <el-dropdown trigger="click" @command="handleUserCommand">
          <span class="user-dropdown-trigger">
            {{ session.username }}
            <el-icon><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="change-password">修改密码</el-dropdown-item>
              <el-dropdown-item command="logout" divided>登出</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </header>

    <div class="main-body">
      <aside class="main-sidebar">
        <nav class="sidebar-nav">
          <div
            v-for="menu in visibleMenus"
            :key="menu.path"
            class="nav-item"
            :class="{ active: route.path === menu.path }"
            @click="navigateTo(menu.path)"
          >
            {{ menu.label }}
          </div>
        </nav>
      </aside>

      <main class="main-content">
        <router-view />
      </main>
    </div>

    <footer class="main-footer">
      <span
        class="connection-dot"
        :class="connection.isConnected ? 'connected' : 'disconnected'"
      />
      <span>{{ connection.isConnected ? '已连接' : '未连接' }}</span>
    </footer>

    <ChangePasswordDialog v-model:visible="changePasswordVisible" />
  </div>
</template>

<style scoped lang="scss">
@use '../styles/variables' as *;

.main-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-header {
  height: $header-height;
  background: linear-gradient(90deg, $brand-header-start, $brand-header-end);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
}

.brand-name {
  font-size: 16px;
  font-weight: 600;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-dropdown-trigger {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #fff;
  cursor: pointer;
  font-size: 14px;
}

.main-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.main-sidebar {
  width: $sidebar-width;
  background: $bg-sidebar;
  border-right: 1px solid $border-color;
  flex-shrink: 0;
  overflow-y: auto;
}

.sidebar-nav {
  padding: 8px 0;
}

.nav-item {
  padding: 12px 20px;
  cursor: pointer;
  color: $text-primary;
  font-size: 14px;

  &:hover {
    background: darken($bg-sidebar, 5%);
  }

  &.active {
    color: $brand-primary;
    background: #e6f0fa;
    border-right: 3px solid $brand-primary;
    font-weight: 500;
  }
}

.main-content {
  flex: 1;
  overflow-y: auto;
  background: $bg-page;
  padding: 16px;
}

.main-footer {
  height: 28px;
  background: #fff;
  border-top: 1px solid $border-color;
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 6px;
  font-size: 12px;
  color: $text-secondary;
  flex-shrink: 0;
}

.connection-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;

  &.connected {
    background: $color-success;
  }

  &.disconnected {
    background: $color-danger;
  }
}
</style>
```

- [ ] **Step 3: 更新 App.vue 区分布局**

LoginPage 和 LicensePage 不使用 MainLayout，其他页面使用。修改路由使用嵌套路由：

更新 `client/src/renderer/router/routes.ts`，将业务页面嵌套在 MainLayout 路由下：
```typescript
import type { RouteRecordRaw } from 'vue-router'
import type { UserRole } from '../../shared/connection-state'

declare module 'vue-router' {
  interface RouteMeta {
    guest?: boolean
    requiresAuth?: boolean
    roles?: UserRole[]
  }
}

export const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/pages/LoginPage.vue'),
    meta: { guest: true },
  },
  {
    path: '/license',
    name: 'License',
    component: () => import('@/pages/LicensePage.vue'),
    meta: { requiresAuth: false },
  },
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      {
        path: 'file-access',
        name: 'FileAccess',
        component: () => import('@/pages/FileAccessPage.vue'),
        meta: { requiresAuth: true, roles: ['operator'] },
      },
      {
        path: 'usb-devices',
        name: 'UsbDevices',
        component: () => import('@/pages/UsbDevicesPage.vue'),
        meta: { requiresAuth: true, roles: ['operator'] },
      },
      {
        path: 'policies',
        name: 'Policies',
        component: () => import('@/pages/PoliciesPage.vue'),
        meta: { requiresAuth: true, roles: ['operator'] },
      },
      {
        path: 'logs',
        name: 'Logs',
        component: () => import('@/pages/LogsPage.vue'),
        meta: { requiresAuth: true, roles: ['auditor'] },
      },
      {
        path: 'system',
        name: 'System',
        component: () => import('@/pages/SystemPage.vue'),
        meta: { requiresAuth: true, roles: ['admin'] },
      },
      {
        path: 'users',
        name: 'Users',
        component: () => import('@/pages/UsersPage.vue'),
        meta: { requiresAuth: true, roles: ['admin'] },
      },
      {
        path: '',
        redirect: '/login',
      },
    ],
  },
]

export const ROLE_DEFAULT_ROUTES: Record<UserRole, string> = {
  admin: '/users',
  operator: '/file-access',
  auditor: '/logs',
}
```

- [ ] **Step 4: 验证布局**

```bash
cd client
npm run dev
```

Expected: 登录页全屏显示，业务页面带有 Header + Sidebar + Content + Footer 布局。

- [ ] **Step 5: 提交**

```bash
git add client/src/renderer/styles/ client/src/renderer/layouts/ \
       client/src/renderer/router/routes.ts
git commit -m "feat(client): 实现设计变量与主框架布局

- 创建 variables.scss 集中管理品牌色/布局/色彩/文字/背景变量
- 实现 MainLayout：Header 渐变(#004b9a→#0066cc) + Sidebar(190px) + Content + Footer
- Header 展示品牌名/装置IP/用户名/登出按钮
- Sidebar 按角色过滤菜单项，高亮当前路由
- Footer 连接状态指示器（绿点/红点）
- Login 和 License 使用独立全屏布局，业务页使用嵌套路由
"
```

---

### Task 11: Services 层

**Files:**
- Create: `client/src/renderer/services/send-command.ts`
- Create: `client/src/renderer/services/auth-service.ts`
- Create: `client/src/renderer/services/whitelist-service.ts`
- Create: `client/src/renderer/services/device-service.ts`
- Create: `client/src/renderer/services/file-policy-service.ts`
- Create: `client/src/renderer/services/policy-service.ts`
- Create: `client/src/renderer/services/log-service.ts`
- Create: `client/src/renderer/services/system-service.ts`
- Create: `client/src/renderer/services/user-service.ts`

**Interfaces:**
- Consumes: `window.desktopApi.tls.send()` (Task 7), Protobuf 生成模块 (Task 3), `ResultCode` (Task 2), `getResultMessage()` (Task 2), `useSessionStore().clearSession()` (Task 8)
- Produces: 8 个 service 模块，所有协议调用的唯一入口

- [ ] **Step 1: 实现通用发送函数**

`client/src/renderer/services/send-command.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import { ResultCode } from '../../shared/result-codes'
import { getResultMessage } from '@/utils/result-code'

const DEFAULT_TIMEOUT = 15_000
const FILE_TRANSFER_TIMEOUT = 300_000

export class ServiceError extends Error {
  resultCode: number

  constructor(message: string, resultCode: number) {
    super(message)
    this.name = 'ServiceError'
    this.resultCode = resultCode
  }
}

export async function sendCommand(
  msgType: number,
  payload: Uint8Array,
  timeout: number = DEFAULT_TIMEOUT,
): Promise<Uint8Array> {
  return window.desktopApi.tls.send(msgType, payload, timeout)
}

export function handleCommonResponse(rsp: usb_control.RspCommon): void {
  if (rsp.success) {
    return
  }
  const code = rsp.resultCode
  const msg = getResultMessage(code, rsp.errorMessage)
  if (msg === null) {
    throw new ServiceError('UNAUTHENTICATED', code)
  }
  throw new ServiceError(msg, code)
}

export { DEFAULT_TIMEOUT, FILE_TRANSFER_TIMEOUT }
```

- [ ] **Step 2: 实现 auth-service**

`client/src/renderer/services/auth-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import {
  sendCommand,
  handleCommonResponse,
  FILE_TRANSFER_TIMEOUT,
} from './send-command'

const MSG_CMD_LOGIN = 0x0001
const MSG_RSP_LOGIN = 0x0002
const MSG_CMD_AUTH_STATUS_QUERY = 0x0003
const MSG_RSP_AUTH_STATUS = 0x0004
const MSG_CMD_GET_MACHINE_CODE = 0x0005
const MSG_RSP_MACHINE_CODE = 0x0006
const MSG_CMD_UPLOAD_LICENSE = 0x0007
const MSG_RSP_UPLOAD_LICENSE = 0x0008
const MSG_CMD_LOGOUT = 0x0009
const MSG_RSP_COMMON = 0xff00

export async function login(
  username: string,
  password: string,
): Promise<usb_control.RspLogin> {
  const cmd = usb_control.CmdLogin.fromObject({ username, password })
  const payload = usb_control.CmdLogin.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_LOGIN, payload)
  return usb_control.RspLogin.decode(rspPayload)
}

export async function logout(token: string): Promise<void> {
  const cmd = usb_control.CmdLogout.fromObject({ sessionToken: token })
  const payload = usb_control.CmdLogout.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_LOGOUT, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function queryAuthStatus(
  token: string,
): Promise<usb_control.RspAuthStatus> {
  const cmd = usb_control.CmdAuthStatusQuery.fromObject({ sessionToken: token })
  const payload = usb_control.CmdAuthStatusQuery.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_AUTH_STATUS_QUERY, payload)
  return usb_control.RspAuthStatus.decode(rspPayload)
}

export async function getMachineCode(
  token: string,
): Promise<usb_control.RspMachineCode> {
  const cmd = usb_control.CmdGetMachineCode.fromObject({ sessionToken: token })
  const payload = usb_control.CmdGetMachineCode.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_GET_MACHINE_CODE, payload)
  return usb_control.RspMachineCode.decode(rspPayload)
}

export async function uploadLicense(
  token: string,
  licenseData: Uint8Array,
): Promise<usb_control.RspUploadLicense> {
  const cmd = usb_control.CmdUploadLicense.fromObject({
    sessionToken: token,
    licenseData,
  })
  const payload = usb_control.CmdUploadLicense.encode(cmd).finish()
  const rspPayload = await sendCommand(
    MSG_CMD_UPLOAD_LICENSE,
    payload,
    FILE_TRANSFER_TIMEOUT,
  )
  return usb_control.RspUploadLicense.decode(rspPayload)
}
```

- [ ] **Step 3: 实现 whitelist-service**

`client/src/renderer/services/whitelist-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import { sendCommand, handleCommonResponse } from './send-command'

const MSG_CMD_LIST_WHITELIST = 0x0100
const MSG_CMD_ADD_WHITELIST = 0x0104
const MSG_CMD_REMOVE_WHITELIST = 0x0105
const MSG_CMD_UPDATE_WHITELIST = 0x0106

export async function listWhitelist(
  token: string,
): Promise<usb_control.RspListWhitelist> {
  const cmd = usb_control.CmdListWhitelist.fromObject({ sessionToken: token })
  const payload = usb_control.CmdListWhitelist.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_LIST_WHITELIST, payload)
  return usb_control.RspListWhitelist.decode(rspPayload)
}

export async function addWhitelist(
  token: string,
  device: {
    serialNumber: string
    vid: string
    pid: string
    deviceName: string
    capacityBytes: number
    permission: string
    description: string
    addMethod: string
    deviceType: string
  },
): Promise<void> {
  const cmd = usb_control.CmdAddWhitelist.fromObject({
    sessionToken: token,
    ...device,
  })
  const payload = usb_control.CmdAddWhitelist.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_ADD_WHITELIST, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function removeWhitelist(
  token: string,
  serialNumber: string,
): Promise<void> {
  const cmd = usb_control.CmdRemoveWhitelist.fromObject({
    sessionToken: token,
    serialNumber,
  })
  const payload = usb_control.CmdRemoveWhitelist.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_REMOVE_WHITELIST, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function updateWhitelist(
  token: string,
  serialNumber: string,
  permission: string,
  description: string,
): Promise<void> {
  const cmd = usb_control.CmdUpdateWhitelist.fromObject({
    sessionToken: token,
    serialNumber,
    permission,
    description,
  })
  const payload = usb_control.CmdUpdateWhitelist.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_UPDATE_WHITELIST, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}
```

- [ ] **Step 4: 实现 device-service**

`client/src/renderer/services/device-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import { sendCommand } from './send-command'

const MSG_CMD_GET_CONNECTED_DEVICES = 0x0102

export async function getConnectedDevices(
  token: string,
): Promise<usb_control.RspConnectedDevices> {
  const cmd = usb_control.CmdGetConnectedDevices.fromObject({ sessionToken: token })
  const payload = usb_control.CmdGetConnectedDevices.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_GET_CONNECTED_DEVICES, payload)
  return usb_control.RspConnectedDevices.decode(rspPayload)
}
```

- [ ] **Step 5: 实现 file-policy-service**

`client/src/renderer/services/file-policy-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import { sendCommand, handleCommonResponse } from './send-command'

const MSG_CMD_GET_FILE_POLICY = 0x0200
const MSG_CMD_UPDATE_FILE_POLICY_SWITCH = 0x0202
const MSG_CMD_ADD_BLACKLIST_EXTENSION = 0x0203
const MSG_CMD_REMOVE_BLACKLIST_EXTENSION = 0x0204

export async function getFilePolicy(
  token: string,
): Promise<usb_control.RspFilePolicy> {
  const cmd = usb_control.CmdGetFilePolicy.fromObject({ sessionToken: token })
  const payload = usb_control.CmdGetFilePolicy.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_GET_FILE_POLICY, payload)
  return usb_control.RspFilePolicy.decode(rspPayload)
}

export async function updateSwitch(
  token: string,
  policyKey: string,
  enabled: boolean,
): Promise<void> {
  const cmd = usb_control.CmdUpdateFilePolicySwitch.fromObject({
    sessionToken: token,
    policyKey,
    enabled,
  })
  const payload = usb_control.CmdUpdateFilePolicySwitch.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_UPDATE_FILE_POLICY_SWITCH, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function addBlacklistExtension(
  token: string,
  extension: string,
  description: string,
): Promise<void> {
  const cmd = usb_control.CmdAddBlacklistExtension.fromObject({
    sessionToken: token,
    extension,
    description,
  })
  const payload = usb_control.CmdAddBlacklistExtension.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_ADD_BLACKLIST_EXTENSION, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function removeBlacklistExtension(
  token: string,
  extension: string,
): Promise<void> {
  const cmd = usb_control.CmdRemoveBlacklistExtension.fromObject({
    sessionToken: token,
    extension,
  })
  const payload = usb_control.CmdRemoveBlacklistExtension.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_REMOVE_BLACKLIST_EXTENSION, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}
```

- [ ] **Step 6: 实现 policy-service**

`client/src/renderer/services/policy-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import {
  sendCommand,
  handleCommonResponse,
  FILE_TRANSFER_TIMEOUT,
} from './send-command'

const MSG_CMD_EXPORT_POLICY = 0x0300
const MSG_CMD_IMPORT_POLICY = 0x0302

export async function exportPolicy(
  token: string,
): Promise<usb_control.RspExportPolicy> {
  const cmd = usb_control.CmdExportPolicy.fromObject({ sessionToken: token })
  const payload = usb_control.CmdExportPolicy.encode(cmd).finish()
  const rspPayload = await sendCommand(
    MSG_CMD_EXPORT_POLICY,
    payload,
    FILE_TRANSFER_TIMEOUT,
  )
  return usb_control.RspExportPolicy.decode(rspPayload)
}

export async function importPolicy(
  token: string,
  policyData: Uint8Array,
): Promise<void> {
  const cmd = usb_control.CmdImportPolicy.fromObject({
    sessionToken: token,
    policyData,
  })
  const payload = usb_control.CmdImportPolicy.encode(cmd).finish()
  const rspPayload = await sendCommand(
    MSG_CMD_IMPORT_POLICY,
    payload,
    FILE_TRANSFER_TIMEOUT,
  )
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}
```

- [ ] **Step 7: 实现 log-service**

`client/src/renderer/services/log-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import {
  sendCommand,
  handleCommonResponse,
  FILE_TRANSFER_TIMEOUT,
} from './send-command'

const MSG_CMD_QUERY_LOGS = 0x0400
const MSG_CMD_EXPORT_LOGS = 0x0402
const MSG_CMD_DELETE_LOGS = 0x0404

export interface LogQueryParams {
  logType: string
  startTime: number
  endTime: number
  keyword: string
  eventType: string
  page: number
  pageSize: number
  logCategory: string
  actionType: string
}

export async function queryLogs(
  token: string,
  params: LogQueryParams,
): Promise<usb_control.RspQueryLogs> {
  const cmd = usb_control.CmdQueryLogs.fromObject({
    sessionToken: token,
    logType: params.logType,
    startTime: params.startTime,
    endTime: params.endTime,
    keyword: params.keyword,
    eventType: params.eventType,
    page: params.page,
    pageSize: params.pageSize,
    logCategory: params.logCategory,
    actionType: params.actionType,
  })
  const payload = usb_control.CmdQueryLogs.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_QUERY_LOGS, payload)
  return usb_control.RspQueryLogs.decode(rspPayload)
}

export async function exportLogs(
  token: string,
  params: {
    logType: string
    startTime: number
    endTime: number
    keyword: string
    eventType: string
    logCategory: string
    actionType: string
  },
): Promise<usb_control.RspExportLogs> {
  const cmd = usb_control.CmdExportLogs.fromObject({
    sessionToken: token,
    ...params,
  })
  const payload = usb_control.CmdExportLogs.encode(cmd).finish()
  const rspPayload = await sendCommand(
    MSG_CMD_EXPORT_LOGS,
    payload,
    FILE_TRANSFER_TIMEOUT,
  )
  return usb_control.RspExportLogs.decode(rspPayload)
}

export async function deleteLogs(
  token: string,
  logType: string,
  startTime: number,
  endTime: number,
): Promise<void> {
  const cmd = usb_control.CmdDeleteLogs.fromObject({
    sessionToken: token,
    logType,
    startTime,
    endTime,
  })
  const payload = usb_control.CmdDeleteLogs.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_DELETE_LOGS, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}
```

- [ ] **Step 8: 实现 system-service**

`client/src/renderer/services/system-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import {
  sendCommand,
  handleCommonResponse,
  FILE_TRANSFER_TIMEOUT,
} from './send-command'

const MSG_CMD_GET_SYSTEM_INFO = 0x0500
const MSG_CMD_UPLOAD_SYSTEM_UPGRADE = 0x0502
const MSG_CMD_UPLOAD_VIRUSDB_UPGRADE = 0x0503
const MSG_CMD_UPDATE_DEVICE_DESC = 0x0504

export async function getSystemInfo(
  token: string,
): Promise<usb_control.RspSystemInfo> {
  const cmd = usb_control.CmdGetSystemInfo.fromObject({ sessionToken: token })
  const payload = usb_control.CmdGetSystemInfo.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_GET_SYSTEM_INFO, payload)
  return usb_control.RspSystemInfo.decode(rspPayload)
}

export async function uploadSystemUpgrade(
  token: string,
  upgradeData: Uint8Array,
  targetVersion: string,
  sha256Checksum: string,
): Promise<void> {
  const cmd = usb_control.CmdUploadSystemUpgrade.fromObject({
    sessionToken: token,
    upgradeData,
    targetVersion,
    sha256Checksum,
  })
  const payload = usb_control.CmdUploadSystemUpgrade.encode(cmd).finish()
  const rspPayload = await sendCommand(
    MSG_CMD_UPLOAD_SYSTEM_UPGRADE,
    payload,
    FILE_TRANSFER_TIMEOUT,
  )
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function uploadVirusdbUpgrade(
  token: string,
  upgradeData: Uint8Array,
  targetVersion: string,
  sha256Checksum: string,
): Promise<void> {
  const cmd = usb_control.CmdUploadVirusdbUpgrade.fromObject({
    sessionToken: token,
    upgradeData,
    targetVersion,
    sha256Checksum,
  })
  const payload = usb_control.CmdUploadVirusdbUpgrade.encode(cmd).finish()
  const rspPayload = await sendCommand(
    MSG_CMD_UPLOAD_VIRUSDB_UPGRADE,
    payload,
    FILE_TRANSFER_TIMEOUT,
  )
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function updateDeviceDescription(
  token: string,
  description: string,
): Promise<void> {
  const cmd = usb_control.CmdUpdateDeviceDesc.fromObject({
    sessionToken: token,
    description,
  })
  const payload = usb_control.CmdUpdateDeviceDesc.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_UPDATE_DEVICE_DESC, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}
```

- [ ] **Step 9: 实现 user-service**

`client/src/renderer/services/user-service.ts`：
```typescript
import { usb_control } from '../../shared/proto/usb_control'
import { sendCommand, handleCommonResponse } from './send-command'

const MSG_CMD_LIST_USERS = 0x0600
const MSG_CMD_CREATE_USER = 0x0602
const MSG_CMD_DELETE_USER = 0x0603
const MSG_CMD_RESET_PASSWORD = 0x0604
const MSG_CMD_CHANGE_PASSWORD = 0x0605

export async function listUsers(
  token: string,
): Promise<usb_control.RspListUsers> {
  const cmd = usb_control.CmdListUsers.fromObject({ sessionToken: token })
  const payload = usb_control.CmdListUsers.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_LIST_USERS, payload)
  return usb_control.RspListUsers.decode(rspPayload)
}

export async function createUser(
  token: string,
  username: string,
  role: string,
  password: string,
  confirmPassword: string,
): Promise<void> {
  const cmd = usb_control.CmdCreateUser.fromObject({
    sessionToken: token,
    username,
    role,
    password,
    confirmPassword,
  })
  const payload = usb_control.CmdCreateUser.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_CREATE_USER, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function deleteUser(
  token: string,
  username: string,
): Promise<void> {
  const cmd = usb_control.CmdDeleteUser.fromObject({
    sessionToken: token,
    username,
  })
  const payload = usb_control.CmdDeleteUser.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_DELETE_USER, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function resetPassword(
  token: string,
  username: string,
  newPassword: string,
  confirmPassword: string,
): Promise<void> {
  const cmd = usb_control.CmdResetPassword.fromObject({
    sessionToken: token,
    username,
    newPassword,
    confirmPassword,
  })
  const payload = usb_control.CmdResetPassword.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_RESET_PASSWORD, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}

export async function changePassword(
  token: string,
  oldPassword: string,
  newPassword: string,
  confirmPassword: string,
): Promise<void> {
  const cmd = usb_control.CmdChangePassword.fromObject({
    sessionToken: token,
    oldPassword,
    newPassword,
    confirmPassword,
  })
  const payload = usb_control.CmdChangePassword.encode(cmd).finish()
  const rspPayload = await sendCommand(MSG_CMD_CHANGE_PASSWORD, payload)
  const rsp = usb_control.RspCommon.decode(rspPayload)
  handleCommonResponse(rsp)
}
```

- [ ] **Step 10: 提交**

```bash
git add client/src/renderer/services/
git commit -m "feat(client): 实现 Services 层（8 个业务 service + 通用发送函数）

- send-command.ts: 通用 sendCommand()、handleCommonResponse()、ServiceError
- auth-service: login/logout/queryAuthStatus/getMachineCode/uploadLicense
- whitelist-service: listWhitelist/addWhitelist/removeWhitelist/updateWhitelist
- device-service: getConnectedDevices
- file-policy-service: getFilePolicy/updateSwitch/addBlacklistExtension/removeBlacklistExtension
- policy-service: exportPolicy/importPolicy
- log-service: queryLogs/exportLogs/deleteLogs
- system-service: getSystemInfo/uploadSystemUpgrade/uploadVirusdbUpgrade/updateDeviceDescription
- user-service: listUsers/createUser/deleteUser/resetPassword/changePassword
- 文件传输类命令使用 300s 超时，普通命令使用 15s 超时
- 组件不直接调 IPC，全部通过 services 层调用
"
```

---

### Task 12: 公共组件

**Files:**
- Create: `client/src/renderer/components/DataTable.vue`
- Create: `client/src/renderer/components/ConfirmDialog.vue`
- Create: `client/src/renderer/components/ProgressDialog.vue`
- Create: `client/src/renderer/components/ConnectionAlert.vue`
- Create: `client/src/renderer/components/ChangePasswordDialog.vue`
- Create: `client/src/renderer/utils/password-validator.ts`

**Interfaces:**
- Consumes: `useConnectionStore()` (Task 8), `useSessionStore()` (Task 8), `changePassword()` (Task 11)
- Produces: 5 个公共组件 + 密码校验工具，供业务页面使用

- [ ] **Step 1: 实现 DataTable**

`client/src/renderer/components/DataTable.vue`：
```vue
<script setup lang="ts">
export interface DataTableColumn {
  prop: string
  label: string
  width?: number | string
  minWidth?: number | string
  fixed?: boolean | 'left' | 'right'
  sortable?: boolean
  slot?: string
}

interface Props {
  columns: DataTableColumn[]
  data: unknown[]
  loading: boolean
  error?: string
  total: number
  page: number
  pageSize: number
  emptyText?: string
}

const props = withDefaults(defineProps<Props>(), {
  error: '',
  emptyText: '暂无数据',
})

const emit = defineEmits<{
  (event: 'page-change', page: number): void
  (event: 'page-size-change', pageSize: number): void
}>()

function handleCurrentChange(page: number): void {
  emit('page-change', page)
}

function handleSizeChange(size: number): void {
  emit('page-size-change', size)
}
</script>

<template>
  <div class="data-table-wrapper">
    <div v-if="props.error" class="table-error">
      {{ props.error }}
    </div>

    <el-table
      v-else
      v-loading="props.loading"
      :data="props.data"
      border
      stripe
      style="width: 100%"
    >
      <el-table-column
        v-for="col in props.columns"
        :key="col.prop"
        :prop="col.prop"
        :label="col.label"
        :width="col.width"
        :min-width="col.minWidth"
        :fixed="col.fixed"
        :sortable="col.sortable"
      >
        <template v-if="col.slot" #default="scope">
          <slot :name="col.slot" v-bind="scope" />
        </template>
      </el-table-column>

      <template #empty>
        <span>{{ props.emptyText }}</span>
      </template>
    </el-table>

    <div v-if="props.total > 0" class="table-pagination">
      <el-pagination
        :current-page="props.page"
        :page-size="props.pageSize"
        :total="props.total"
        :page-sizes="[20, 50, 100]"
        layout="total, sizes, prev, pager, next, jumper"
        @current-change="handleCurrentChange"
        @size-change="handleSizeChange"
      />
    </div>
  </div>
</template>

<style scoped>
.data-table-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.table-error {
  padding: 20px;
  text-align: center;
  color: #f56c6c;
}

.table-pagination {
  display: flex;
  justify-content: flex-end;
}
</style>
```

- [ ] **Step 2: 实现 ConfirmDialog**

`client/src/renderer/components/ConfirmDialog.vue`：
```vue
<script setup lang="ts">
interface Props {
  visible: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'warning' | 'danger' | 'info'
}

const props = withDefaults(defineProps<Props>(), {
  confirmText: '确定',
  cancelText: '取消',
  type: 'warning',
})

const emit = defineEmits<{
  (event: 'confirm'): void
  (event: 'cancel'): void
  (event: 'update:visible', value: boolean): void
}>()

function handleConfirm(): void {
  emit('confirm')
  emit('update:visible', false)
}

function handleCancel(): void {
  emit('cancel')
  emit('update:visible', false)
}

function handleClose(): void {
  emit('update:visible', false)
}
</script>

<template>
  <el-dialog
    :model-value="props.visible"
    :title="props.title"
    width="420px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <span>{{ props.message }}</span>
    <template #footer>
      <el-button @click="handleCancel">{{ props.cancelText }}</el-button>
      <el-button
        :type="props.type === 'danger' ? 'danger' : 'primary'"
        @click="handleConfirm"
      >
        {{ props.confirmText }}
      </el-button>
    </template>
  </el-dialog>
</template>
```

- [ ] **Step 3: 实现 ProgressDialog**

`client/src/renderer/components/ProgressDialog.vue`：
```vue
<script setup lang="ts">
interface Props {
  visible: boolean
  title: string
  message?: string
}

const props = withDefaults(defineProps<Props>(), {
  message: '正在处理，请稍候...',
})
</script>

<template>
  <el-dialog
    :model-value="props.visible"
    :title="props.title"
    width="360px"
    :close-on-click-modal="false"
    :show-close="false"
  >
    <div class="progress-content">
      <el-icon class="is-loading" :size="24">
        <Loading />
      </el-icon>
      <span>{{ props.message }}</span>
    </div>
  </el-dialog>
</template>

<style scoped>
.progress-content {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
}
</style>
```

- [ ] **Step 4: 实现 ConnectionAlert**

`client/src/renderer/components/ConnectionAlert.vue`：
```vue
<script setup lang="ts">
import { useConnectionStore } from '@/stores/connection'

const connection = useConnectionStore()
</script>

<template>
  <el-alert
    v-if="connection.wasConnected && connection.isDisconnected"
    title="USB 管控装置已断开连接，请检查网络或设备连接。"
    type="error"
    :closable="false"
    show-icon
    class="connection-alert"
  />
</template>

<style scoped>
.connection-alert {
  margin-bottom: 12px;
}
</style>
```

- [ ] **Step 5: 实现密码复杂度校验工具**

`client/src/renderer/utils/password-validator.ts`：
```typescript
export interface PasswordValidationResult {
  valid: boolean
  message: string
}

const MIN_LENGTH = 8

export function validatePasswordComplexity(password: string): PasswordValidationResult {
  if (password.length < MIN_LENGTH) {
    return { valid: false, message: `密码长度不能少于${MIN_LENGTH}位` }
  }

  let categoryCount = 0
  if (/[a-zA-Z]/.test(password)) {
    categoryCount++
  }
  if (/\d/.test(password)) {
    categoryCount++
  }
  if (/[^a-zA-Z0-9]/.test(password)) {
    categoryCount++
  }

  if (categoryCount < 2) {
    return { valid: false, message: '密码至少包含字母、数字、特殊字符中的两种' }
  }

  return { valid: true, message: '' }
}
```

- [ ] **Step 6: 实现 ChangePasswordDialog**

`client/src/renderer/components/ChangePasswordDialog.vue`：
```vue
<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { useSessionStore } from '@/stores/session'
import { changePassword } from '@/services/user-service'
import { validatePasswordComplexity } from '@/utils/password-validator'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const session = useSessionStore()
const formRef = ref<FormInstance | null>(null)
const submitting = ref(false)

const formModel = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: '',
})

const passwordComplexityValidator = (_rule: unknown, value: string, callback: (error?: Error) => void) => {
  if (value === '') {
    callback()
    return
  }
  const result = validatePasswordComplexity(value)
  if (!result.valid) {
    callback(new Error(result.message))
    return
  }
  callback()
}

const confirmPasswordValidator = (_rule: unknown, value: string, callback: (error?: Error) => void) => {
  if (value !== '' && value !== formModel.newPassword) {
    callback(new Error('两次输入的密码不一致'))
    return
  }
  callback()
}

const rules: FormRules = {
  oldPassword: [
    { required: true, message: '请输入旧密码', trigger: 'blur' },
  ],
  newPassword: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { validator: passwordComplexityValidator, trigger: 'blur' },
  ],
  confirmPassword: [
    { required: true, message: '请再次输入新密码', trigger: 'blur' },
    { validator: confirmPasswordValidator, trigger: 'blur' },
  ],
}

watch(() => props.visible, (val) => {
  if (!val) {
    formModel.oldPassword = ''
    formModel.newPassword = ''
    formModel.confirmPassword = ''
    formRef.value?.resetFields()
  }
})

function handleClose(): void {
  emit('update:visible', false)
}

async function handleSubmit(): Promise<void> {
  if (formRef.value == null) {
    return
  }

  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) {
    return
  }

  submitting.value = true
  try {
    await changePassword(
      session.token,
      formModel.oldPassword,
      formModel.newPassword,
      formModel.confirmPassword,
    )
    ElMessage.success('密码修改成功')
    handleClose()
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : '密码修改失败'
    ElMessage.error(message)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    title="修改密码"
    width="420px"
    :close-on-click-modal="false"
    @update:model-value="emit('update:visible', $event)"
  >
    <el-form
      ref="formRef"
      :model="formModel"
      :rules="rules"
      label-width="80px"
    >
      <el-form-item label="旧密码" prop="oldPassword">
        <el-input
          v-model="formModel.oldPassword"
          type="password"
          show-password
          placeholder="请输入旧密码"
        />
      </el-form-item>
      <el-form-item label="新密码" prop="newPassword">
        <el-input
          v-model="formModel.newPassword"
          type="password"
          show-password
          placeholder="请输入新密码"
        />
      </el-form-item>
      <el-form-item label="确认密码" prop="confirmPassword">
        <el-input
          v-model="formModel.confirmPassword"
          type="password"
          show-password
          placeholder="请再次输入新密码"
        />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" :loading="submitting" @click="handleSubmit">确定</el-button>
    </template>
  </el-dialog>
</template>
```

- [ ] **Step 7: 提交**

```bash
git add client/src/renderer/components/ client/src/renderer/utils/password-validator.ts
git commit -m "feat(client): 实现公共组件（DataTable/ConfirmDialog/ProgressDialog/ConnectionAlert/ChangePasswordDialog）

- DataTable: el-table + el-pagination 封装，支持 loading/empty/error 状态、分页、插槽列
- ConfirmDialog: v-model:visible 双向绑定，支持 warning/danger/info 类型
- ProgressDialog: 不可关闭的加载弹窗，用于文件传输类指令防重复提交
- ConnectionAlert: 读取 connection store，曾经连接过且当前 DISCONNECTED 时展示错误提示条
- ChangePasswordDialog: 修改密码弹窗，含密码复杂度校验（至少8位，至少2种字符类型）
- password-validator: 密码复杂度校验工具函数
"
```

---

### Task 13: Element Plus 集成

**Files:**
- Modify: `client/src/renderer/main.ts` (注册 Element Plus)
- Create: `client/src/renderer/styles/element-overrides.scss` (可选)

**Interfaces:**
- Consumes: Element Plus 2.8.x
- Produces: 全局注册 Element Plus 组件和样式

- [ ] **Step 1: 安装 Element Plus 依赖**

```bash
cd client
npm install element-plus @element-plus/icons-vue
npm install -D unplugin-auto-import unplugin-vue-components
```

- [ ] **Step 2: 配置 auto-import**

修改 `client/electron.vite.config.ts`，为 renderer 添加 Element Plus auto-import 插件：

```typescript
import { resolve } from 'path'
import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
  },
  renderer: {
    resolve: {
      alias: {
        '@': resolve('src/renderer'),
      },
    },
    plugins: [
      vue(),
      AutoImport({
        resolvers: [ElementPlusResolver()],
      }),
      Components({
        resolvers: [ElementPlusResolver()],
      }),
    ],
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@use "@/styles/variables" as *;`,
        },
      },
    },
  },
})
```

- [ ] **Step 3: 注册 Element Plus 图标**

修改 `client/src/renderer/main.ts`：
```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import router from './router'
import App from './App.vue'
import 'element-plus/dist/index.css'

const app = createApp(App)
const pinia = createPinia()

for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.use(pinia)
app.use(router)
app.mount('#app')
```

- [ ] **Step 4: 验证 Element Plus 组件可用**

```bash
cd client
npm run dev
```

Expected: 应用启动无报错，公共组件中的 `el-table`、`el-dialog`、`el-pagination`、`el-alert`、`el-button` 正常渲染。

- [ ] **Step 5: 提交**

```bash
git add client/electron.vite.config.ts client/src/renderer/main.ts \
       client/package.json client/package-lock.json
git commit -m "feat(client): 集成 Element Plus 2.8.x

- 安装 element-plus 和 @element-plus/icons-vue
- 配置 unplugin-auto-import 和 unplugin-vue-components 按需导入
- 全局注册 Element Plus 图标组件
- 导入 Element Plus 样式
- 配置 SCSS additionalData 全局注入 variables
"
```

---

### Task 14: 冒烟验证与收尾

**Files:**
- Verify all created files compile and link correctly
- Verify `npm run dev` starts without errors

- [ ] **Step 1: TypeScript 编译检查**

```bash
cd client
npx tsc --noEmit
```

Expected: 无编译错误。如有类型错误，逐个修复。

- [ ] **Step 2: 运行全部单元测试**

```bash
npx vitest run
```

Expected: 全部 PASS。覆盖 frame-codec、connection-state、request-dispatcher、heartbeat、stores。

- [ ] **Step 3: 开发服务器启动验证**

```bash
npm run dev
```

手动验证：
1. 应用窗口启动，自动跳转到 `#/login`
2. 登录页全屏显示（无 Sidebar/Header）
3. DevTools 中无 JS 错误
4. `window.desktopApi` 对象存在且含 `tls`、`dialog` 命名空间

- [ ] **Step 4: 代码风格检查**

```bash
npx eslint src/ --ext .ts,.vue
npx prettier --check "src/**/*.{ts,vue,scss}"
```

Expected: 无风格错误。

- [ ] **Step 5: 提交收尾**

```bash
git add -A
git commit -m "chore(client): 冒烟验证与收尾

- 确认 TypeScript 编译无错误
- 确认全部单元测试通过
- 确认开发服务器启动正常
- 确认 ESLint 和 Prettier 检查通过
"
```

---

## Self-Review Checklist

### 1. Spec 覆盖检查

| Spec 章节 | Plan Task |
|---|---|
| §2 目录结构 | Task 1 |
| §3 技术栈 | Task 1 (package.json) |
| §4 Proto 管线 | Task 3 |
| §5.1-5.3 frame-codec + state machine | Task 4 |
| §5.4-5.5 dispatcher + heartbeat | Task 5 |
| §5.2 tls-transport + facade + IPC | Task 6 |
| §6 Preload | Task 7 |
| §7 Pinia Stores | Task 8 |
| §8 路由与守卫 | Task 9 |
| §9.1-9.4 布局与设计变量 | Task 10 |
| §10 Services 层 | Task 11 |
| §11 结果码映射 | Task 2 |
| §12 公共组件 | Task 12 |
| §13 冒烟联调 | Task 14 |
| §14 测试策略 | Task 4, 5, 8 |
| §15 约束清单 | Global Constraints |

### 2. 接口一致性检查

- `ConnectionStatus` 类型在 Task 2 定义，Task 7 (preload)、Task 8 (stores)、Task 9 (router) 一致引用 `../../shared/connection-state`
- `UserRole` 类型在 Task 2 定义，Task 8 (session store)、Task 9 (router)、Task 10 (MainLayout) 一致引用
- `ResultCode` 在 Task 2 定义，Task 11 (services) 通过 `getResultMessage()` 使用
- `IpcChannels` 在 Task 2 定义，Task 6 (tls-ipc)、Task 7 (dialog-ipc) 一致使用
- `window.desktopApi` 在 Task 7 定义类型，Task 8 (connection store)、Task 11 (services) 调用
- `useSessionStore()` / `useConnectionStore()` 在 Task 8 定义，Task 9 (router guard)、Task 10 (MainLayout)、Task 12 (ConnectionAlert/ChangePasswordDialog) 调用

### 3. 架构对齐修正记录

以下问题在 review 环节发现并已在本计划中修正：

1. **Session store 补充 login/logout 方法** — `login(ip, username, password)` 封装 TLS 连接 + CMD_LOGIN 发送 + setSession，`logout()` 封装 CMD_LOGOUT 发送 + clearSession + disconnect，与 `05-数据流设计.md` §3/§4 对齐
2. **handleLogout 调用 session.logout()** — MainLayout 中 `handleLogout()` 通过 `session.logout()` 发送 CMD_LOGOUT 到装置端再清会话，而非仅 `clearSession()`
3. **会话超时断开 TCP + 跳转登录页** — `startInactivityTimer` 回调中增加 `connection.disconnect()`，与 `07-权限模型.md` §7 "管理端跳转登录页并断开 TCP 连接" 对齐
4. **Protobuf `.create()` → `.fromObject()`** — `--no-create` 标志下生成代码不含 `.create()` 方法，全部 service 已统一使用 `.fromObject()`
5. **changePassword 移至 user-service.ts** — CMD_CHANGE_PASSWORD (0x0605) 属于用户管理消息段 (0x0600-0x06FF)，从 auth-service 移至 user-service
6. **MainLayout 添加用户下拉菜单** — 替换原有 logout 按钮为 el-dropdown，包含「修改密码」和「登出」两项，与原型图 P09 对齐
7. **ChangePasswordDialog 组件** — 新增修改密码弹窗组件，含表单校验和 changePassword service 调用
8. **密码复杂度校验器** — `password-validator.ts` 实现 8 位最低长度 + 至少 2 种字符类型规则，与 `07-权限模型.md` §6 对齐
9. **重连会话校验** — connection store 增加 `reconnect()` 方法，session store 增加 `validateSession()` 方法，与 `05-数据流设计.md` §7 对齐
10. **移除 DEVICE_HAS_USB_CONNECTED** — 该结果码已在 commit 4cac19e 从装置端代码移除，plan 同步移除
11. **添加 DEVICE_UNAUTHORIZED 映射** — 0x0102 在 RESULT_CODE_MESSAGES 中映射为 null，getResultMessage 特殊处理（静默返回 null，由调用方跳转授权页）
12. **ConnectionAlert 条件优化** — 增加 `wasConnected` 标志，仅在曾经连接过且当前断开时展示提示，避免初始状态误展示
13. **ConnectionAlert 文案修正** — 修正为"USB 管控装置已断开连接，请检查网络或设备连接。"

### 4. 已知限制

- Proto 生成步骤（Task 3）依赖 `protobufjs-cli` 可执行文件在 Windows 上正常运行
- 冒烟联调（Task 14）仅覆盖启动验证，完整 MockDeviceServer 联调属于后续 P 阶段
- `sendCommand` 当前接收 `(msgType, payload: Uint8Array, timeout)` 三参数签名；如后续需要统一为更高层的泛型封装可在实施阶段调整
