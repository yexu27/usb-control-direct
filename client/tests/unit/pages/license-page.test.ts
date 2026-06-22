import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { ElMessage } from 'element-plus'
import { usb_control } from '../../../src/shared/proto/usb_control'
import LicensePage from '../../../src/renderer/pages/LicensePage.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { getMachineCode, uploadLicense } from '../../../src/renderer/services/auth-service'

vi.mock('../../../src/renderer/services/auth-service', () => ({
  getMachineCode: vi.fn(),
  uploadLicense: vi.fn(),
}))

vi.mock('element-plus', () => ({
  ElMessage: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
  },
}))

const push = vi.fn()
vi.mock('vue-router', () => ({
  useRouter: () => ({ push }),
}))

const openFile = vi.fn()
const readFile = vi.fn()
const saveFile = vi.fn()
const writeFile = vi.fn()
const applyStateEvent = vi.fn().mockResolvedValue(undefined)
const disconnect = vi.fn().mockResolvedValue(undefined)
const getMachineCodeMock = vi.mocked(getMachineCode)
const uploadLicenseMock = vi.mocked(uploadLicense)

interface LicensePageVm {
  machineCode: string
  qrcodePng: Uint8Array
  selectedLicensePath: string
  uploadError: string
  handleOpenMachineCode: () => Promise<void>
  handleDownloadQrcode: () => Promise<void>
  handleOpenUploadDialog: () => void
  handleSelectLicenseFile: () => Promise<void>
  handleConfirmUpload: () => Promise<void>
}

function setTemporarySession(authStatus: 'unauthorized' | 'expired' = 'unauthorized'): void {
  useSessionStore().setSession({
    token: 'temporary-token',
    username: 'operator',
    role: 'operator',
    authStatus,
    authExpireTime: 0,
    deviceDescription: '',
  })
  const connection = useConnectionStore()
  connection.deviceIp = '19.19.19.16'
  connection.updateStatus(authStatus === 'expired' ? 'LICENSE_EXPIRED' : 'AUTH_REQUIRED')
}

function mountPage() {
  return mount(LicensePage, {
    global: {
      stubs: {
        ElDialog: { template: '<section><slot /><slot name="footer" /></section>' },
        ElButton: true,
        ElInput: true,
        ElAlert: true,
        ElIcon: true,
      },
      directives: {
        loading: {},
      },
    },
  })
}

describe('LicensePage', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    applyStateEvent.mockResolvedValue(undefined)
    disconnect.mockResolvedValue(undefined)
    window.desktopApi = {
      tls: { applyStateEvent, disconnect },
      dialog: { openFile, readFile, saveFile, writeFile },
      window: {
        minimize: vi.fn(),
        maximize: vi.fn(),
        close: vi.fn(),
      },
    } as unknown as Window['desktopApi']
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: vi.fn().mockResolvedValue(undefined) },
    })
    vi.stubGlobal('URL', {
      createObjectURL: vi.fn(() => 'blob:qrcode'),
      revokeObjectURL: vi.fn(),
    })
  })

  it('授权到期时展示续期提示且不展示页面角色归属标识', () => {
    setTemporarySession('expired')
    const wrapper = mountPage()

    expect(wrapper.get('.license-page').classes()).toContain('auth-page')
    expect(wrapper.get('.license-card').classes()).toContain('auth-card')
    expect(wrapper.get('#license-title').classes()).toContain('auth-title')
    expect(wrapper.text()).toContain('授权已到期')
    expect(wrapper.text()).not.toContain('操作员')
    expect(wrapper.text()).not.toContain('审计员')
    expect(wrapper.text()).not.toContain('系统管理员')
  })

  it('获取机器码后保留文本和二维码原始字节', async () => {
    setTemporarySession()
    getMachineCodeMock.mockResolvedValue(usb_control.RspMachineCode.fromObject({
      machineCode: 'USB-DEVICE-RK3568-001',
      qrcodePng: new Uint8Array([1, 2, 3]),
    }))
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LicensePageVm

    await component.handleOpenMachineCode()

    expect(component.machineCode).toBe('USB-DEVICE-RK3568-001')
    expect(component.qrcodePng).toEqual(new Uint8Array([1, 2, 3]))
  })

  it('二维码通过安全保存 IPC 写入用户选择的位置', async () => {
    setTemporarySession()
    getMachineCodeMock.mockResolvedValue(usb_control.RspMachineCode.fromObject({
      machineCode: 'USB-DEVICE-RK3568-001',
      qrcodePng: new Uint8Array([1, 2, 3]),
    }))
    saveFile.mockResolvedValue({ canceled: false, filePath: 'C:\\Users\\Admin\\qrcode.png' })
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LicensePageVm
    await component.handleOpenMachineCode()

    await component.handleDownloadQrcode()

    expect(writeFile).toHaveBeenCalledWith(
      'C:\\Users\\Admin\\qrcode.png',
      new Uint8Array([1, 2, 3]),
    )
    expect(ElMessage.success).toHaveBeenCalledWith('二维码图片已下载')
  })

  it('未选择授权文件时拒绝上传', async () => {
    setTemporarySession()
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LicensePageVm
    component.handleOpenUploadDialog()

    await component.handleConfirmUpload()

    expect(component.uploadError).toBe('请先选择授权文件')
    expect(readFile).not.toHaveBeenCalled()
    expect(uploadLicenseMock).not.toHaveBeenCalled()
  })

  it('选择一次授权文件并在确认时读取一次', async () => {
    setTemporarySession()
    openFile.mockResolvedValue({
      canceled: false,
      filePaths: ['C:\\Users\\Admin\\license.txt'],
    })
    readFile.mockResolvedValue(new Uint8Array([1, 2, 3]))
    uploadLicenseMock.mockResolvedValue(usb_control.RspUploadLicense.fromObject({
      success: true,
      resultCode: 0,
      expireTime: 1_893_455_999,
    }))
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LicensePageVm
    component.handleOpenUploadDialog()

    await component.handleSelectLicenseFile()
    await component.handleConfirmUpload()

    expect(openFile).toHaveBeenCalledTimes(1)
    expect(readFile).toHaveBeenCalledTimes(1)
    expect(uploadLicenseMock).toHaveBeenCalledWith(
      'temporary-token',
      new Uint8Array([1, 2, 3]),
    )
    expect(applyStateEvent).toHaveBeenCalledWith('LICENSE_UPLOAD_SUCCESS')
    expect(disconnect).toHaveBeenCalledTimes(1)
    expect(useSessionStore().token).toBe('')
    expect(useConnectionStore().deviceIp).toBe('')
    expect(push).toHaveBeenCalledWith('/login')
  })

  it('拒绝扩展名不是 txt 的授权文件', async () => {
    setTemporarySession()
    openFile.mockResolvedValue({
      canceled: false,
      filePaths: ['C:\\Users\\Admin\\license.bin'],
    })
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LicensePageVm
    component.handleOpenUploadDialog()

    await component.handleSelectLicenseFile()
    await component.handleConfirmUpload()

    expect(component.uploadError).toBe('仅支持 .txt 格式授权文件')
    expect(readFile).not.toHaveBeenCalled()
  })
})
