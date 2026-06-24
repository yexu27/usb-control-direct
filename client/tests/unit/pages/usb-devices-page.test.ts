// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { usb_control } from '../../../src/shared/proto/usb_control'
import UsbDevicesPage from '../../../src/renderer/pages/UsbDevicesPage.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { useWhitelistStore } from '../../../src/renderer/stores/whitelist'
import { getConnectedDevices } from '../../../src/renderer/services/device-service'
import { listManagementUsbStorageDevices } from '../../../src/renderer/services/management-usb-service'
import { ServiceError } from '../../../src/renderer/services/send-command'
import { showErrorDialog, showSuccessToast } from '../../../src/renderer/utils/operation-feedback'

vi.mock('element-plus', () => ({
  ElMessage: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
  ElMessageBox: { confirm: vi.fn() },
}))
vi.mock('../../../src/renderer/services/device-service', () => ({ getConnectedDevices: vi.fn() }))
vi.mock('../../../src/renderer/services/management-usb-service', () => ({
  listManagementUsbStorageDevices: vi.fn(),
}))
vi.mock('../../../src/renderer/utils/operation-feedback', () => ({
  showSuccessToast: vi.fn(),
  showErrorDialog: vi.fn().mockResolvedValue(undefined),
  errorMessage: (error: unknown, fallback: string) => error instanceof Error ? error.message : fallback,
}))

const AddDialogStub = defineComponent({
  name: 'AddWhitelistDialog',
  props: ['visible', 'source', 'candidates', 'loading', 'submitting', 'errorMessage'],
  emits: ['update:visible', 'submit'],
  template: '<div data-testid="add-dialog" :data-visible="visible" :data-source="source">{{ errorMessage }}</div>',
})
const EditDialogStub = defineComponent({
  name: 'EditWhitelistDialog',
  props: ['visible', 'serialNumber', 'currentDescription', 'currentPermission', 'submitting', 'errorMessage'],
  emits: ['update:visible', 'submit'],
  template: '<div data-testid="edit-dialog" :data-visible="visible" :data-serial="serialNumber">{{ errorMessage }}</div>',
})

function seed(): void {
  useSessionStore().setSession({ token: 'token', username: 'operator', role: 'operator',
    authStatus: 'authorized', authExpireTime: 0, deviceDescription: '' })
  useConnectionStore().updateStatus('CONNECTED')
  useWhitelistStore().devices = Array.from({ length: 21 }, (_, index) =>
    usb_control.WhitelistDevice.fromObject({
      serialNumber: `SN-${index}`, vid: '0781', pid: '5591', deviceName: '隐藏名称',
      capacityBytes: 100, deviceType: 'storage', description: `说明 ${index}`,
      permission: index === 0 ? 'readonly' : 'readwrite',
      addMethod: index === 0 ? 'device' : 'management',
      createdAt: index === 0 ? 0 : index === 1 ? -1 : index === 2 ? Number.MAX_VALUE : 1_700_000_000,
    }))
}

function mountPage() {
  return mount(UsbDevicesPage, { global: { stubs: {
    ConnectionAlert: { template: '<aside data-testid="connection-alert"/>' },
    AddWhitelistDialog: AddDialogStub, EditWhitelistDialog: EditDialogStub,
    ElCard: { template: '<section v-bind="$attrs"><slot name="header"/><slot /></section>' },
    ElButton: {
      props: ['type'],
      template: '<button v-bind="$attrs" :data-button-type="type" @click="$emit(\'click\')"><slot/></button>',
    },
    DataTable: {
      name: 'DataTable',
      props: ['data', 'columns', 'error', 'total', 'page', 'pageSize'], emits: ['page-change', 'page-size-change'],
      template: `<div data-testid="table" :data-total="total" :data-page="page" :data-page-size="pageSize">
        <div v-if="error" data-testid="table-error">{{ error }}</div>
        <span data-testid="columns">{{ columns.map(c => c.label).join('|') }}</span><slot name="filters"/>
        <div v-for="row in data" :key="row.serialNumber" data-testid="row">
          <slot name="serialNumber" :row="row">{{ row.serialNumber }}</slot>
          {{ row.description }}
          <slot name="permissionLabel" :row="row">{{ row.permissionLabel }}</slot>
          {{ row.addMethodLabel }} {{ row.createdAtLabel }}
          <slot name="actions" :row="row"/>
        </div><button data-testid="page2" @click="$emit('page-change', 2)">2</button>
      </div>`,
    },
  } } })
}

const managementCandidate = {
  serialNumber: ' LOCAL-SN ', vid: 'abcd', pid: '1234', deviceName: '本机U盘',
  capacityBytes: 1000, deviceType: 'storage' as const, addable: true, unavailableReason: '',
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

describe('UsbDevicesPage', () => {
  beforeEach(() => {
    setActivePinia(createPinia()); vi.clearAllMocks(); seed()
    vi.mocked(getConnectedDevices).mockResolvedValue(usb_control.RspConnectedDevices.fromObject({ devices: [] }))
    vi.mocked(listManagementUsbStorageDevices).mockResolvedValue([])
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm' as never)
  })

  it('仅展示六个白名单列，前端分页默认20并格式化Unix秒', async () => {
    const wrapper = mountPage()
    expect(wrapper.get('[data-testid="columns"]').text()).toBe('序列号|描述|权限|添加方式|添加时间|操作')
    expect(wrapper.findAll('[data-testid="row"]')).toHaveLength(20)
    expect(wrapper.get('[data-testid="table"]').attributes('data-total')).toBe('21')
    expect(wrapper.get('[data-testid="table"]').attributes('data-page-size')).toBe('20')
    expect(wrapper.text()).toContain('--')
    expect(wrapper.text()).toMatch(/2023-11-(14|15) \d{2}:\d{2}:\d{2}/)
    expect(wrapper.text().match(/--/g)).toHaveLength(3)
    expect(wrapper.text()).not.toContain('隐藏名称')
    expect(wrapper.text()).not.toContain('VID')
    expect(wrapper.text()).not.toContain('操作员')
    await wrapper.get('[data-testid="page2"]').trigger('click')
    expect(wrapper.findAll('[data-testid="row"]')).toHaveLength(1)
    expect(useWhitelistStore().devices).toHaveLength(21)

    useWhitelistStore().devices.pop()
    await nextTick()
    expect(wrapper.get('[data-testid="table"]').attributes('data-page')).toBe('1')
    expect(wrapper.findAll('[data-testid="row"]')).toHaveLength(20)
  })

  it('展示白名单卡片标题并按原型设置按钮主次', () => {
    const wrapper = mountPage()

    expect(wrapper.get('[data-testid="usb-table-panel"]').classes()).toContain('usb-panel')
    expect(wrapper.text()).toContain('受信任普通移动存储设备白名单')
    expect(wrapper.text()).toContain('管理受信任移动存储设备白名单')
    expect(wrapper.text()).toContain('白名单设备重新插入后经过扫描审计方可使用')
    expect(wrapper.get('[data-testid="add-device-trigger"]').text()).toContain('装置端添加')
    expect(wrapper.get('[data-testid="add-management-trigger"]').text()).toContain('管理端添加')
    expect(wrapper.get('[data-testid="add-device-trigger"]').attributes('data-button-type')).toBeUndefined()
    expect(wrapper.get('[data-testid="add-management-trigger"]').attributes('data-button-type')).toBe('primary')
    expect(wrapper.text()).not.toContain('安全U盘自由使用')
  })

  it('每行展示修改入口并打开编辑弹窗', async () => {
    const wrapper = mountPage()

    await wrapper.get('[data-testid="edit-SN-0"]').trigger('click')

    const editDialog = wrapper.getComponent(EditDialogStub)
    expect(editDialog.props('visible')).toBe(true)
    expect(editDialog.props('serialNumber')).toBe('SN-0')
    expect(editDialog.props('currentDescription')).toBe('说明 0')
    expect(editDialog.props('currentPermission')).toBe('readonly')
  })

  it('提交白名单修改后调用store并提示重新拔插生效', async () => {
    const update = vi.spyOn(useWhitelistStore(), 'updateWhitelist').mockResolvedValue()
    const wrapper = mountPage()
    await wrapper.get('[data-testid="edit-SN-0"]').trigger('click')

    wrapper.getComponent(EditDialogStub).vm.$emit('submit', {
      description: '财务只读盘',
      permission: 'readwrite',
    })
    await flushPromises()

    expect(update).toHaveBeenCalledWith('token', 'SN-0', 'readwrite', '财务只读盘')
    expect(showSuccessToast).toHaveBeenCalledWith('修改成功，重新拔插后生效')
    expect(wrapper.getComponent(EditDialogStub).props('visible')).toBe(false)
  })

  it('白名单修改失败展示错误并保留弹窗', async () => {
    vi.spyOn(useWhitelistStore(), 'updateWhitelist').mockRejectedValue(new Error('权限修改失败'))
    const wrapper = mountPage()
    await wrapper.get('[data-testid="edit-SN-0"]').trigger('click')

    wrapper.getComponent(EditDialogStub).vm.$emit('submit', {
      description: '财务只读盘',
      permission: 'readwrite',
    })
    await flushPromises()

    const editDialog = wrapper.getComponent(EditDialogStub)
    expect(editDialog.props('visible')).toBe(true)
    expect(editDialog.props('errorMessage')).toBe('权限修改失败')
    expect(showSuccessToast).not.toHaveBeenCalled()
  })

  it('断线时阻止提交白名单修改', async () => {
    const update = vi.spyOn(useWhitelistStore(), 'updateWhitelist').mockResolvedValue()
    const wrapper = mountPage()
    await wrapper.get('[data-testid="edit-SN-0"]').trigger('click')
    useConnectionStore().updateStatus('DISCONNECTED')

    wrapper.getComponent(EditDialogStub).vm.$emit('submit', {
      description: '财务只读盘',
      permission: 'readwrite',
    })
    await flushPromises()

    expect(update).not.toHaveBeenCalled()
    expect(ElMessage.warning).toHaveBeenCalledWith('装置已断开连接，无法修改白名单')
  })

  it('按确认原型渲染白名单面板、标签和底部说明', async () => {
    const wrapper = mountPage()
    await flushPromises()

    expect(wrapper.text()).toContain('管理受信任移动存储设备白名单')
    expect(wrapper.find('[data-testid="usb-table-panel"]').exists()).toBe(true)
    expect(wrapper.find('.serial-chip').exists()).toBe(true)
    expect(wrapper.find('.permission-chip').exists()).toBe(true)
    expect(wrapper.text()).toContain('白名单设备重新插入后经过扫描审计方可使用')
    expect(wrapper.text()).toContain('未授权设备显示黄色指示灯常亮')
    expect(wrapper.text()).toContain('添加完成后需重新拔插U盘生效')
  })

  it('管理端枚举原始执行错误映射为稳定中文且不泄露路径命令', async () => {
    vi.mocked(listManagementUsbStorageDevices).mockRejectedValue(
      new Error('spawn powershell.exe C:\\private\\script.ps1 stderr=Access denied'),
    )
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click')
    await flushPromises()

    expect(wrapper.getComponent(AddDialogStub).props('errorMessage')).toBe('管理端 USB 设备读取失败，请重试')
    expect(wrapper.text()).not.toContain('powershell')
  })

  it('白名单加载错误仅向表格展示固定可信文案', () => {
    useWhitelistStore().errorMessage = 'powershell.exe C:\\private\\load.ps1 stderr=denied'
    const wrapper = mountPage()

    expect(wrapper.getComponent({ name: 'DataTable' }).props('error'))
      .toBe('U盘白名单加载失败，请重试')
    expect(wrapper.get('[data-testid="table-error"]').text()).toBe('U盘白名单加载失败，请重试')
    expect(wrapper.text()).not.toContain('powershell.exe')
    expect(wrapper.text()).not.toContain('load.ps1')
  })

  it('严格在打开对应来源时调用各自枚举服务并映射装置候选', async () => {
    vi.mocked(getConnectedDevices).mockResolvedValue(usb_control.RspConnectedDevices.fromObject({ devices: [{
      serialNumber: 'DEVICE-SN', deviceName: '装置U盘', vid: '1', pid: '2', capacityBytes: 3,
      deviceType: 'storage', interfaceType: 'mass_storage', admissionStatus: 'addable', failReason: '',
    }] }))
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-device-trigger"]').trigger('click'); await flushPromises()
    expect(getConnectedDevices).toHaveBeenCalledWith('token')
    expect(listManagementUsbStorageDevices).not.toHaveBeenCalled()
    expect(wrapper.getComponent(AddDialogStub).props('candidates')[0]).toMatchObject({
      serialNumber: 'DEVICE-SN', deviceType: 'storage', addable: true,
    })
    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click'); await flushPromises()
    expect(listManagementUsbStorageDevices).toHaveBeenCalledTimes(1)
  })

  it('装置候选缺少可选字符串字段时按不可添加设备处理', async () => {
    vi.mocked(getConnectedDevices).mockResolvedValue({
      devices: [{ deviceType: 'storage', admissionStatus: 'addable' }],
    } as usb_control.RspConnectedDevices)
    const wrapper = mountPage()

    await wrapper.get('[data-testid="add-device-trigger"]').trigger('click')
    await flushPromises()

    expect(wrapper.getComponent(AddDialogStub).props('candidates')).toEqual([{
      serialNumber: '',
      vid: '',
      pid: '',
      deviceName: '',
      capacityBytes: 0,
      deviceType: 'storage',
      addable: false,
      unavailableReason: '设备标识异常',
    }])
  })

  it('关闭后切换来源创建新Dialog实例且management提交方法正确', async () => {
    vi.mocked(getConnectedDevices).mockResolvedValue(usb_control.RspConnectedDevices.fromObject({ devices: [{
      serialNumber: 'DEVICE-OLD', deviceName: '旧装置盘', deviceType: 'storage',
      admissionStatus: 'addable',
    }] }))
    vi.mocked(listManagementUsbStorageDevices).mockResolvedValue([managementCandidate])
    const add = vi.spyOn(useWhitelistStore(), 'addWhitelist').mockResolvedValue()
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-device-trigger"]').trigger('click')
    await flushPromises()
    const oldDialog = wrapper.getComponent(AddDialogStub)
    const oldDialogVm = oldDialog.vm
    oldDialog.vm.$emit('update:visible', false)
    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click')
    await flushPromises()
    const newDialog = wrapper.getComponent(AddDialogStub)

    expect(newDialog.vm).not.toBe(oldDialogVm)
    expect(newDialog.props('source')).toBe('management')
    newDialog.vm.$emit('submit', {
      candidate: managementCandidate, description: '', permission: 'readonly',
    })
    await flushPromises()
    expect(add).toHaveBeenCalledWith('token', expect.objectContaining({ addMethod: 'management' }))
  })

  it('装置端确认不重复GET并以device和受限权限调用store.add', async () => {
    const store = useWhitelistStore(); const add = vi.spyOn(store, 'addWhitelist').mockResolvedValue()
    vi.mocked(getConnectedDevices).mockResolvedValue(usb_control.RspConnectedDevices.fromObject({ devices: [{
      serialNumber: 'DEVICE-SN', deviceName: '装置U盘', vid: '1', pid: '2', capacityBytes: 3,
      deviceType: 'storage', interfaceType: 'mass_storage', admissionStatus: 'addable', failReason: '',
    }] }))
    const wrapper = mountPage(); await wrapper.get('[data-testid="add-device-trigger"]').trigger('click'); await flushPromises()
    wrapper.getComponent(AddDialogStub).vm.$emit('submit', {
      candidate: { ...managementCandidate, serialNumber: 'DEVICE-SN' }, description: '描述', permission: 'readonly',
    }); await flushPromises()
    expect(getConnectedDevices).toHaveBeenCalledTimes(1)
    expect(add).toHaveBeenCalledWith('token', expect.objectContaining({
      serialNumber: 'DEVICE-SN', addMethod: 'device', permission: 'readonly', deviceType: 'storage',
    }))
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(showSuccessToast).toHaveBeenCalledWith('添加成功，重新拔插后生效')
  })

  it('管理端提交前二次枚举，trim精确匹配；拔出和异常候选均不写入', async () => {
    const store = useWhitelistStore(); const add = vi.spyOn(store, 'addWhitelist').mockResolvedValue()
    vi.mocked(listManagementUsbStorageDevices)
      .mockResolvedValueOnce([managementCandidate]).mockResolvedValueOnce([])
      .mockResolvedValueOnce([managementCandidate]).mockResolvedValueOnce([{ ...managementCandidate, serialNumber: '' }])
    const wrapper = mountPage(); await wrapper.get('[data-testid="add-management-trigger"]').trigger('click'); await flushPromises()
    const submit = { candidate: managementCandidate, description: '', permission: 'readwrite' as const }
    wrapper.getComponent(AddDialogStub).vm.$emit('submit', submit); await flushPromises()
    expect(add).not.toHaveBeenCalled()
    expect(wrapper.getComponent(AddDialogStub).props('errorMessage')).toBe('设备已移除，请重新插入后再添加')
    wrapper.getComponent(AddDialogStub).vm.$emit('update:visible', false)
    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click'); await flushPromises()
    wrapper.getComponent(AddDialogStub).vm.$emit('submit', submit); await flushPromises()
    expect(add).not.toHaveBeenCalled()
  })

  it('管理端二次枚举同一trim序列号出现多条时按设备移除拒绝', async () => {
    const normalizedDuplicate = { ...managementCandidate, serialNumber: 'LOCAL-SN' }
    vi.mocked(listManagementUsbStorageDevices)
      .mockResolvedValueOnce([managementCandidate])
      .mockResolvedValueOnce([managementCandidate, normalizedDuplicate])
    const add = vi.spyOn(useWhitelistStore(), 'addWhitelist').mockResolvedValue()
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click')
    await flushPromises()

    wrapper.getComponent(AddDialogStub).vm.$emit('submit', {
      candidate: managementCandidate, description: '', permission: 'readonly',
    })
    await flushPromises()

    expect(add).not.toHaveBeenCalled()
    expect(wrapper.getComponent(AddDialogStub).props('errorMessage')).toBe('设备已移除，请重新插入后再添加')
  })

  it('管理端二次枚举错误同样映射稳定中文', async () => {
    vi.mocked(listManagementUsbStorageDevices)
      .mockResolvedValueOnce([managementCandidate])
      .mockRejectedValueOnce(new Error('execFile C:\\secret stderr'))
    const add = vi.spyOn(useWhitelistStore(), 'addWhitelist').mockResolvedValue()
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click')
    await flushPromises()
    wrapper.getComponent(AddDialogStub).vm.$emit('submit', {
      candidate: managementCandidate, description: '', permission: 'readonly',
    })
    await flushPromises()

    expect(add).not.toHaveBeenCalled()
    expect(wrapper.getComponent(AddDialogStub).props('errorMessage')).toBe('管理端 USB 设备读取失败，请重试')
    expect(wrapper.text()).not.toContain('execFile')
  })

  it('关闭管理端弹窗后迟到的二次枚举不添加也不提示成功', async () => {
    const secondEnumeration = createDeferred<Array<typeof managementCandidate>>()
    vi.mocked(listManagementUsbStorageDevices)
      .mockResolvedValueOnce([managementCandidate])
      .mockReturnValueOnce(secondEnumeration.promise)
    const add = vi.spyOn(useWhitelistStore(), 'addWhitelist').mockResolvedValue()
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click')
    await flushPromises()
    const dialog = wrapper.getComponent(AddDialogStub)
    dialog.vm.$emit('submit', {
      candidate: managementCandidate, description: '', permission: 'readonly',
    })
    dialog.vm.$emit('update:visible', false)
    secondEnumeration.resolve([managementCandidate])
    await flushPromises()

    expect(add).not.toHaveBeenCalled()
    expect(showSuccessToast).not.toHaveBeenCalled()
  })

  it('管理端真实成功使用二次枚举结果、management方法且重复提交只成功一次', async () => {
    vi.mocked(listManagementUsbStorageDevices).mockResolvedValue([managementCandidate])
    const store = useWhitelistStore(); const add = vi.spyOn(store, 'addWhitelist').mockResolvedValue()
    const wrapper = mountPage(); await wrapper.get('[data-testid="add-management-trigger"]').trigger('click'); await flushPromises()
    const dialog = wrapper.getComponent(AddDialogStub)
    const value = { candidate: managementCandidate, description: '本地', permission: 'readwrite' }
    dialog.vm.$emit('submit', value); dialog.vm.$emit('submit', value); await flushPromises()
    expect(add).toHaveBeenCalledTimes(1)
    expect(add).toHaveBeenCalledWith('token', expect.objectContaining({
      serialNumber: 'LOCAL-SN', addMethod: 'management', permission: 'readwrite',
    }))
    expect(showSuccessToast).toHaveBeenCalledWith('添加成功，重新拔插后生效')
  })

  it('添加失败展示统一错误、不假成功并释放ownership允许重试', async () => {
    vi.mocked(getConnectedDevices).mockResolvedValue(usb_control.RspConnectedDevices.fromObject({ devices: [{
      serialNumber: 'DUPLICATE-SN', deviceName: '装置U盘', vid: '1', pid: '2', capacityBytes: 3,
      deviceType: 'storage', interfaceType: 'mass_storage', admissionStatus: 'addable', failReason: '',
    }] }))
    const store = useWhitelistStore()
    const add = vi.spyOn(store, 'addWhitelist')
      .mockRejectedValueOnce(new ServiceError('该设备已在白名单中', 0x1002, 'business'))
      .mockResolvedValueOnce()
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-device-trigger"]').trigger('click')
    await flushPromises()
    const dialog = wrapper.getComponent(AddDialogStub)
    const value = {
      candidate: { ...managementCandidate, serialNumber: 'DUPLICATE-SN' },
      description: '', permission: 'readonly',
    }

    dialog.vm.$emit('submit', value)
    await flushPromises()

    expect(add).toHaveBeenCalledTimes(1)
    expect(dialog.props('errorMessage')).toBe('该设备已在白名单中')
    expect(showSuccessToast).not.toHaveBeenCalled()
    expect(dialog.props('visible')).toBe(true)
    expect(useWhitelistStore().devices).toHaveLength(21)

    dialog.vm.$emit('submit', value)
    await flushPromises()
    expect(add).toHaveBeenCalledTimes(2)
    expect(showSuccessToast).toHaveBeenCalledWith('添加成功，重新拔插后生效')
  })

  it('添加请求期间拒绝打开另一来源，断线后迟到成功不关闭或提示成功', async () => {
    const addRequest = createDeferred<void>()
    vi.mocked(getConnectedDevices).mockResolvedValue(usb_control.RspConnectedDevices.fromObject({ devices: [{
      serialNumber: 'DEVICE-PENDING', deviceName: '装置U盘', vid: '1', pid: '2', capacityBytes: 3,
      deviceType: 'storage', interfaceType: 'mass_storage', admissionStatus: 'addable', failReason: '',
    }] }))
    vi.spyOn(useWhitelistStore(), 'addWhitelist').mockReturnValue(addRequest.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="add-device-trigger"]').trigger('click')
    await flushPromises()
    const dialog = wrapper.getComponent(AddDialogStub)
    dialog.vm.$emit('submit', {
      candidate: { ...managementCandidate, serialNumber: 'DEVICE-PENDING' },
      description: '', permission: 'readonly',
    })
    await nextTick()

    await wrapper.get('[data-testid="add-management-trigger"]').trigger('click')
    expect(listManagementUsbStorageDevices).not.toHaveBeenCalled()
    expect(dialog.props('source')).toBe('device')

    useConnectionStore().updateStatus('DISCONNECTED')
    addRequest.resolve()
    await flushPromises()
    expect(dialog.props('visible')).toBe(true)
    expect(showSuccessToast).not.toHaveBeenCalled()
  })

  it('删除成功确认明确serial并显示成功文案', async () => {
    const store = useWhitelistStore()
    const remove = vi.spyOn(store, 'removeWhitelist').mockResolvedValue()
    const wrapper = mountPage()

    await wrapper.find('[data-testid="remove-SN-1"]').trigger('click')
    await flushPromises()

    expect(ElMessageBox.confirm).toHaveBeenCalledWith(
      '确定删除白名单设备 SN-1 吗？', '删除确认', expect.any(Object),
    )
    expect(remove).toHaveBeenCalledWith('token', 'SN-1')
    expect(showSuccessToast).toHaveBeenCalledWith('删除成功')
  })

  it('删除失败不假成功且释放ownership允许相同serial重试', async () => {
    const store = useWhitelistStore()
    const remove = vi.spyOn(store, 'removeWhitelist')
      .mockRejectedValueOnce(new Error('C:\\secret\\powershell.exe stderr'))
      .mockResolvedValueOnce()
    const wrapper = mountPage()
    const deleteButton = wrapper.find('[data-testid="remove-SN-1"]')

    await deleteButton.trigger('click')
    await flushPromises()
    expect(remove).toHaveBeenCalledTimes(1)
    expect(remove).toHaveBeenLastCalledWith('token', 'SN-1')
    expect(showErrorDialog).toHaveBeenCalledWith('白名单删除失败', 'C:\\secret\\powershell.exe stderr')
    expect(wrapper.text()).not.toContain('powershell.exe')
    expect(showSuccessToast).not.toHaveBeenCalled()
    expect(useWhitelistStore().devices).toHaveLength(21)

    await deleteButton.trigger('click')
    await flushPromises()
    expect(remove).toHaveBeenCalledTimes(2)
    expect(ElMessageBox.confirm).toHaveBeenCalledTimes(2)
    expect(showSuccessToast).toHaveBeenCalledWith('删除成功')
  })

  it('删除确认等待期间断线二次复检', async () => {
    const store = useWhitelistStore()
    const remove = vi.spyOn(store, 'removeWhitelist').mockResolvedValue()
    const wrapper = mountPage()
    let confirmResolve!: (value: never) => void
    vi.mocked(ElMessageBox.confirm).mockReturnValue(new Promise((resolve) => { confirmResolve = resolve }))
    await wrapper.find('[data-testid="remove-SN-1"]').trigger('click')
    useConnectionStore().updateStatus('DISCONNECTED'); confirmResolve('confirm' as never); await flushPromises()
    expect(ElMessageBox.confirm).toHaveBeenCalled()
    expect(remove).not.toHaveBeenCalled()
    expect(ElMessage.warning).toHaveBeenCalledWith('装置已断开连接，无法修改白名单')
  })

})
