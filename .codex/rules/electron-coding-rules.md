# Electron 编码规范

本文档用于约束开发人员和 AI 编程助手编写、修改和重构 Electron 代码时的行为。文档采用“章节 / 说明 / 规范条目 / 示例”的结构，便于作为团队编码规范使用。

参考来源取向：Electron 官方安全建议、Electron API 文档、Chromium 安全模型、Node.js 工程实践和桌面端应用常见工程约定。

## 0. 规范说明

### 0.1 适用范围

本规范适用于 Electron 主进程、preload 脚本、渲染进程集成代码、IPC 接口、窗口管理、原生能力封装、打包配置和 Electron 端到端测试代码。

如果项目已有更具体的 Electron 版本约束、安全配置、目录结构、打包配置或测试规范，必须优先使用项目已有规范。

### 0.2 总体原则

### 进程边界清晰。

说明：主进程负责系统能力和窗口生命周期，preload 负责暴露最小桥接接口，渲染进程负责界面交互。不同进程职责不得混写。

### 最小权限。

说明：渲染进程默认不应拥有 Node.js、文件系统、子进程或 shell 能力。需要原生能力时，必须通过受控 IPC 接口调用。

### IPC 契约稳定。

说明：IPC 是进程间接口，必须像公共 API 一样设计、命名、类型化和测试。

### 安全默认。

说明：新增窗口、WebView、外部链接、文件访问和原生能力调用必须默认选择更安全的配置。

## 1. 目录与进程组织

### 不同进程代码必须分离

- 主进程代码必须独立于渲染进程代码。

- preload 脚本必须独立放置，不得与页面组件或主进程业务逻辑混写。

- 共享类型可以放在独立模块中，但共享模块不得依赖 Electron 主进程 API 或浏览器 DOM API，除非模块名称和用途明确。

- 主进程模块应按窗口管理、IPC 注册、原生能力、应用生命周期等职责拆分。

- 不要把所有 Electron 逻辑写在一个入口文件中。

示例：

```text
electron/
  main/
    app-lifecycle.ts
    create-main-window.ts
    ipc/
      file-dialog-ipc.ts
  preload/
    index.ts
  shared/
    ipc-contract.ts
```

### Electron 代码命名必须体现进程和能力边界

- 主进程文件和函数命名必须体现 Electron 能力或生命周期。

示例：

```text
create-main-window.ts
register-app-menu.ts
register-file-dialog-ipc.ts
```

- preload 暴露给渲染进程的对象名称必须表达受控原生能力，不要使用过宽的名称。

错误示例：

```ts
contextBridge.exposeInMainWorld('electron', api);
```

正确示例：

```ts
contextBridge.exposeInMainWorld('desktopApi', api);
```

- IPC channel 常量必须使用稳定名称，变量名使用 `camelCase`，常量值使用清晰的命名空间格式。

示例：

```ts
const dialogOpenFileChannel = 'dialog:open-file';
const appGetVersionChannel = 'app:get-version';
```

- IPC handler 函数使用 `handleXxx` 命名，注册函数使用 `registerXxxIpc` 命名。

示例：

```ts
function registerDialogIpc() {
  ipcMain.handle('dialog:open-file', handleOpenFileDialog);
}
```

- BrowserWindow 实例变量必须体现窗口用途，不要使用 `win`、`window` 作为长期变量名。

错误示例：

```ts
let win: BrowserWindow | null = null;
```

正确示例：

```ts
let mainWindow: BrowserWindow | null = null;
```

- 局部变量必须表达 Electron 对象语义，例如 `preloadPath`、`appMenu`、`selectedFilePaths`、`targetWebContents`。

### Electron 控制流必须保护生命周期

- 使用 BrowserWindow、webContents、Tray、Menu 等对象前，必须确认对象仍有效。

示例：

```ts
if (mainWindow == null || mainWindow.isDestroyed()) {
  return;
}

mainWindow.webContents.send('app:ready');
```

- 主进程事件处理函数必须使用提前返回处理无效状态，避免深层嵌套。

示例：

```ts
function focusMainWindow() {
  if (mainWindow == null) {
    createMainWindow();
    return;
  }

  if (mainWindow.isMinimized()) {
    mainWindow.restore();
  }

  mainWindow.focus();
}
```

- IPC handler 中的 `if` 必须显式处理参数非法、窗口销毁、用户取消和底层失败等分支。

- `for` 循环中处理多个窗口或多个 webContents 时，必须跳过已销毁对象。

示例：

```ts
for (const browserWindow of BrowserWindow.getAllWindows()) {
  if (browserWindow.isDestroyed()) {
    continue;
  }

  browserWindow.webContents.send('theme:changed', theme);
}
```

- 需要串行处理系统资源时可以在 `for...of` 循环中使用 `await`；可以并发处理的任务应显式使用 `Promise.all`。

- `switch` 处理应用状态、窗口类型或 IPC 动作时，必须覆盖所有已定义状态，并在默认分支拒绝未知状态。

## 2. BrowserWindow

### 窗口配置必须安全

- 创建窗口时必须显式配置 `webPreferences`。

- 渲染进程默认禁用 Node.js 集成。

- 必须启用上下文隔离。

- 不要在生产窗口中启用不必要的远程调试能力。

示例：

```ts
const mainWindow = new BrowserWindow({
  width: 1200,
  height: 800,
  webPreferences: {
    nodeIntegration: false,
    contextIsolation: true,
    sandbox: true,
    preload: preloadPath,
  },
});
```

- `preload` 路径必须使用可靠的路径构造方式，不要拼接未校验字符串。

- 窗口引用必须有明确生命周期管理，避免窗口关闭后继续访问已销毁对象。

- 多窗口应用必须明确每类窗口的创建、复用、聚焦和销毁策略。

## 3. preload

### preload 只能暴露最小 API

- preload 必须使用 `contextBridge.exposeInMainWorld` 暴露受控接口。

- 禁止把 `ipcRenderer`、`require`、`process` 或 Node.js 原始能力直接暴露给渲染进程。

错误示例：

```ts
contextBridge.exposeInMainWorld('electron', {
  ipcRenderer,
});
```

正确示例：

```ts
contextBridge.exposeInMainWorld('nativeApi', {
  openFile: (options: OpenFileOptions) => ipcRenderer.invoke('dialog:open-file', options),
});
```

- 暴露接口必须有 TypeScript 类型声明。

- preload 不应包含复杂业务逻辑，只做参数收敛、IPC 调用和结果返回。

- preload 中注册的事件监听必须提供取消订阅函数。

示例：

```ts
contextBridge.exposeInMainWorld('nativeApi', {
  onThemeChanged: (listener: (theme: ThemeName) => void) => {
    const channel = 'theme:changed';
    const handler = (_event: IpcRendererEvent, theme: ThemeName) => listener(theme);

    ipcRenderer.on(channel, handler);

    return () => {
      ipcRenderer.removeListener(channel, handler);
    };
  },
});
```

## 4. IPC

### IPC channel 必须稳定且白名单化

- IPC channel 名称必须集中定义为常量或类型，不要散落字符串。

- channel 命名必须表达方向和动作。

示例：

```ts
export const IpcChannels = {
  dialogOpenFile: 'dialog:open-file',
  appGetVersion: 'app:get-version',
} as const;
```

- 禁止使用用户输入拼接 channel 名称。

- 主进程必须集中注册 IPC handler。

- `ipcMain.handle` 的处理函数必须校验参数。

- IPC handler 必须校验消息来源。需要限制来源时，应检查 `event.senderFrame` 或等价上下文，确保消息来自预期窗口、页面或 origin。

- IPC 返回值必须可结构化克隆，不要返回类实例、函数、DOM 对象、复杂原生句柄或不可序列化对象。

- 可预期失败必须返回明确结果或抛出受控错误，不要把底层异常原样透传给渲染进程。

示例：

```ts
ipcMain.handle(IpcChannels.dialogOpenFile, async (_event, input: unknown) => {
  const options = parseOpenFileOptions(input);
  const result = await dialog.showOpenDialog(options);

  return {
    canceled: result.canceled,
    filePaths: result.filePaths,
  };
});
```

需要限制来源的 handler 必须显式接收并校验事件对象：

```ts
ipcMain.handle(IpcChannels.dialogOpenFile, async (event, input: unknown) => {
  assertTrustedSender(event.senderFrame);

  const options = parseOpenFileOptions(input);
  const result = await dialog.showOpenDialog(options);

  return {
    canceled: result.canceled,
    filePaths: result.filePaths,
  };
});
```

### IPC 事件必须可清理

- 渲染进程订阅主进程事件时，preload API 必须返回取消订阅函数。

- 主进程向窗口发送事件前必须检查窗口是否已销毁。

示例：

```ts
if (!mainWindow.isDestroyed()) {
  mainWindow.webContents.send(IpcChannels.appGetVersion, version);
}
```

## 5. 原生能力封装

### 文件系统和系统能力只能在主进程使用

- 渲染进程不得直接访问 `fs`、`path`、`child_process`、`shell` 等 Node.js 或 Electron 原生能力。

- 文件选择、保存、读取、写入、拖拽文件处理等能力必须由主进程或受控 preload API 封装。

- 路径处理必须使用 `path` 模块，不要手工拼接路径分隔符。

- 处理用户选择的路径时，必须进行参数校验和错误处理。

- 不要把未校验的用户输入传给 shell、子进程或系统命令。

### shell 与外部链接必须受控

- 打开外部链接必须校验协议和目标。

- 禁止让未校验 URL 进入 `shell.openExternal`。

- 应拦截新窗口打开行为，避免任意页面获得应用上下文。

示例：

```ts
mainWindow.webContents.setWindowOpenHandler(({ url }) => {
  if (isAllowedExternalUrl(url)) {
    shell.openExternal(url).catch((error) => {
      logger.warn('failed to open external url', error);
    });
  }

  return { action: 'deny' };
});
```

## 6. 安全配置

### 安全选项必须显式

- `nodeIntegration` 默认必须为 `false`。

- `contextIsolation` 必须为 `true`。

- 能启用 `sandbox` 的窗口应启用 `sandbox`。

- 禁止使用已废弃或高风险的 `remote` 模块。

- 禁止加载不可信远程内容，除非项目已有明确安全设计。

- 生产环境不得默认打开 DevTools。

- Content Security Policy 应由项目统一配置。

- 不要禁用证书校验来绕过开发或测试问题。

## 7. 应用生命周期

### 生命周期处理必须跨平台清晰

- `app.whenReady()` 后再创建窗口。

- `window-all-closed`、`activate`、`before-quit` 等事件必须按平台行为处理。

- 退出前需要释放的资源必须集中管理。

- 应用单实例逻辑必须明确，避免多个实例同时操作同一资源。

- 长耗时任务不得阻塞主进程事件循环。

## 8. 菜单、托盘与快捷键

### 全局入口必须集中管理

- 菜单、托盘和全局快捷键必须在主进程集中注册和注销。

- 全局快捷键必须在应用退出时释放。

- 菜单动作不得直接包含复杂业务逻辑，应调用命名函数。

- 上下文菜单和窗口菜单必须避免暴露调试或危险能力。

## 9. 打包与环境

### 环境差异必须显式

- 开发环境和生产环境入口必须清晰区分。

- 不要依赖当前工作目录定位资源。

- 打包后资源路径必须通过应用路径、构建注入变量或项目约定方法获取。

- 主进程代码不得依赖仅在开发服务器中存在的路径。

- 生产构建不得包含无用调试输出、测试入口和开发专用开关。

## 10. 日志与错误处理

### 主进程错误必须可定位

- 主进程异步任务必须捕获并处理错误。

- `unhandledRejection` 和 `uncaughtException` 可以作为最后防线，但不能替代局部错误处理。

- 日志不得包含敏感信息。

- 渲染进程收到错误时，应得到稳定错误结构，而不是底层异常对象。

示例：

```ts
interface IpcErrorResult {
  ok: false;
  code: string;
  message: string;
}
```

## 11. 测试代码

### Electron 行为必须端到端验证

- IPC handler 必须有单元测试或集成测试覆盖参数校验、成功结果和失败结果。

- preload 暴露 API 必须测试类型契约和订阅清理行为。

- 窗口创建逻辑必须覆盖关键 `webPreferences`。

- Electron E2E 测试必须覆盖窗口启动、主要交互、IPC 调用和原生对话框封装。

- 测试中不要依赖开发者本机的固定路径、固定窗口位置或固定屏幕尺寸。

## 12. 禁止写法

- 禁止在渲染进程直接启用 Node.js 能力。

- 禁止把 `ipcRenderer` 原样暴露给页面。

- 禁止使用用户输入拼接 IPC channel。

- 禁止在主进程入口文件堆放所有逻辑。

- 禁止把不可序列化对象作为 IPC 返回值。

- 禁止未经校验调用 `shell.openExternal`。

- 禁止使用 `remote` 模块。

- 禁止通过 shell 字符串拼接执行未校验命令。

- 禁止生产环境默认打开 DevTools。

- 禁止禁用安全校验来规避开发或测试问题。
