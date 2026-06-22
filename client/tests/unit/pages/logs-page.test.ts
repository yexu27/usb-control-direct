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
  props: ['columns', 'data', 'total', 'page', 'pageSize', 'loading', 'error'],
  emits: ['page-change', 'page-size-change'],
  setup(props, { slots }) {
    return () => h('section', { 'data-testid': 'logs-table' }, [
      slots.filters?.(),
      h('div', { 'data-testid': 'columns' }, props.columns.map((column: { label: string }) => column.label).join('|')),
      props.data.map((row: { id: string; content: string }) =>
        h('div', { key: row.id, 'data-testid': 'log-row' }, row.content),
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
        ElInput: { template: '<input data-testid="keyword-input" />' },
        ElDatePicker: { template: '<input data-testid="date-picker" />' },
        ElSelect: { template: '<select><slot /></select>' },
        ElOption: { template: '<option>{{ label }}</option>', props: ['label', 'value'] },
        ElButton: { template: '<button type="button" @click="$emit(\'click\')"><slot /></button>' },
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
        eventType: 'mapped',
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
      logCategory: '',
      actionType: '',
    }))
    expect(wrapper.get('[data-testid="columns"]').text()).toContain('设备名称')
    expect(wrapper.get('[data-testid="columns"]').text()).toContain('序列号')
    expect(wrapper.get('[data-testid="columns"]').text()).toContain('事件类型')
    expect(wrapper.text()).toContain('USB审计日志')
    expect(wrapper.text()).not.toContain('刷新')
    expect(wrapper.text()).not.toContain('重置')
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
})
