# Log Search Reset And Mock Filter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复日志管理页搜索交互：切换三个日志 Tab 时清空搜索条件，并让 mock 装置按查询条件真实过滤返回数据。

**Architecture:** 页面层只管理当前 Tab 的搜索状态，切换 Tab 统一恢复默认搜索条件后重新查询。协议服务层保持现有 `CmdQueryLogs` / `CmdExportLogs` 字段语义，不新增接口字段。E2E mock 装置实现与协议一致的过滤逻辑，避免 mock 手工测试时“点搜索表格不变”。

**Tech Stack:** Electron 32, Vue 3.4, Pinia, Element Plus, TypeScript, Vitest, Playwright。

---

## Ground Rules

- macOS 只做代码编写和静态检查，不运行 `npm run typecheck`、`npm run test`、`npm run build`、`npm run test:e2e`。
- Windows 验证路径：`F:\aiCode\github\usb-control-direct\client`。
- Windows 同步方式：逐个 `scp` 文件内容到对应路径，不使用 tar/zip 打包，不使用远端 git。
- Windows 机器：`Andisec@172.16.0.219`，SSH key：`~/.ssh/WinPC-Personal`。
- 不修改协议字段，不修改后端接口语义。
- 不改变三个 Tab 的可见筛选项：
  - USB 审计日志：关键字、时间范围、USB 事件类型。
  - 恶意代码检测日志：关键字、时间范围。
  - 操作日志：关键字、时间范围、操作日志类型。
- 切换任意日志 Tab 时，搜索条件恢复默认值：
  - `keyword = ''`
  - `dateRange = createDefaultRange()`
  - `selectedEventType = ''`
  - `selectedOperationLogCategory = ''`
  - `page = 1`
  - `pageSize = PAGE_SIZE`

---

## File Structure

### Modify

- `client/src/renderer/pages/LogsPage.vue`
  - 增加 `resetSearchState()`。
  - `handleTabChange()` 调用 `resetSearchState()` 后查询当前 Tab。
  - 保持 `handleSearch()` 只刷新当前 Tab。

- `client/tests/unit/pages/logs-page.test.ts`
  - 补充 Tab 切换时清空关键字、类型筛选、页码和页大小的测试。
  - 补充 USB 审计搜索按钮参数测试。
  - 补充恶意代码日志搜索按钮参数测试。
  - 保留操作日志 `logCategory` 查询与导出测试。

- `client/tests/e2e/support/mock-device.ts`
  - 给 mock 日志数据补充多条不同类型/关键字的数据。
  - 新增并导出 `filterMockLogsForQuery()`。
  - `queryLogsResponse()` 根据 `keyword`、`eventType`、`logCategory`、`actionType` 过滤并分页。

- `client/tests/unit/e2e-support/mock-device-log-filter.test.ts`
  - 新增 mock 日志过滤单测，覆盖三类日志。

- `client/tests/e2e/admin-auditor-pages.spec.ts`
  - 修正操作日志旧断言：现在应显示“操作日志类型”列和操作日志类型筛选控件。

---

## Task 1: Reset Search State On Log Tab Switch

**Files:**
- Modify: `client/src/renderer/pages/LogsPage.vue`
- Test: `client/tests/unit/pages/logs-page.test.ts`

**Mock verification:**
- `queryLogs` 使用 `vi.fn()`。
- Element Plus `ElInput`、`ElDatePicker`、`ElSelect` stub 必须支持 `v-model`。
- 断言切换 Tab 后下一次 `queryLogs` 参数恢复默认搜索条件。

- [x] **Step 1: Update test stubs to support v-model**

In `client/tests/unit/pages/logs-page.test.ts`, ensure these stubs exist.

```ts
const ElInputStub = defineComponent({
  props: ['modelValue'],
  emits: ['update:modelValue'],
  template: '<input v-bind="$attrs" :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
})

const ElDatePickerStub = defineComponent({
  props: ['modelValue'],
  emits: ['update:modelValue'],
  template: '<input v-bind="$attrs" :value="String(modelValue)" @change="$emit(\'update:modelValue\', new Date(1767225600000))" />',
})

const ElSelectStub = defineComponent({
  props: ['modelValue'],
  emits: ['update:modelValue'],
  template: '<select v-bind="$attrs" :value="modelValue" @change="$emit(\'update:modelValue\', $event.target.value)"><slot /></select>',
})
```

Use them in `mountPage()`:

```ts
ElInput: ElInputStub,
ElDatePicker: ElDatePickerStub,
ElSelect: ElSelectStub,
ElOption: { template: '<option :value="value">{{ label }}</option>', props: ['label', 'value'] },
```

- [x] **Step 2: Write failing test for tab switch reset**

Add this test to `client/tests/unit/pages/logs-page.test.ts`.

```ts
it('切换日志类型时清空搜索条件并恢复默认分页', async () => {
  const wrapper = mountPage()
  await flushPromises()

  await wrapper.get('[data-testid="log-keyword"]').setValue('Kingston')
  await wrapper.get('[data-testid="log-event-type"]').setValue('mapped')
  wrapper.getComponent(DataTableStub).vm.$emit('page-size-change', 50)
  await flushPromises()

  await wrapper.get('[data-testid="logs-tab-operation"]').trigger('click')
  await flushPromises()

  expect(queryLogs).toHaveBeenLastCalledWith('token', expect.objectContaining({
    logType: 'operation',
    keyword: '',
    eventType: '',
    logCategory: '',
    page: 1,
    pageSize: 20,
  }))
})
```

- [x] **Step 3: Verify the new test fails on Windows**

Run in Windows PowerShell:

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test -- tests/unit/pages/logs-page.test.ts
```

Expected before implementation: FAIL because `keyword` and `pageSize` still carry over across Tab switches.

- [x] **Step 4: Implement `resetSearchState()`**

In `client/src/renderer/pages/LogsPage.vue`, add:

```ts
function resetSearchState(): void {
  dateRange.value = createDefaultRange()
  keyword.value = ''
  selectedEventType.value = ''
  selectedOperationLogCategory.value = ''
  page.value = 1
  pageSize.value = PAGE_SIZE
}
```

Update `handleTabChange()`:

```ts
function handleTabChange(tabName: string | number): void {
  activeLogType.value = tabName as LogType
  resetSearchState()
  void loadLogs()
}
```

Keep `handleSearch()` unchanged:

```ts
function handleSearch(): void {
  page.value = 1
  void loadLogs()
}
```

- [x] **Step 5: Run focused Windows test**

Run in Windows PowerShell:

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test -- tests/unit/pages/logs-page.test.ts
```

Expected: PASS.

---

## Task 2: Cover Search Parameters For All Three Log Tabs

**Files:**
- Modify: `client/tests/unit/pages/logs-page.test.ts`

**Mock verification:**
- `queryLogs` validates the exact request payload for each Tab.
- USB 审计验证 `eventType`。
- 恶意代码验证不带 `eventType` / `logCategory`。
- 操作日志验证 `logCategory`。

- [x] **Step 1: Add USB audit search parameter test**

```ts
it('USB审计日志搜索时带关键字和事件类型', async () => {
  const wrapper = mountPage()
  await flushPromises()

  await wrapper.get('[data-testid="log-keyword"]').setValue('Kingston')
  await wrapper.get('[data-testid="log-event-type"]').setValue('mapped')
  await wrapper.get('[data-testid="log-search"]').trigger('click')
  await flushPromises()

  expect(queryLogs).toHaveBeenLastCalledWith('token', expect.objectContaining({
    logType: 'usb_audit',
    keyword: 'Kingston',
    eventType: 'mapped',
    logCategory: '',
    page: 1,
  }))
})
```

- [x] **Step 2: Add malware search parameter test**

```ts
it('恶意代码检测日志搜索时只带关键字和时间条件', async () => {
  const wrapper = mountPage()
  await flushPromises()

  await wrapper.get('[data-testid="logs-tab-malware"]').trigger('click')
  await flushPromises()
  await wrapper.get('[data-testid="log-keyword"]').setValue('EICAR')
  await wrapper.get('[data-testid="log-search"]').trigger('click')
  await flushPromises()

  expect(queryLogs).toHaveBeenLastCalledWith('token', expect.objectContaining({
    logType: 'malware',
    keyword: 'EICAR',
    eventType: '',
    logCategory: '',
    page: 1,
  }))
})
```

- [x] **Step 3: Keep operation search parameter test**

Ensure this test remains in `client/tests/unit/pages/logs-page.test.ts`:

```ts
it('操作日志按类型筛选并在导出时携带相同类型', async () => {
  vi.mocked(exportLogs).mockResolvedValue({
    success: true,
    zipData: new Uint8Array([1, 2, 3]),
    suggestedFilename: 'OperationLog20260622120000.zip',
    resultCode: 0,
    errorMessage: '',
  } as never)
  vi.mocked(queryLogs).mockResolvedValue({
    success: true,
    total: 0,
    usbAuditEntries: [],
    malwareEntries: [],
    operationEntries: [],
    resultCode: 0,
    errorMessage: '',
  } as never)
  const wrapper = mountPage()
  await flushPromises()

  await wrapper.get('[data-testid="logs-tab-operation"]').trigger('click')
  await flushPromises()
  await wrapper.get('[data-testid="log-operation-category"]').setValue('user_management')
  await wrapper.get('[data-testid="log-search"]').trigger('click')
  await flushPromises()

  expect(queryLogs).toHaveBeenLastCalledWith('token', expect.objectContaining({
    logType: 'operation',
    eventType: '',
    logCategory: 'user_management',
  }))

  await wrapper.get('[data-testid="log-export"]').trigger('click')
  await flushPromises()

  expect(exportLogs).toHaveBeenCalledWith('token', expect.objectContaining({
    logType: 'operation',
    eventType: '',
    logCategory: 'user_management',
  }))
})
```

- [x] **Step 4: Run focused Windows test**

Run:

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test -- tests/unit/pages/logs-page.test.ts
```

Expected: PASS.

---

## Task 3: Make E2E Mock Device Filter Logs

**Files:**
- Modify: `client/tests/e2e/support/mock-device.ts`
- Create: `client/tests/unit/e2e-support/mock-device-log-filter.test.ts`

**Mock verification:**
- 直接测试 `filterMockLogsForQuery()`。
- 覆盖 USB 审计、恶意代码、操作日志三类数据。

- [x] **Step 1: Write failing mock filter tests**

Create `client/tests/unit/e2e-support/mock-device-log-filter.test.ts`.

```ts
import { describe, expect, it } from 'vitest'
import { filterMockLogsForQuery } from '../../e2e/support/mock-device'

const usbAuditLogs = [{
  id: 1,
  deviceName: 'Kingston DataTraveler',
  deviceSn: 'USB-AUDIT-001',
  eventType: 'mapped',
  detail: '白名单设备映射成功',
}, {
  id: 2,
  deviceName: 'Blocked USB',
  deviceSn: 'USB-AUDIT-002',
  eventType: 'whitelist_denied',
  detail: '未授权设备禁止接入',
}]

const malwareLogs = [{
  id: 1,
  deviceName: 'SanDisk',
  deviceSn: 'USB-MALWARE-001',
  virusName: 'EICAR-Test-File',
  filePath: '/mnt/usb/eicar.com',
  detail: '发现病毒并阻断',
}, {
  id: 2,
  deviceName: 'Clean USB',
  deviceSn: 'USB-MALWARE-002',
  virusName: '',
  filePath: '/mnt/usb/readme.txt',
  detail: '扫描通过',
}]

const operationLogs = [{
  id: 1,
  username: 'admin',
  role: 'admin',
  logCategory: 'user_management',
  actionType: 'user_create',
  target: 'new_operator',
  detail: '新建用户成功',
}, {
  id: 2,
  username: 'audit',
  role: 'auditor',
  logCategory: 'login_auth',
  actionType: 'login',
  target: 'audit',
  detail: '审计员登录成功',
}]

describe('mock-device log filter', () => {
  it('按 USB 事件类型和关键字过滤 USB 审计日志', () => {
    const result = filterMockLogsForQuery(usbAuditLogs, {
      keyword: 'Blocked',
      eventType: 'whitelist_denied',
      logCategory: '',
      actionType: '',
    })

    expect(result.map((entry) => entry.id)).toEqual([2])
  })

  it('按关键字过滤恶意代码检测日志', () => {
    const result = filterMockLogsForQuery(malwareLogs, {
      keyword: 'EICAR',
      eventType: '',
      logCategory: '',
      actionType: '',
    })

    expect(result.map((entry) => entry.id)).toEqual([1])
  })

  it('按操作日志类型和关键字过滤操作日志', () => {
    const result = filterMockLogsForQuery(operationLogs, {
      keyword: '审计员',
      eventType: '',
      logCategory: 'login_auth',
      actionType: '',
    })

    expect(result.map((entry) => entry.id)).toEqual([2])
  })
})
```

- [x] **Step 2: Verify the mock filter test fails on Windows**

Run:

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test -- tests/unit/e2e-support/mock-device-log-filter.test.ts
```

Expected before implementation: FAIL with `filterMockLogsForQuery is not a function`.

- [x] **Step 3: Implement filter helper**

In `client/tests/e2e/support/mock-device.ts`, add:

```ts
interface MockLogFilter {
  keyword: string
  eventType: string
  logCategory: string
  actionType: string
}

function stringifyLogValue(value: unknown): string {
  if (value == null) {
    return ''
  }
  if (value instanceof Uint8Array) {
    return ''
  }
  return String(value)
}

function matchesKeyword(entry: Record<string, unknown>, keyword: string): boolean {
  const normalizedKeyword = keyword.trim().toLowerCase()
  if (normalizedKeyword === '') {
    return true
  }
  return Object.values(entry).some((value) =>
    stringifyLogValue(value).toLowerCase().includes(normalizedKeyword),
  )
}

export function filterMockLogsForQuery<TEntry extends object>(
  entries: TEntry[],
  filter: MockLogFilter,
): TEntry[] {
  return entries.filter((entry) => {
    const logEntry = entry as Record<string, unknown>
    if (!matchesKeyword(logEntry, filter.keyword)) {
      return false
    }
    if (filter.eventType !== '' && logEntry.eventType !== filter.eventType) {
      return false
    }
    if (filter.logCategory !== '' && logEntry.logCategory !== filter.logCategory) {
      return false
    }
    if (filter.actionType !== '' && logEntry.actionType !== filter.actionType) {
      return false
    }
    return true
  })
}
```

- [x] **Step 4: Add pagination helper**

In `client/tests/e2e/support/mock-device.ts`, add:

```ts
function paginateMockLogs<TEntry>(entries: TEntry[], page: number, pageSize: number): TEntry[] {
  const safePage = Number.isFinite(page) && page > 0 ? Math.trunc(page) : 1
  const safePageSize = Number.isFinite(pageSize) && pageSize > 0 ? Math.trunc(pageSize) : entries.length
  const start = (safePage - 1) * safePageSize
  return entries.slice(start, start + safePageSize)
}
```

- [x] **Step 5: Add diverse mock data**

Ensure `usbAuditLogs`, `malwareLogs`, and `operationLogs` each have at least two records with different keywords/types.

```ts
private usbAuditLogs = [{
  id: 1,
  eventTime: 1_767_225_610,
  deviceSn: 'USB-AUDIT-001',
  deviceName: 'Kingston DataTraveler',
  eventType: 'mapped',
  result: 'allowed',
  detail: '白名单设备映射成功',
}, {
  id: 2,
  eventTime: 1_767_225_611,
  deviceSn: 'USB-AUDIT-002',
  deviceName: 'Blocked USB',
  eventType: 'whitelist_denied',
  result: 'denied',
  detail: '未授权设备禁止接入',
}]
```

```ts
private malwareLogs = [{
  id: 1,
  scanTime: 1_767_225_620,
  deviceSn: 'USB-MALWARE-001',
  deviceName: 'SanDisk',
  filePath: '/mnt/usb/eicar.com',
  scanResult: 'infected',
  virusName: 'EICAR-Test-File',
  processResult: 'blocked',
  detail: '发现病毒并阻断',
}, {
  id: 2,
  scanTime: 1_767_225_621,
  deviceSn: 'USB-MALWARE-002',
  deviceName: 'Clean USB',
  filePath: '/mnt/usb/readme.txt',
  scanResult: 'clean',
  virusName: '',
  processResult: 'allowed',
  detail: '扫描通过',
}]
```

```ts
private operationLogs = [{
  id: 1,
  opTime: 1_767_225_630,
  username: 'admin',
  role: 'admin',
  logCategory: 'user_management',
  actionType: 'user_create',
  target: 'new_operator',
  result: '0',
  detail: '新建用户成功',
}, {
  id: 2,
  opTime: 1_767_225_631,
  username: 'audit',
  role: 'auditor',
  logCategory: 'login_auth',
  actionType: 'login',
  target: 'audit',
  result: '0',
  detail: '审计员登录成功',
}]
```

- [x] **Step 6: Wire filter into `queryLogsResponse()`**

Replace `queryLogsResponse()` body logic with:

```ts
private queryLogsResponse(payload: Uint8Array): MockResponse {
  const command = usb_control.CmdQueryLogs.decode(payload)
  const filter = {
    keyword: command.keyword,
    eventType: command.eventType,
    logCategory: command.logCategory,
    actionType: command.actionType,
  }
  const usbAuditEntries = command.logType === 'usb_audit'
    ? filterMockLogsForQuery(this.usbAuditLogs, filter)
    : []
  const malwareEntries = command.logType === 'malware'
    ? filterMockLogsForQuery(this.malwareLogs, filter)
    : []
  const operationEntries = command.logType === 'operation'
    ? filterMockLogsForQuery(this.operationLogs, filter)
    : []

  return {
    msgType: 0x0401,
    messageClass: usb_control.RspQueryLogs,
    body: {
      success: true,
      total: this.filteredLogCount(command.logType, {
        usbAuditEntries,
        malwareEntries,
        operationEntries,
      }),
      usbAuditEntries: paginateMockLogs(usbAuditEntries, command.page, command.pageSize),
      malwareEntries: paginateMockLogs(malwareEntries, command.page, command.pageSize),
      operationEntries: paginateMockLogs(operationEntries, command.page, command.pageSize),
      resultCode: 0,
      errorMessage: '',
    },
  }
}
```

Replace `logCount()` with:

```ts
private filteredLogCount(
  logType: string,
  entries: {
    usbAuditEntries: unknown[]
    malwareEntries: unknown[]
    operationEntries: unknown[]
  },
): number {
  if (logType === 'usb_audit') {
    return entries.usbAuditEntries.length
  }
  if (logType === 'malware') {
    return entries.malwareEntries.length
  }
  if (logType === 'operation') {
    return entries.operationEntries.length
  }
  return 0
}
```

- [x] **Step 7: Run mock filter test on Windows**

Run:

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test -- tests/unit/e2e-support/mock-device-log-filter.test.ts
```

Expected: PASS.

---

## Task 4: Update Log E2E Contract

**Files:**
- Modify: `client/tests/e2e/admin-auditor-pages.spec.ts`

- [x] **Step 1: Replace stale operation log assertion**

Find this old assertion:

```ts
await expect(page.getByText('操作日志类型')).toHaveCount(0)
```

Replace it with:

```ts
await expect(page.getByRole('columnheader', { name: '操作日志类型' })).toBeVisible()
await expect(page.getByTestId('log-operation-category')).toBeVisible()
```

Keep this assertion unchanged:

```ts
await expect(page.getByTestId('log-event-type')).toHaveCount(0)
```

- [x] **Step 2: Run Windows E2E**

If port `127.0.0.1:9600` is occupied, inspect first:

```powershell
netstat -ano | Select-String ':9600'
Get-Process -Id (Get-NetTCPConnection -LocalPort 9600 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique) -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,Path
```

Only if the process is a stale `node.exe` mock-device/test process, stop that exact PID:

```powershell
Stop-Process -Id <PID> -Force
```

Run:

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test:e2e -- tests/e2e/admin-auditor-pages.spec.ts
```

Expected: PASS, `6 passed`.

---

## Task 5: Windows Verification

- [x] **Step 1: Sync files to Windows without tar/git**

Use `scp` per file:

```bash
scp -i ~/.ssh/WinPC-Personal client/src/renderer/pages/LogsPage.vue Andisec@172.16.0.219:F:/aiCode/github/usb-control-direct/client/src/renderer/pages/LogsPage.vue
scp -i ~/.ssh/WinPC-Personal client/tests/unit/pages/logs-page.test.ts Andisec@172.16.0.219:F:/aiCode/github/usb-control-direct/client/tests/unit/pages/logs-page.test.ts
scp -i ~/.ssh/WinPC-Personal client/tests/e2e/support/mock-device.ts Andisec@172.16.0.219:F:/aiCode/github/usb-control-direct/client/tests/e2e/support/mock-device.ts
scp -i ~/.ssh/WinPC-Personal client/tests/unit/e2e-support/mock-device-log-filter.test.ts Andisec@172.16.0.219:F:/aiCode/github/usb-control-direct/client/tests/unit/e2e-support/mock-device-log-filter.test.ts
scp -i ~/.ssh/WinPC-Personal client/tests/e2e/admin-auditor-pages.spec.ts Andisec@172.16.0.219:F:/aiCode/github/usb-control-direct/client/tests/e2e/admin-auditor-pages.spec.ts
```

- [x] **Step 2: Run typecheck on Windows**

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run typecheck
```

Expected: exit code 0.

- [x] **Step 3: Run focused unit tests on Windows**

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test -- tests/unit/pages/logs-page.test.ts tests/unit/utils/log-display.test.ts tests/unit/e2e-support/mock-device-log-filter.test.ts
```

Expected: PASS.

- [x] **Step 4: Run existing focused regression set on Windows**

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test -- tests/unit/pages/system-page.test.ts tests/unit/pages/usb-devices-page.test.ts tests/unit/utils/log-display.test.ts tests/unit/pages/logs-page.test.ts tests/unit/stores/session.test.ts tests/unit/components/connection-alert.test.ts tests/unit/pages/file-access-page.test.ts tests/unit/e2e-support/mock-device-log-filter.test.ts
```

Expected: PASS.

- [x] **Step 5: Run log-related E2E on Windows**

```powershell
cd F:\aiCode\github\usb-control-direct\client
npm run test:e2e -- tests/e2e/admin-auditor-pages.spec.ts
```

Expected: PASS, `6 passed`.

- [ ] **Step 6: Commit**

```bash
git add \
  client/src/renderer/pages/LogsPage.vue \
  client/tests/unit/pages/logs-page.test.ts \
  client/tests/e2e/support/mock-device.ts \
  client/tests/unit/e2e-support/mock-device-log-filter.test.ts \
  client/tests/e2e/admin-auditor-pages.spec.ts \
  docs/superpowers/plans/2026-06-24-log-search-reset-and-mock-filter-plan.md

git commit -m "fix(logs): 修复日志搜索条件切换和mock过滤" -m "- 切换日志类型时统一清空关键字、时间范围、类型筛选和分页条件
- 保持搜索按钮只刷新当前日志类型
- 让 E2E mock 装置按 keyword、eventType、logCategory 和 actionType 过滤日志
- 修正操作日志类型列的 E2E 契约
- 补充日志搜索状态和 mock 过滤测试"
```

---

## Self-Review

- Spec coverage:
  - Tab 切换清空搜索条件：Task 1。
  - 三个 Tab 搜索按钮只作用当前 Tab：Task 2。
  - mock 环境搜索结果真实变化：Task 3。
  - 操作日志类型列 E2E 契约：Task 4。
  - Windows 验证：Task 5。
- Placeholder scan: no `TBD`, no `TODO`, no unspecified “write tests”.
- Type consistency:
  - `selectedOperationLogCategory` matches `LogsPage.vue`.
  - `filterMockLogsForQuery()` is defined before unit tests import it.
  - `MockLogFilter` fields match protocol query fields.
