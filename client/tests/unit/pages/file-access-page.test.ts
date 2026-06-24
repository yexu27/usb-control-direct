// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h, nextTick } from 'vue'
import { ElMessageBox } from 'element-plus'
import { usb_control } from '../../../src/shared/proto/usb_control'
import FileAccessPage from '../../../src/renderer/pages/FileAccessPage.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useFilePolicyStore } from '../../../src/renderer/stores/file-policy'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { showErrorDialog, showSuccessToast } from '../../../src/renderer/utils/operation-feedback'

vi.mock('element-plus', () => ({
  ElMessage: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
  ElMessageBox: { confirm: vi.fn() },
}))

vi.mock('../../../src/renderer/utils/operation-feedback', () => ({
  showSuccessToast: vi.fn(),
  showErrorDialog: vi.fn().mockResolvedValue(undefined),
  errorMessage: (error: unknown, fallback: string) => error instanceof Error ? error.message : fallback,
}))

interface PageVm {
  handleAdd: (entry: { extension: string; description: string }) => Promise<void>
  handleRemove: (extension: string) => Promise<void>
}

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

const ElCheckboxStub = defineComponent({
  inheritAttrs: false,
  props: {
    modelValue: { type: Boolean, required: true },
  },
  emits: ['change'],
  setup(props, { attrs, emit }) {
    return () => h('button', {
      ...attrs,
      'data-checked': String(props.modelValue),
      onClick: () => emit('change', !props.modelValue),
    })
  },
})

const AddBlacklistDialogStub = defineComponent({
  name: 'AddBlacklistDialog',
  props: {
    visible: { type: Boolean, required: true },
    submitting: { type: Boolean, required: true },
    errorMessage: { type: String, default: '' },
  },
  emits: ['update:visible', 'submit'],
  template: `
    <div
      data-testid="add-dialog"
      :data-visible="visible"
      :data-submitting="submitting"
    >{{ errorMessage }}</div>
  `,
})

function seed(): void {
  useSessionStore().setSession({
    token: 'session-token',
    username: 'operator',
    role: 'operator',
    authStatus: 'authorized',
    authExpireTime: 0,
    deviceDescription: '',
  })
  const connection = useConnectionStore()
  connection.updateStatus('CONNECTED')
  connection.wasConnected = true
  useFilePolicyStore().policy = usb_control.RspFilePolicy.fromObject({
    execControlEnabled: false,
    autoReadControlEnabled: true,
    fileTypeBlacklistEnabled: false,
    blacklist: [
      { extension: '.doc', description: '文档', isDefault: true },
      ...Array.from({ length: 20 }, (_, index) => ({
        extension: `.x${index}`, description: `类型 ${index}`, isDefault: false,
      })),
    ],
  })
}

function mountPage() {
  return mount(FileAccessPage, {
    global: {
      stubs: {
        ConnectionAlert: { template: '<aside data-testid="connection-alert" />' },
        AddBlacklistDialog: AddBlacklistDialogStub,
        ElCard: { template: '<section><slot /></section>' },
        ElCheckbox: ElCheckboxStub,
        ElButton: { template: '<button><slot /></button>' },
        DataTable: {
          name: 'DataTable',
          props: ['data', 'page', 'pageSize', 'columns'],
          emits: ['page-change', 'page-size-change'],
          template: `
            <div data-testid="blacklist-table">
              <slot name="filters" />
              <div v-for="row in data" :key="row.extension" data-testid="blacklist-row">
                <span>{{ row.extension }}</span>
                <slot name="actions" :row="row" />
              </div>
              <button data-testid="blacklist-page-2" @click="$emit('page-change', 2)">2</button>
              <button data-testid="blacklist-page-size-50" @click="$emit('page-size-change', 50)">
                50
              </button>
            </div>
          `,
        },
      },
    },
  })
}

describe('FileAccessPage', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    seed()
  })

  it('三个策略卡片在共同容器中单列全宽纵向排列', () => {
    const wrapper = mountPage()
    const policyList = wrapper.get('[data-testid="file-policy-list"]')
    const sections = wrapper.findAll('[data-testid="file-policy-section"]')

    expect(sections.map((section) => section.attributes('data-policy'))).toEqual([
      'exec_control', 'auto_read_control', 'file_type_blacklist_control',
    ])
    expect(policyList.classes()).toContain('policy-list')
    expect(policyList.classes()).not.toContain('grid-2')
    expect(sections.every((section) => section.element.parentElement === policyList.element))
      .toBe(true)
    expect(sections.every((section) => section.classes().includes('policy-section'))).toBe(true)
    const cards = wrapper.findAll('[data-testid="file-policy-card"]')
    expect(cards).toHaveLength(3)
    expect(cards.every((card) => card.classes().includes('policy-card'))).toBe(true)
    expect(wrapper.findAll('.app-checkbox-row')).toHaveLength(3)
    expect(cards.map((card) => card.text())).toEqual([
      expect.stringContaining('可执行程序访问控制'),
      expect.stringContaining('介质自动读取功能控制'),
      expect.stringContaining('文件类型访问控制'),
    ])
    expect(wrapper.findAll('[data-testid="executable-type"]').map((item) => item.text())).toEqual([
      'dll', 'exe', 'PE', 'ELF',
    ])
    expect(wrapper.get('[data-testid="blacklist-panel"]').text()).toContain('文件类型黑名单')
    expect(wrapper.text()).not.toContain('23种')
    const connectionAlert = wrapper.find('[data-testid="connection-alert"]')
    expect(connectionAlert.element.previousElementSibling?.tagName).toBe('HEADER')
    expect(wrapper.text()).not.toContain('操作员')
  })

  it('展示 PRD 可执行类型与页面说明', () => {
    const wrapper = mountPage()

    expect(wrapper.text()).toContain('管理移动存储设备的文件访问策略')
    expect(wrapper.text()).toContain('可执行程序访问控制')
    expect(wrapper.text()).toContain('介质自动读取功能控制')
    expect(wrapper.text()).toContain('文件类型访问控制')
    expect(wrapper.text()).toContain('文件类型黑名单')
    expect(wrapper.findAll('[data-testid="executable-type"]').map((item) => item.text())).toEqual([
      'dll', 'exe', 'PE', 'ELF',
    ])
  })

  it.each([
    ['exec-control-switch', '可执行程序访问控制', 'exec_control', true],
    ['auto-read-control-switch', '介质自动读取控制', 'auto_read_control', false],
    [
      'blacklist-control-switch',
      '文件类型黑名单控制',
      'file_type_blacklist_control',
      true,
    ],
  ] as const)('%s 通过中文 aria-label 与精确 policy key 更新', async (
    testId,
    ariaLabel,
    key,
    enabled,
  ) => {
    const store = useFilePolicyStore()
    const setSwitch = vi.spyOn(store, 'setSwitch').mockResolvedValue()
    const wrapper = mountPage()
    const policySwitch = wrapper.get(`[data-testid="${testId}"]`)

    expect(policySwitch.attributes('aria-label')).toBe(ariaLabel)
    await policySwitch.trigger('click')
    await flushPromises()

    expect(setSwitch).toHaveBeenCalledWith('session-token', key, enabled)
    expect(showSuccessToast).not.toHaveBeenCalled()
  })

  it('远端开关更新失败时受控 checked 保持不变', async () => {
    const store = useFilePolicyStore()
    vi.spyOn(store, 'setSwitch').mockRejectedValue(new Error('更新失败'))
    const wrapper = mountPage()
    const execSwitch = wrapper.get('[data-testid="exec-control-switch"]')

    expect(execSwitch.attributes('data-checked')).toBe('false')
    await execSwitch.trigger('click')
    await flushPromises()
    expect(execSwitch.attributes('data-checked')).toBe('false')
    expect(store.policy?.execControlEnabled).toBe(false)
    expect(showErrorDialog).toHaveBeenCalledWith('开关修改失败', '更新失败')
  })

  it('断线时保留 store 数据且禁止写操作', async () => {
    const connection = useConnectionStore()
    connection.updateStatus('DISCONNECTED')
    const store = useFilePolicyStore()
    const setSwitch = vi.spyOn(store, 'setSwitch')
    const wrapper = mountPage()

    expect(wrapper.text()).toContain('.doc')
    const execSwitch = wrapper.get('[data-testid="exec-control-switch"]')
    expect(execSwitch.attributes('disabled')).toBeUndefined()
    await execSwitch.trigger('click')
    await flushPromises()
    expect(setSwitch).not.toHaveBeenCalled()
    expect(showErrorDialog).toHaveBeenCalledWith('操作失败', '装置已断开连接，无法修改策略')
    expect(execSwitch.attributes('data-checked')).toBe('false')
    expect(store.policy?.blacklist).toHaveLength(21)
  })

  it('断线时仍可打开添加弹窗，提交时警告并保留弹窗', async () => {
    useConnectionStore().updateStatus('DISCONNECTED')
    const store = useFilePolicyStore()
    const add = vi.spyOn(store, 'addExtension')
    const wrapper = mountPage()

    const trigger = wrapper.get('[data-testid="add-blacklist-trigger"]')
    expect(trigger.attributes('disabled')).toBeUndefined()
    await trigger.trigger('click')
    const dialog = wrapper.getComponent(AddBlacklistDialogStub)
    expect(dialog.props('visible')).toBe(true)
    dialog.vm.$emit('submit', { extension: '.zip', description: '压缩包' })
    await flushPromises()

    expect(showErrorDialog).toHaveBeenCalledWith('操作失败', '装置已断开连接，无法修改策略')
    expect(add).not.toHaveBeenCalled()
    expect(dialog.props('visible')).toBe(true)
    expect(dialog.props('submitting')).toBe(false)
  })

  it('断线时删除仍可点击，但仅警告且不弹确认不调用 store', async () => {
    useConnectionStore().updateStatus('DISCONNECTED')
    const store = useFilePolicyStore()
    const remove = vi.spyOn(store, 'removeExtension')
    const wrapper = mountPage()
    const deleteButton = wrapper.get('[data-extension=".doc"]')

    expect(deleteButton.attributes('disabled')).toBeUndefined()
    await deleteButton.trigger('click')
    await flushPromises()

    expect(showErrorDialog).toHaveBeenCalledWith('操作失败', '装置已断开连接，无法修改策略')
    expect(ElMessageBox.confirm).not.toHaveBeenCalled()
    expect(remove).not.toHaveBeenCalled()
  })

  it('同一后缀确认与删除期间始终只允许一个操作', async () => {
    const confirmDeferred = createDeferred<string>()
    const removeDeferred = createDeferred<void>()
    vi.mocked(ElMessageBox.confirm).mockReturnValue(confirmDeferred.promise)
    const store = useFilePolicyStore()
    const remove = vi.spyOn(store, 'removeExtension').mockReturnValue(removeDeferred.promise)
    const wrapper = mountPage()
    const deleteButton = wrapper.get('[data-extension=".doc"]')

    const firstClick = deleteButton.trigger('click')
    const secondClick = deleteButton.trigger('click')
    await Promise.all([firstClick, secondClick])
    expect(ElMessageBox.confirm).toHaveBeenCalledTimes(1)
    expect(remove).not.toHaveBeenCalled()
    expect(deleteButton.attributes('disabled')).toBeDefined()
    expect(deleteButton.attributes('loading')).toBeDefined()

    confirmDeferred.resolve('confirm')
    await flushPromises()
    expect(remove).toHaveBeenCalledTimes(1)
    expect(deleteButton.attributes('disabled')).toBeDefined()
    expect(deleteButton.attributes('loading')).toBeDefined()
    await deleteButton.trigger('click')
    expect(ElMessageBox.confirm).toHaveBeenCalledTimes(1)
    expect(remove).toHaveBeenCalledTimes(1)
    expect(showSuccessToast).not.toHaveBeenCalled()

    removeDeferred.resolve()
    await flushPromises()
    expect(showSuccessToast).toHaveBeenCalledTimes(1)
    expect(deleteButton.attributes('disabled')).toBeUndefined()
  })

  it('删除确认期间断线时不调用 store，释放占有后可重试', async () => {
    const confirmDeferred = createDeferred<string>()
    vi.mocked(ElMessageBox.confirm).mockReturnValueOnce(confirmDeferred.promise)
      .mockResolvedValueOnce('confirm')
    const store = useFilePolicyStore()
    const remove = vi.spyOn(store, 'removeExtension').mockResolvedValue()
    const wrapper = mountPage()
    const deleteButton = wrapper.get('[data-extension=".doc"]')

    await deleteButton.trigger('click')
    expect(deleteButton.attributes('disabled')).toBeDefined()
    useConnectionStore().updateStatus('DISCONNECTED')
    confirmDeferred.resolve('confirm')
    await flushPromises()

    expect(showErrorDialog).toHaveBeenCalledWith('操作失败', '装置已断开连接，无法修改策略')
    expect(remove).not.toHaveBeenCalled()
    expect(showSuccessToast).not.toHaveBeenCalled()
    expect(deleteButton.attributes('disabled')).toBeUndefined()

    useConnectionStore().updateStatus('CONNECTED')
    await deleteButton.trigger('click')
    await flushPromises()
    expect(ElMessageBox.confirm).toHaveBeenCalledTimes(2)
    expect(remove).toHaveBeenCalledTimes(1)
  })

  it('不同后缀的删除确认可以独立进行', async () => {
    const firstConfirm = createDeferred<string>()
    const secondConfirm = createDeferred<string>()
    vi.mocked(ElMessageBox.confirm)
      .mockReturnValueOnce(firstConfirm.promise)
      .mockReturnValueOnce(secondConfirm.promise)
    const store = useFilePolicyStore()
    const remove = vi.spyOn(store, 'removeExtension').mockResolvedValue()
    const wrapper = mountPage()

    await wrapper.get('[data-extension=".doc"]').trigger('click')
    await wrapper.get('[data-extension=".x0"]').trigger('click')
    expect(ElMessageBox.confirm).toHaveBeenCalledTimes(2)

    firstConfirm.resolve('confirm')
    secondConfirm.resolve('confirm')
    await flushPromises()
    expect(remove).toHaveBeenCalledTimes(2)
  })

  it('删除失败无成功提示，释放占有后可重试', async () => {
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm')
    const removeDeferred = createDeferred<void>()
    const store = useFilePolicyStore()
    const remove = vi.spyOn(store, 'removeExtension')
      .mockReturnValueOnce(removeDeferred.promise)
      .mockResolvedValueOnce()
    const wrapper = mountPage()
    const deleteButton = wrapper.get('[data-extension=".doc"]')

    await deleteButton.trigger('click')
    removeDeferred.reject(new Error('删除失败'))
    await flushPromises()
    expect(showSuccessToast).not.toHaveBeenCalled()
    expect(showErrorDialog).toHaveBeenCalledWith('黑名单删除失败', '删除失败')
    expect(deleteButton.attributes('disabled')).toBeUndefined()

    await deleteButton.trigger('click')
    await flushPromises()
    expect(ElMessageBox.confirm).toHaveBeenCalledTimes(2)
    expect(remove).toHaveBeenCalledTimes(2)
    expect(showSuccessToast).toHaveBeenCalledTimes(1)
  })

  it('取消删除确认后释放占有并可再次点击', async () => {
    vi.mocked(ElMessageBox.confirm)
      .mockRejectedValueOnce(new Error('取消'))
      .mockResolvedValueOnce('confirm')
    const store = useFilePolicyStore()
    const remove = vi.spyOn(store, 'removeExtension').mockResolvedValue()
    const wrapper = mountPage()
    const deleteButton = wrapper.get('[data-extension=".doc"]')

    await deleteButton.trigger('click')
    await flushPromises()
    expect(remove).not.toHaveBeenCalled()
    expect(deleteButton.attributes('disabled')).toBeUndefined()

    await deleteButton.trigger('click')
    await flushPromises()
    expect(ElMessageBox.confirm).toHaveBeenCalledTimes(2)
    expect(remove).toHaveBeenCalledTimes(1)
  })

  it('黑名单使用 DataTable 且每页 20 条，默认项也显示删除', () => {
    const wrapper = mountPage()
    const table = wrapper.getComponent({ name: 'DataTable' })

    expect(table.props('pageSize')).toBe(20)
    expect(table.props('columns').map((column: { label: string }) => column.label)).toEqual([
      '后缀名', '说明', '操作',
    ])
    const deleteButtons = wrapper.findAll('[data-extension]')
    expect(deleteButtons).toHaveLength(20)
    expect(deleteButtons[0].attributes('data-extension')).toBe('.doc')
  })

  it('切换到第二页后仅展示超过首页 20 条的内容', async () => {
    const wrapper = mountPage()

    expect(wrapper.text()).toContain('.x18')
    expect(wrapper.text()).not.toContain('.x19')
    await wrapper.get('[data-testid="blacklist-page-2"]').trigger('click')
    await flushPromises()

    expect(wrapper.getComponent({ name: 'DataTable' }).props('page')).toBe(2)
    expect(wrapper.text()).toContain('.x19')
    expect(wrapper.text()).not.toContain('.x18')
    expect(wrapper.findAll('[data-testid="blacklist-row"]')).toHaveLength(1)
  })

  it('使用 DataTable 现有 page-size-change API 切换条数并重置到第一页', async () => {
    const wrapper = mountPage()
    await wrapper.get('[data-testid="blacklist-page-2"]').trigger('click')
    await wrapper.get('[data-testid="blacklist-page-size-50"]').trigger('click')
    await flushPromises()
    const table = wrapper.getComponent({ name: 'DataTable' })

    expect(table.props('pageSize')).toBe(50)
    expect(table.props('page')).toBe(1)
    expect(wrapper.findAll('[data-testid="blacklist-row"]')).toHaveLength(21)
  })

  it('添加请求防重入，远端成功前不关闭不提示', async () => {
    const deferred = createDeferred<void>()
    const store = useFilePolicyStore()
    const add = vi.spyOn(store, 'addExtension').mockReturnValue(deferred.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-blacklist-trigger"]').trigger('click')
    const dialog = wrapper.getComponent(AddBlacklistDialogStub)
    const entry = { extension: '.zip', description: '压缩包' }

    dialog.vm.$emit('submit', entry)
    dialog.vm.$emit('submit', entry)
    await nextTick()

    expect(add).toHaveBeenCalledTimes(1)
    expect(dialog.props('visible')).toBe(true)
    expect(dialog.props('submitting')).toBe(true)
    expect(showSuccessToast).not.toHaveBeenCalled()

    deferred.resolve()
    await flushPromises()
    expect(dialog.props('visible')).toBe(false)
    expect(dialog.props('submitting')).toBe(false)
    expect(showSuccessToast).toHaveBeenCalledWith(
      '修改成功，重新拔插或重新映射后生效',
    )
  })

  it('添加失败后保留弹窗并恢复可提交状态', async () => {
    const deferred = createDeferred<void>()
    const store = useFilePolicyStore()
    vi.spyOn(store, 'addExtension').mockReturnValue(deferred.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-blacklist-trigger"]').trigger('click')
    const dialog = wrapper.getComponent(AddBlacklistDialogStub)

    dialog.vm.$emit('submit', { extension: '.zip', description: '压缩包' })
    await nextTick()
    expect(dialog.props('visible')).toBe(true)
    expect(dialog.props('submitting')).toBe(true)

    deferred.reject(new Error('添加失败'))
    await flushPromises()

    expect(dialog.props('visible')).toBe(true)
    expect(dialog.props('submitting')).toBe(false)
    expect(showSuccessToast).not.toHaveBeenCalled()
    expect(dialog.props('errorMessage')).toBe('添加失败')
    expect(wrapper.text()).toContain('添加失败')
  })

  it('添加和删除均通过 store，删除前必须确认', async () => {
    const store = useFilePolicyStore()
    const add = vi.spyOn(store, 'addExtension').mockResolvedValue()
    const remove = vi.spyOn(store, 'removeExtension').mockResolvedValue()
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm')
    const component = mountPage().vm as unknown as PageVm

    await component.handleAdd({ extension: '.zip', description: '压缩包' })
    await component.handleRemove('.doc')

    expect(add).toHaveBeenCalledWith('session-token', '.zip', '压缩包')
    expect(ElMessageBox.confirm).toHaveBeenCalled()
    expect(remove).toHaveBeenCalledWith('session-token', '.doc')
    expect(showSuccessToast).toHaveBeenCalledWith(
      '修改成功，重新拔插或重新映射后生效',
    )
  })
})
