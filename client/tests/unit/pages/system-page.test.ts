// @vitest-environment happy-dom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import { ElMessageBox } from 'element-plus'
import SystemPage from '../../../src/renderer/pages/SystemPage.vue'
import { getMachineCode, uploadLicense } from '../../../src/renderer/services/auth-service'
import {
  getSystemInfo,
  updateDeviceDescription,
  uploadSystemUpgrade,
  uploadVirusdbUpgrade,
} from '../../../src/renderer/services/system-service'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { parseSystemUpgradeVersion, parseVirusdbUpgradeVersion } from '../../../src/renderer/utils/upgrade-package'
import { showSuccessToast } from '../../../src/renderer/utils/operation-feedback'

vi.mock('../../../src/renderer/services/system-service', () => ({
  getSystemInfo: vi.fn(),
  uploadSystemUpgrade: vi.fn(),
  uploadVirusdbUpgrade: vi.fn(),
  updateDeviceDescription: vi.fn(),
}))

vi.mock('../../../src/renderer/services/auth-service', () => ({
  getMachineCode: vi.fn(),
  uploadLicense: vi.fn(),
}))

vi.mock('../../../src/renderer/utils/upgrade-package', () => ({
  calculateSha256Hex: vi.fn(() => Promise.resolve('checksum')),
  parseSystemUpgradeVersion: vi.fn(() => 'v1.2.3'),
  parseVirusdbUpgradeVersion: vi.fn(() => 'v3.0.5'),
}))

vi.mock('element-plus', () => ({
  ElMessage: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
  ElMessageBox: {
    alert: vi.fn(() => Promise.resolve()),
    confirm: vi.fn(() => Promise.resolve()),
  },
}))

vi.mock('../../../src/renderer/utils/operation-feedback', async () => {
  const actual = await vi.importActual<typeof import('../../../src/renderer/utils/operation-feedback')>(
    '../../../src/renderer/utils/operation-feedback',
  )
  return {
    ...actual,
    showSuccessToast: vi.fn(),
  }
})

const openFile = vi.fn()
const readFile = vi.fn()
const saveFile = vi.fn()
const writeFile = vi.fn()
let pinia: Pinia

const ElInputStub = defineComponent({
  props: ['modelValue'],
  emits: ['update:modelValue'],
  template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)">',
})

function mountPage() {
  return mount(SystemPage, {
    global: {
      plugins: [pinia],
      stubs: {
        ConnectionAlert: { template: '<aside />' },
        ProgressDialog: { props: ['visible'], template: '<div data-testid="progress" :data-visible="visible" />' },
        ElCard: { template: '<section><header><slot name="header" /></header><slot /></section>' },
        ElButton: { template: '<button type="button" @click="$emit(\'click\')"><slot /></button>' },
        ElInput: ElInputStub,
        ElDialog: { template: '<section><slot /><slot name="footer" /></section>' },
      },
    },
  })
}

function seedStores(): void {
  useSessionStore().setSession({
    token: 'token',
    username: 'admin',
    role: 'admin',
    authStatus: 'authorized',
    authExpireTime: 0,
    deviceDescription: '',
  })
  useConnectionStore().updateStatus('CONNECTED')
}

describe('SystemPage', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    seedStores()
    vi.clearAllMocks()
    vi.stubGlobal('URL', {
      createObjectURL: vi.fn(() => 'blob:qrcode'),
      revokeObjectURL: vi.fn(),
    })
    window.desktopApi = {
      dialog: { openFile, readFile, saveFile, writeFile },
    } as unknown as Window['desktopApi']
    openFile.mockResolvedValue({ canceled: true, filePaths: [] })
    saveFile.mockResolvedValue({ canceled: false, filePath: 'C:\\tmp\\qrcode.png' })
    readFile.mockResolvedValue(new Uint8Array([97, 98, 99]))
    writeFile.mockResolvedValue(undefined)
    vi.mocked(getSystemInfo).mockResolvedValue({
      systemVersion: 'v1.0.0',
      virusDbVersion: 'v3.0.3',
      virusDbUpdatedAt: 1_767_225_610,
      authorized: true,
      authStatus: 'authorized',
      authExpireTime: 1_893_455_999,
      deviceDescription: 'USB_DEVICE_OLD',
    } as never)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('loads system data into four prototype-style management cards', async () => {
    const wrapper = mountPage()

    await flushPromises()

    expect(getSystemInfo).toHaveBeenCalledWith('token')
    expect(wrapper.get('[data-testid="system-card-grid"]').classes()).toContain('system-grid')
    const cards = wrapper.findAll('[data-testid="system-management-card"]')
    expect(cards).toHaveLength(4)
    expect(cards.map((card) => card.classes())).toEqual([
      expect.arrayContaining(['system-card', 'system-card-upgrade']),
      expect.arrayContaining(['system-card', 'system-card-virusdb']),
      expect.arrayContaining(['system-card', 'system-card-license']),
      expect.arrayContaining(['system-card', 'system-card-device-desc']),
    ])
    expect(cards.map((card) => card.text())).toEqual([
      expect.stringContaining('系统升级'),
      expect.stringContaining('病毒库升级'),
      expect.stringContaining('授权信息管理'),
      expect.stringContaining('自定义设备描述'),
    ])
    expect(wrapper.text()).not.toContain('系统信息')
    expect(wrapper.text()).toContain('系统升级')
    expect(wrapper.text()).toContain('当前版本: v1.0.0')
    expect(wrapper.text()).toContain('上传 .bin 升级包')
    expect(wrapper.text()).toContain('病毒库升级')
    expect(wrapper.text()).toContain('当前: v3.0.3')
    expect(wrapper.text()).toContain('上传 .zip 升级包')
    expect(wrapper.text()).toContain('授权信息管理')
    expect(wrapper.text()).toContain('状态:')
    expect(wrapper.text()).toContain('已授权')
    expect(wrapper.text()).toContain('自定义设备描述')
    expect(wrapper.text()).toContain('当前:')
    expect(wrapper.text()).toContain('USB_DEVICE_OLD')
    expect(wrapper.text()).toContain('修改完成后需重启设备')
    expect(wrapper.text()).toContain('修改')
    expect(wrapper.text()).not.toContain('系统管理员')
    expect(wrapper.text()).not.toContain('安全U盘自动升级')
  })

  it('uploads system upgrade package immediately after selecting a valid file', async () => {
    vi.useFakeTimers()
    openFile.mockResolvedValue({ canceled: false, filePaths: ['C:\\pkg\\usb-control-system-v1.2.3.bin'] })
    vi.mocked(uploadSystemUpgrade).mockResolvedValue(undefined)
    const connection = useConnectionStore()
    const disconnect = vi.spyOn(connection, 'disconnect').mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="system-upgrade-upload"]').trigger('click')
    await flushPromises()

    expect(uploadSystemUpgrade).toHaveBeenCalledWith(
      'token',
      new Uint8Array([97, 98, 99]),
      'v1.2.3',
      expect.any(String),
    )
    expect(disconnect).not.toHaveBeenCalled()

    vi.advanceTimersByTime(3_000)
    await flushPromises()

    expect(disconnect).toHaveBeenCalledWith(true)
  })

  it('keeps system upgrade progress dialog visible for at least 3 seconds', async () => {
    vi.useFakeTimers()
    openFile.mockResolvedValue({ canceled: false, filePaths: ['C:\\pkg\\usb-control-system-v1.2.3.bin'] })
    vi.mocked(uploadSystemUpgrade).mockResolvedValue(undefined)
    const connection = useConnectionStore()
    vi.spyOn(connection, 'disconnect').mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="system-upgrade-upload"]').trigger('click')
    await flushPromises()
    expect(uploadSystemUpgrade).toHaveBeenCalled()

    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('true')
    vi.advanceTimersByTime(2_999)
    await flushPromises()
    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('true')

    vi.advanceTimersByTime(1)
    await flushPromises()

    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('false')
    expect(showSuccessToast).toHaveBeenCalledWith('升级完成，请重新连接')
  })

  it('uploads virusdb package immediately after selecting a valid file and refreshes system info', async () => {
    vi.useFakeTimers()
    openFile.mockResolvedValue({ canceled: false, filePaths: ['D:\\pkg\\usb-control-virusdb-v3.0.5.zip'] })
    vi.mocked(uploadVirusdbUpgrade).mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()
    const initialInfoCallCount = vi.mocked(getSystemInfo).mock.calls.length

    await wrapper.get('[data-testid="virusdb-upgrade-upload"]').trigger('click')
    await flushPromises()

    expect(uploadVirusdbUpgrade).toHaveBeenCalledWith(
      'token',
      new Uint8Array([97, 98, 99]),
      'v3.0.5',
      expect.any(String),
    )
    expect(vi.mocked(getSystemInfo).mock.calls.length).toBe(initialInfoCallCount)

    vi.advanceTimersByTime(3_000)
    await flushPromises()

    expect(vi.mocked(getSystemInfo).mock.calls.length).toBeGreaterThan(initialInfoCallCount)
  })

  it('keeps virusdb upgrade progress dialog visible for at least 3 seconds', async () => {
    vi.useFakeTimers()
    openFile.mockResolvedValue({ canceled: false, filePaths: ['D:\\pkg\\usb-control-virusdb-v3.0.5.zip'] })
    vi.mocked(uploadVirusdbUpgrade).mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="virusdb-upgrade-upload"]').trigger('click')
    await flushPromises()
    expect(uploadVirusdbUpgrade).toHaveBeenCalled()

    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('true')
    vi.advanceTimersByTime(2_999)
    await flushPromises()
    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('true')

    vi.advanceTimersByTime(1)
    await flushPromises()

    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('false')
    expect(showSuccessToast).toHaveBeenCalledWith('病毒库升级成功')
  })

  it('does not upload when upgrade file selection is canceled', async () => {
    openFile.mockResolvedValue({ canceled: true, filePaths: [] })
    const wrapper = mountPage()
    await flushPromises()
    vi.mocked(uploadSystemUpgrade).mockClear()
    vi.mocked(uploadVirusdbUpgrade).mockClear()

    await wrapper.get('[data-testid="system-upgrade-upload"]').trigger('click')
    await wrapper.get('[data-testid="virusdb-upgrade-upload"]').trigger('click')
    await flushPromises()

    expect(uploadSystemUpgrade).not.toHaveBeenCalled()
    expect(uploadVirusdbUpgrade).not.toHaveBeenCalled()
  })

  it('shows filename format errors without reading files or uploading', async () => {
    const wrapper = mountPage()
    await flushPromises()
    vi.mocked(uploadSystemUpgrade).mockClear()
    vi.mocked(uploadVirusdbUpgrade).mockClear()
    readFile.mockClear()

    openFile.mockResolvedValueOnce({ canceled: false, filePaths: ['C:\\pkg\\bad-system.bin'] })
    vi.mocked(parseSystemUpgradeVersion).mockImplementationOnce(() => {
      throw new Error('系统升级包文件名格式错误，请使用 usb-control-system-vX.Y.Z.bin')
    })
    await wrapper.get('[data-testid="system-upgrade-upload"]').trigger('click')
    await flushPromises()

    expect(ElMessageBox.alert).toHaveBeenCalledWith(
      '系统升级包文件名格式错误，请使用 usb-control-system-vX.Y.Z.bin',
      '系统升级包文件名错误',
      expect.objectContaining({
        customClass: 'app-confirm-message-box',
        modalClass: 'app-confirm-message-box-overlay',
        type: 'error',
      }),
    )
    expect(readFile).not.toHaveBeenCalled()
    expect(uploadSystemUpgrade).not.toHaveBeenCalled()
    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('false')

    openFile.mockResolvedValueOnce({ canceled: false, filePaths: ['D:\\pkg\\bad-virusdb.zip'] })
    vi.mocked(parseVirusdbUpgradeVersion).mockImplementationOnce(() => {
      throw new Error('病毒库升级包文件名格式错误，请使用 usb-control-virusdb-vX.Y.Z.zip')
    })
    await wrapper.get('[data-testid="virusdb-upgrade-upload"]').trigger('click')
    await flushPromises()

    expect(ElMessageBox.alert).toHaveBeenCalledWith(
      '病毒库升级包文件名格式错误，请使用 usb-control-virusdb-vX.Y.Z.zip',
      '病毒库升级包文件名错误',
      expect.objectContaining({
        customClass: 'app-confirm-message-box',
        modalClass: 'app-confirm-message-box-overlay',
        type: 'error',
      }),
    )
    expect(readFile).not.toHaveBeenCalled()
    expect(uploadVirusdbUpgrade).not.toHaveBeenCalled()
    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('false')
  })

  it('shows machine code, saves qrcode, and uploads license file', async () => {
    vi.mocked(getMachineCode).mockResolvedValue({
      machineCode: 'USB-DEVICE-RK3568-001',
      qrcodePng: new Uint8Array([1, 2]),
    } as never)
    vi.mocked(uploadLicense).mockResolvedValue({
      success: true,
      expireTime: 1_893_455_999,
      resultCode: 0,
      errorMessage: '',
    } as never)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="machine-code-open"]').trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('USB-DEVICE-RK3568-001')
    await wrapper.get('[data-testid="machine-code-save"]').trigger('click')
    await flushPromises()
    expect(writeFile).toHaveBeenCalledWith('C:\\tmp\\qrcode.png', new Uint8Array([1, 2]))

    openFile.mockResolvedValue({ canceled: false, filePaths: ['C:\\auth\\license.txt'] })
    await wrapper.get('[data-testid="license-upload"]').trigger('click')
    await flushPromises()
    expect(uploadLicense).toHaveBeenCalledWith('token', new Uint8Array([97, 98, 99]))
  })

  it('keeps license upload progress visible for at least 3 seconds and shows success toast', async () => {
    vi.useFakeTimers()
    vi.mocked(uploadLicense).mockResolvedValue({
      success: true,
      expireTime: 1_893_455_999,
      resultCode: 0,
      errorMessage: '',
    } as never)
    openFile.mockResolvedValue({ canceled: false, filePaths: ['C:\\auth\\license.txt'] })
    const wrapper = mountPage()
    await flushPromises()
    const initialInfoCallCount = vi.mocked(getSystemInfo).mock.calls.length

    await wrapper.get('[data-testid="license-upload"]').trigger('click')
    await flushPromises()
    expect(uploadLicense).toHaveBeenCalledWith('token', new Uint8Array([97, 98, 99]))

    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('true')
    vi.advanceTimersByTime(2_999)
    await flushPromises()
    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('true')

    vi.advanceTimersByTime(1)
    await flushPromises()

    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('false')
    expect(showSuccessToast).toHaveBeenCalledWith('授权文件上传成功')
    expect(vi.mocked(getSystemInfo).mock.calls.length).toBeGreaterThan(initialInfoCallCount)
  })

  it('shows centered license file format error without reading files or uploading', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['C:\\auth\\license.bin'] })
    const wrapper = mountPage()
    await flushPromises()
    vi.mocked(uploadLicense).mockClear()
    readFile.mockClear()

    await wrapper.get('[data-testid="license-upload"]').trigger('click')
    await flushPromises()

    expect(ElMessageBox.alert).toHaveBeenCalledWith(
      '仅支持 .txt 格式授权文件',
      '授权文件格式错误',
      expect.objectContaining({
        customClass: 'app-confirm-message-box',
        modalClass: 'app-confirm-message-box-overlay',
        type: 'error',
      }),
    )
    expect(readFile).not.toHaveBeenCalled()
    expect(uploadLicense).not.toHaveBeenCalled()
    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('false')
  })

  it('opens device description dialog with the PRD default value and saves after confirmation', async () => {
    vi.mocked(updateDeviceDescription).mockResolvedValue(undefined)
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm' as never)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="device-desc-edit"]').trigger('click')
    await flushPromises()

    const input = wrapper.get('[data-testid="device-desc-input"]')
    expect((input.element as HTMLInputElement).value).toBe('(AD USB protection dev)USB Device')

    await input.setValue('USB_DEVICE_01')
    await wrapper.get('[data-testid="device-desc-save"]').trigger('click')
    await flushPromises()

    expect(ElMessageBox.confirm).toHaveBeenCalledWith(
      expect.stringContaining('请确认当前装置未连接移动存储设备、键盘、鼠标等 USB 设备'),
      expect.any(String),
      expect.any(Object),
    )
    expect(updateDeviceDescription).toHaveBeenCalledWith('token', 'USB_DEVICE_01')
  })
})
