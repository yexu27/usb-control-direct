// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { ElMessageBox } from 'element-plus'
import LogsPage from '../../../src/renderer/pages/LogsPage.vue'
import { deleteLogs, exportLogs, queryLogs } from '../../../src/renderer/services/log-service'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { emitPageRefresh, resetPageRefreshListenersForTest } from '../../../src/renderer/services/page-refresh-events'

vi.mock('../../../src/renderer/services/log-service', () => ({
  queryLogs: vi.fn(),
  exportLogs: vi.fn(),
  deleteLogs: vi.fn(),
}))

vi.mock('element-plus', () => ({
  ElMessage: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
  ElMessageBox: { confirm: vi.fn(() => Promise.resolve()) },
}))

const saveFile = vi.fn()
const writeFile = vi.fn()
const revokeFileAccess = vi.fn()
let pinia: Pinia

const DataTableStub = defineComponent({
  name: 'DataTable',
  props: [
    'columns',
    'data',
    'total',
    'page',
    'pageSize',
    'loading',
    'error',
    'showDefaultPagination',
  ],
  emits: ['page-change', 'page-size-change'],
  setup(props, { slots }) {
    return () => h('section', {
      'data-testid': 'logs-table',
      'data-show-default-pagination': String(props.showDefaultPagination),
    }, [
      slots.filters?.(),
      h('div', { 'data-testid': 'columns' }, props.columns.map((column: { label: string }) => column.label).join('|')),
      props.data.map((row: {
        id: string
        content: string
        serialNumber?: string
        eventType?: string
        logCategory?: string
      }) =>
        h('div', { key: row.id, 'data-testid': 'log-row' }, [
          row.serialNumber != null ? slots.serialNumber?.({ row }) : null,
          row.eventType != null ? slots.eventType?.({ row }) : null,
          row.logCategory != null ? h('span', row.logCategory) : null,
          h('span', row.content),
        ]),
      ),
    ])
  },
})

const ElTabsStub = defineComponent({
  props: ['modelValue'],
  emits: ['tab-change'],
  setup(_props, { slots }) {
    return () => h('div', { 'data-testid': 'tabs' }, slots.default?.())
  },
})

const ElTabPaneStub = defineComponent({
  props: ['label', 'name'],
  template: '<button type="button">{{ label }}</button>',
})

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

function mountPage() {
  return mount(LogsPage, {
    global: {
      plugins: [pinia],
      stubs: {
        ConnectionAlert: { template: '<aside data-testid="connection-alert" />' },
        ProgressDialog: { props: ['visible'], template: '<div data-testid="progress" :data-visible="visible" />' },
        DataTable: DataTableStub,
        ElTabs: ElTabsStub,
        ElTabPane: ElTabPaneStub,
        ElCard: { template: '<section><slot /></section>' },
        ElInput: ElInputStub,
        ElDatePicker: ElDatePickerStub,
        ElSelect: ElSelectStub,
        ElOption: { template: '<option :value="value">{{ label }}</option>', props: ['label', 'value'] },
        ElButton: { template: '<button v-bind="$attrs" type="button" @click="$emit(\'click\')"><slot /></button>' },
        ElTag: { template: '<span><slot /></span>' },
        ElDialog: { template: '<section><slot /><slot name="footer" /></section>' },
      },
    },
  })
}

function seedStores(): void {
  const session = useSessionStore()
  session.setSession({
    token: 'token',
    username: 'audit',
    role: 'auditor',
    authStatus: 'authorized',
    authExpireTime: 0,
    deviceDescription: '',
  })
  const connection = useConnectionStore()
  connection.updateStatus('CONNECTED')
}

describe('LogsPage', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    seedStores()
    vi.clearAllMocks()
    resetPageRefreshListenersForTest()
    saveFile.mockResolvedValue({
      canceled: false,
      filePath: 'C:\\导出\\USBUsageLog20260622120000.zip',
    })
    writeFile.mockResolvedValue(undefined)
    revokeFileAccess.mockResolvedValue(undefined)
    window.desktopApi = {
      dialog: { saveFile, writeFile, revokeFileAccess },
    } as unknown as Window['desktopApi']
    vi.mocked(queryLogs).mockResolvedValue({
      success: true,
      total: 1,
      usbAuditEntries: [{
        id: 1,
        eventTime: 1_767_225_610,
        deviceName: 'Kingston',
        deviceSn: 'SN001',
        eventType: 'insert_success',
        detail: '映射成功',
      }],
      malwareEntries: [],
      operationEntries: [],
      resultCode: 0,
      errorMessage: '',
    } as never)
  })

  it('loads USB audit logs with PRD columns and no refresh/reset buttons', async () => {
    const wrapper = mountPage()

    await flushPromises()

    expect(queryLogs).toHaveBeenCalledWith('token', expect.objectContaining({
      logType: 'usb_audit',
      eventType: '',
    }))
    expect(wrapper.get('[data-testid="columns"]').text()).toContain('设备名称')
    expect(wrapper.get('[data-testid="columns"]').text()).toContain('序列号')
    expect(wrapper.get('[data-testid="columns"]').text()).toContain('插拔类型')
    expect(wrapper.get('[data-testid="columns"]').text()).not.toContain('事件类型')
    expect(wrapper.text()).toContain('USB审计日志')
    expect(wrapper.text()).not.toContain('刷新')
    expect(wrapper.text()).not.toContain('重置')
  })

  it('按确认原型渲染日志管理页框架、筛选栏、表格和分页', async () => {
    const wrapper = mountPage()

    await flushPromises()

    expect(wrapper.find('[data-testid="logs-prototype-shell"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="logs-tab-usb_audit"]').text()).toBe('USB审计日志')
    expect(wrapper.find('[data-testid="logs-tab-malware"]').text()).toBe('恶意代码检测日志')
    expect(wrapper.find('[data-testid="logs-tab-operation"]').text()).toBe('操作日志')
    expect(wrapper.find('[data-testid="logs-role-badge"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="logs-table-shell"]').attributes('data-show-default-pagination')).toBe('false')
    expect(wrapper.find('[data-testid="logs-prototype-pagination"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('显示 1-1，共 1 条')
    expect(wrapper.text()).not.toContain('Go to')
    expect(wrapper.text()).not.toContain('20/page')
  })

  it('USB审计日志按确认原型展示插拔类型和序列号标签', async () => {
    const wrapper = mountPage()

    await flushPromises()

    expect(wrapper.get('[data-testid="columns"]').text()).toContain('插拔类型')
    expect(wrapper.get('[data-testid="columns"]').text()).not.toContain('事件类型')
    expect(wrapper.find('[data-testid="log-serial-chip"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="log-event-chip"]').exists()).toBe(true)
  })

  it('USB审计日志内容列按 PRD 9.2 组合展示结构化字段', async () => {
    vi.mocked(queryLogs).mockResolvedValue({
      success: true,
      total: 5,
      usbAuditEntries: [{
        id: 1,
        eventTime: 1_767_225_610,
        deviceName: 'Kingston',
        deviceSn: 'SN001',
        deviceType: 'storage',
        interfaceType: 'mass_storage',
        eventType: 'insert_success',
        permission: 'readwrite',
        capacityBytes: 32_000_000_000,
        result: 'success',
        failReason: '',
        detail: '',
      }, {
        id: 2,
        eventTime: 1_767_225_611,
        deviceName: 'Unknown USB',
        deviceSn: '',
        deviceType: 'unsupported',
        interfaceType: 'unsupported',
        eventType: 'insert_failed',
        permission: '',
        capacityBytes: 0,
        result: 'failed',
        failReason: '不支持的设备类型',
        detail: '',
      }, {
        id: 3,
        eventTime: 1_767_225_612,
        deviceName: 'Keyboard',
        deviceSn: '',
        deviceType: 'keyboard',
        interfaceType: 'hid_keyboard',
        eventType: 'insert_success',
        permission: '',
        capacityBytes: 0,
        result: 'success',
        failReason: '',
        detail: '验证通过',
      }, {
        id: 4,
        eventTime: 1_767_225_613,
        deviceName: 'Mouse',
        deviceSn: '',
        deviceType: 'mouse',
        interfaceType: 'hid_mouse',
        eventType: 'insert_failed',
        permission: '',
        capacityBytes: 0,
        result: 'failed',
        failReason: '映射失败',
        detail: '',
      }, {
        id: 5,
        eventTime: 1_767_225_614,
        deviceName: 'Kingston',
        deviceSn: 'SN001',
        deviceType: 'storage',
        interfaceType: 'mass_storage',
        eventType: 'device_remove',
        permission: 'readonly',
        capacityBytes: 32_000_000_000,
        result: 'removed',
        failReason: '',
        detail: '',
      }],
      malwareEntries: [],
      operationEntries: [],
      resultCode: 0,
      errorMessage: '',
    } as never)

    const wrapper = mountPage()
    await flushPromises()

    const text = wrapper.text()
    expect(text).toContain('授权设备, 读写, 32GB, 映射完成')
    expect(text).toContain('不支持的 USB 设备类型, 禁止使用')
    expect(text).toContain('键盘, 验证通过, 映射完成')
    expect(text).toContain('鼠标, 映射失败')
    expect(text).toContain('授权设备, 只读')
    expect(text).not.toContain('结果：success')
    expect(text).not.toContain('结果：failed')
  })

  it('切换日志类型时清空搜索条件并恢复默认分页', async () => {
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="log-keyword"]').setValue('Kingston')
    await wrapper.get('[data-testid="log-event-type"]').setValue('insert_success')
    wrapper.getComponent(DataTableStub).vm.$emit('page-size-change', 50)
    await flushPromises()

    await wrapper.get('[data-testid="logs-tab-operation"]').trigger('click')
    await flushPromises()

    expect(queryLogs).toHaveBeenLastCalledWith('token', expect.objectContaining({
      logType: 'operation',
      keyword: '',
      eventType: '',
      page: 1,
      pageSize: 20,
    }))
  })

  it('USB审计日志搜索时带关键字和事件类型', async () => {
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="log-keyword"]').setValue('Kingston')
    await wrapper.get('[data-testid="log-event-type"]').setValue('insert_success')
    await wrapper.get('[data-testid="log-search"]').trigger('click')
    await flushPromises()

    expect(queryLogs).toHaveBeenLastCalledWith('token', expect.objectContaining({
      logType: 'usb_audit',
      keyword: 'Kingston',
      eventType: 'insert_success',
      page: 1,
    }))
  })

  it('重连成功事件后按当前筛选条件重新查询日志', async () => {
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="log-keyword"]').setValue('Kingston')
    await wrapper.get('[data-testid="log-event-type"]').setValue('insert_success')
    await wrapper.get('[data-testid="log-search"]').trigger('click')
    await flushPromises()
    vi.mocked(queryLogs).mockClear()

    emitPageRefresh('reconnect')
    await flushPromises()

    expect(queryLogs).toHaveBeenCalledWith('token', expect.objectContaining({
      logType: 'usb_audit',
      keyword: 'Kingston',
      eventType: 'insert_success',
      page: 1,
      pageSize: 20,
    }))
  })

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
      page: 1,
    }))
  })

  it('恶意代码检测日志按 scan_result 组装内容且不展示独立病毒列', async () => {
    vi.mocked(queryLogs).mockResolvedValue({
      success: true,
      total: 4,
      usbAuditEntries: [],
      malwareEntries: [{
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
        processResult: 'no_action',
        detail: '扫描通过',
      }, {
        id: 3,
        scanTime: 1_767_225_622,
        deviceSn: 'USB-MALWARE-003',
        deviceName: 'Broken USB',
        filePath: '',
        scanResult: 'failed',
        virusName: '',
        processResult: 'failed',
        failReason: '病毒库不可用',
        detail: '旧失败详情',
      }, {
        id: 4,
        scanTime: 1_767_225_623,
        deviceSn: 'USB-MALWARE-004',
        deviceName: 'Removed USB',
        filePath: '',
        scanResult: 'cancelled',
        virusName: '',
        processResult: 'no_action',
        failReason: '设备拔出',
        detail: '',
      }],
      operationEntries: [],
      resultCode: 0,
      errorMessage: '',
    } as never)

    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="logs-tab-malware"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="columns"]').text()).toBe('时间|设备名称|序列号|内容')
    expect(wrapper.text()).toContain('文件: /mnt/usb/eicar.com, 病毒: EICAR-Test-File')
    expect(wrapper.text()).toContain('未发现病毒文件')
    expect(wrapper.text()).toContain('扫描失败: 病毒库不可用')
    expect(wrapper.text()).toContain('扫描中断: 设备拔出')
    expect(wrapper.text()).not.toContain('结果：blocked')
    expect(wrapper.text()).not.toContain('发现病毒并阻断')
  })

  it('操作日志按 PRD 四列展示类型并使用结构化字段组装内容', async () => {
    vi.mocked(queryLogs).mockResolvedValue({
      success: true,
      total: 3,
      usbAuditEntries: [],
      malwareEntries: [],
      operationEntries: [{
        id: 1,
        opTime: 1_767_225_610,
        username: 'admin',
        logCategory: 'login_auth',
        actionType: 'login',
        target: 'admin',
        result: '0',
        failReason: '',
        detail: '旧 detail 不应展示',
      }, {
        id: 2,
        opTime: 1_767_225_611,
        username: 'operator',
        logCategory: 'security_config',
        actionType: 'file_policy_update',
        target: 'exec_control',
        beforeValue: '{"enabled":false}',
        afterValue: '{"enabled":true}',
        result: '0',
        failReason: '',
        detail: '',
      }, {
        id: 3,
        opTime: 1_767_225_612,
        username: 'audit',
        logCategory: 'program_upgrade',
        actionType: 'virusdb_upgrade',
        target: 'V3.0.0.3',
        result: '0',
        failReason: '',
        detail: '',
      }],
      resultCode: 0,
      errorMessage: '',
    } as never)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="logs-tab-operation"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="columns"]').text()).toBe('时间|用户|操作日志类型|内容')
    expect(wrapper.get('[data-testid="log-row"]').text()).toContain('用户登录，用户名：admin，成功')
    expect(wrapper.get('[data-testid="log-row"]').text()).toContain('登录认证')
    expect(wrapper.text()).toContain('安全配置变更')
    expect(wrapper.text()).toContain('程序升级')
    expect(wrapper.text()).toContain('修改文件访问控制策略 exec_control，关闭→开启，成功')
    expect(wrapper.text()).toContain('病毒库升级，版本升级至V3.0.0.3，成功')
    expect(wrapper.text()).not.toContain('结果：0')
    expect(wrapper.text()).not.toContain('旧 detail 不应展示')
  })

  it('操作日志不展示分类筛选且查询和导出不携带分类字段', async () => {
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
    await wrapper.get('[data-testid="log-search"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('select').exists()).toBe(false)
    expect(queryLogs).toHaveBeenLastCalledWith('token', expect.objectContaining({
      logType: 'operation',
      eventType: '',
    }))
    expect(queryLogs).toHaveBeenLastCalledWith('token', expect.not.objectContaining({
      logCategory: expect.anything(),
      actionType: expect.anything(),
    }))

    await wrapper.get('[data-testid="log-export"]').trigger('click')
    await flushPromises()

    expect(exportLogs).toHaveBeenCalledWith('token', expect.objectContaining({
      logType: 'operation',
      eventType: '',
    }))
    expect(exportLogs).toHaveBeenCalledWith('token', expect.not.objectContaining({
      logCategory: expect.anything(),
      actionType: expect.anything(),
    }))
  })

  it('exports selected log type to user selected path', async () => {
    vi.mocked(exportLogs).mockResolvedValue({
      success: true,
      zipData: new Uint8Array([1, 2, 3]),
      suggestedFilename: 'USBUsageLog20260622120000.zip',
      resultCode: 0,
      errorMessage: '',
    } as never)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="log-export"]').trigger('click')
    await flushPromises()

    expect(exportLogs).toHaveBeenCalledWith('token', expect.objectContaining({ logType: 'usb_audit' }))
    expect(saveFile).toHaveBeenCalledWith({
      title: '导出日志',
      defaultPath: 'USBUsageLog20260622120000.zip',
      filters: [{ name: '日志压缩包', extensions: ['zip'] }],
    })
    expect(writeFile).toHaveBeenCalledWith(
      'C:\\导出\\USBUsageLog20260622120000.zip',
      new Uint8Array([1, 2, 3]),
    )
  })

  it('clears logs after range dialog and second confirmation', async () => {
    vi.mocked(deleteLogs).mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="log-clear"]').trigger('click')
    expect(wrapper.find('.app-danger-block').exists()).toBe(true)
    await wrapper.get('[data-testid="log-clear-confirm"]').trigger('click')
    await flushPromises()

    expect(ElMessageBox.confirm).toHaveBeenCalled()
    expect(deleteLogs).toHaveBeenCalledWith('token', 'usb_audit', expect.any(Number), expect.any(Number))
  })

  it.each([
    'DISCONNECTED',
    'AUTH_REQUIRED',
    'LICENSE_EXPIRED',
  ] as const)('%s 状态下查询、导出、清理不请求装置', async (status) => {
    useConnectionStore().updateStatus(status)
    const wrapper = mountPage()
    await flushPromises()
    vi.mocked(queryLogs).mockClear()
    vi.mocked(exportLogs).mockClear()
    vi.mocked(deleteLogs).mockClear()

    const searchButton = wrapper.get('[data-testid="log-search"]')
    const exportButton = wrapper.get('[data-testid="log-export"]')
    const clearButton = wrapper.get('[data-testid="log-clear"]')
    expect(searchButton.attributes('disabled')).toBeDefined()
    expect(exportButton.attributes('disabled')).toBeDefined()
    expect(clearButton.attributes('disabled')).toBeDefined()

    await searchButton.trigger('click')
    await exportButton.trigger('click')
    await clearButton.trigger('click')
    await flushPromises()

    expect(queryLogs).not.toHaveBeenCalled()
    expect(exportLogs).not.toHaveBeenCalled()
    expect(deleteLogs).not.toHaveBeenCalled()
    expect(saveFile).not.toHaveBeenCalled()
  })
})
