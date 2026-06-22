// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vitest'
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

vi.mock('element-plus', () => ({
  ElMessage: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
  ElMessageBox: { confirm: vi.fn(() => Promise.resolve()) },
}))

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

  it('loads and displays system information', async () => {
    const wrapper = mountPage()

    await flushPromises()

    expect(getSystemInfo).toHaveBeenCalledWith('token')
    expect(wrapper.text()).toContain('v1.0.0')
    expect(wrapper.text()).toContain('v3.0.3')
    expect(wrapper.text()).toContain('已授权')
    expect(wrapper.text()).toContain('USB_DEVICE_OLD')
  })

  it('uploads system upgrade package and disconnects after success', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['C:\\pkg\\usb-control-system-v1.2.3.bin'] })
    vi.mocked(uploadSystemUpgrade).mockResolvedValue(undefined)
    const connection = useConnectionStore()
    const disconnect = vi.spyOn(connection, 'disconnect').mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="system-upgrade-select"]').trigger('click')
    await flushPromises()
    await wrapper.get('[data-testid="system-upgrade-submit"]').trigger('click')
    await flushPromises()

    expect(uploadSystemUpgrade).toHaveBeenCalledWith(
      'token',
      new Uint8Array([97, 98, 99]),
      'v1.2.3',
      expect.any(String),
    )
    expect(disconnect).toHaveBeenCalledWith(true)
  })

  it('uploads virusdb package and refreshes system info', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['D:\\pkg\\usb-control-virusdb-v3.0.5.zip'] })
    vi.mocked(uploadVirusdbUpgrade).mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="virusdb-upgrade-select"]').trigger('click')
    await flushPromises()
    await wrapper.get('[data-testid="virusdb-upgrade-submit"]').trigger('click')
    await flushPromises()

    expect(uploadVirusdbUpgrade).toHaveBeenCalledWith(
      'token',
      new Uint8Array([97, 98, 99]),
      'v3.0.5',
      expect.any(String),
    )
    expect(getSystemInfo).toHaveBeenCalledTimes(2)
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

  it('updates device description after validation and confirmation only', async () => {
    vi.mocked(updateDeviceDescription).mockResolvedValue(undefined)
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm' as never)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="device-desc-input"]').setValue('USB_DEVICE_01')
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
