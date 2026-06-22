// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import PoliciesPage from '../../../src/renderer/pages/PoliciesPage.vue'
import { exportPolicy, importPolicy } from '../../../src/renderer/services/policy-service'
import { ServiceError } from '../../../src/renderer/services/send-command'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useFilePolicyStore } from '../../../src/renderer/stores/file-policy'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { useWhitelistStore } from '../../../src/renderer/stores/whitelist'
import type { ConnectionStatus } from '../../../src/shared/connection-state'

vi.mock('element-plus', () => ({
  ElMessage: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
  ElMessageBox: { confirm: vi.fn() },
}))
vi.mock('../../../src/renderer/services/policy-service', () => ({
  exportPolicy: vi.fn(),
  importPolicy: vi.fn(),
}))

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

const ProgressDialogStub = defineComponent({
  name: 'ProgressDialog',
  props: ['visible', 'title', 'message'],
  template: `
    <div data-testid="progress" :data-visible="visible" :data-title="title">
      {{ message }}
    </div>
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
}

function mountPage() {
  return mount(PoliciesPage, {
    global: {
      stubs: {
        ConnectionAlert: { template: '<aside data-testid="connection-alert" />' },
        ProgressDialog: ProgressDialogStub,
        ElCard: { template: '<section><slot /></section>' },
        ElButton: {
          emits: ['click'],
          template: '<button v-bind="$attrs" @click="$emit(\'click\')"><slot /></button>',
        },
      },
    },
  })
}

const saveFile = vi.fn()
const openFile = vi.fn()
const readFile = vi.fn()
const writeFile = vi.fn()
const revokeFileAccess = vi.fn()

describe('PoliciesPage', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    seed()
    window.desktopApi = {
      dialog: { saveFile, openFile, readFile, writeFile, revokeFileAccess },
    } as unknown as Window['desktopApi']
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm' as never)
    vi.mocked(exportPolicy).mockResolvedValue({ policyData: new Uint8Array([1, 2]) } as never)
    vi.mocked(importPolicy).mockResolvedValue()
    writeFile.mockResolvedValue(undefined)
    revokeFileAccess.mockResolvedValue(undefined)
  })

  it('按严格时间格式和 bin 过滤条件打开导出对话框', async () => {
    saveFile.mockResolvedValue({ canceled: true })
    const wrapper = mountPage()

    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await flushPromises()

    expect(saveFile).toHaveBeenCalledOnce()
    expect(saveFile.mock.calls[0][0]).toEqual({
      title: '导出安全策略',
      defaultPath: expect.stringMatching(/^安全策略-\d{8}-\d{6}\.bin$/),
      filters: [{ name: '安全策略', extensions: ['bin'] }],
    })
  })

  it.each([
    'DISCONNECTED',
    'CONNECTING',
    'AUTHENTICATING',
    'CHECK_LICENSE',
    'AUTH_REQUIRED',
    'LICENSE_EXPIRED',
    'LOADING_CONFIG',
  ] satisfies ConnectionStatus[])('%s 状态下警告且不打开文件对话框', async (status) => {
    useConnectionStore().updateStatus(status)
    const wrapper = mountPage()

    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await wrapper.get('[data-testid="import-policy"]').trigger('click')

    expect(saveFile).not.toHaveBeenCalled()
    expect(openFile).not.toHaveBeenCalled()
    expect(ElMessage.warning).toHaveBeenCalledTimes(2)
    expect(ElMessage.warning).toHaveBeenNthCalledWith(1, '装置已断开连接，无法传输策略')
    expect(ElMessage.warning).toHaveBeenNthCalledWith(2, '装置已断开连接，无法传输策略')
  })

  it('导出取消不请求装置，并阻止重复操作', async () => {
    const pendingSave = deferred<{ canceled: boolean }>()
    saveFile.mockReturnValue(pendingSave.promise)
    const wrapper = mountPage()

    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    expect(saveFile).toHaveBeenCalledTimes(1)
    pendingSave.resolve({ canceled: true })
    await flushPromises()

    expect(exportPolicy).not.toHaveBeenCalled()
    expect(writeFile).not.toHaveBeenCalled()
  })

  it('导出只写一次且成功后不额外撤权', async () => {
    saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/policy.bin' })
    const wrapper = mountPage()

    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await flushPromises()

    expect(exportPolicy).toHaveBeenCalledWith('session-token')
    expect(writeFile).toHaveBeenCalledOnce()
    expect(writeFile).toHaveBeenCalledWith('/tmp/policy.bin', new Uint8Array([1, 2]))
    expect(revokeFileAccess).not.toHaveBeenCalled()
    expect(ElMessage.success).toHaveBeenCalledWith('策略导出成功')
  })

  it.each(['export', 'write'] as const)('%s 失败时 await 撤权并脱敏', async (stage) => {
    saveFile.mockResolvedValue({ canceled: false, filePath: '/secret/policy.bin' })
    if (stage === 'export') vi.mocked(exportPolicy).mockRejectedValue(new Error('/secret cmd'))
    else writeFile.mockRejectedValue(new Error('/secret bytes'))
    const revokeDone = deferred<void>()
    revokeFileAccess.mockReturnValue(revokeDone.promise)
    const wrapper = mountPage()

    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await nextTick()
    expect(ElMessage.error).not.toHaveBeenCalled()
    revokeDone.resolve()
    await flushPromises()

    expect(revokeFileAccess).toHaveBeenCalledWith('/secret/policy.bin')
    expect(ElMessage.error).toHaveBeenCalledWith('策略导出失败')
  })

  it('仅传递可信 ServiceError 文案，且撤权失败不替换主错误', async () => {
    saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/policy.bin' })
    vi.mocked(exportPolicy).mockRejectedValue(new ServiceError('策略版本不兼容', 1, 'business'))
    revokeFileAccess.mockRejectedValue(new Error('/tmp/revoke failed'))
    const wrapper = mountPage()
    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await flushPromises()
    expect(ElMessage.error).toHaveBeenCalledWith('策略版本不兼容')
  })

  it('导入仅选 bin，取消时无动作', async () => {
    openFile.mockResolvedValue({ canceled: true, filePaths: [] })
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    expect(openFile.mock.calls[0][0].filters).toEqual([
      { name: '加密策略文件', extensions: ['bin'] },
    ])
    expect(readFile).not.toHaveBeenCalled()
    expect(importPolicy).not.toHaveBeenCalled()
    expect(revokeFileAccess).not.toHaveBeenCalled()
  })

  it('本地非 bin 文件立即拒绝且不读取', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.BIN.txt'] })
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    expect(readFile).not.toHaveBeenCalled()
    expect(importPolicy).not.toHaveBeenCalled()
    expect(ElMessage.error).toHaveBeenCalledWith('仅支持 .bin 策略文件')
    expect(revokeFileAccess).toHaveBeenCalledOnce()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.BIN.txt')
  })

  it('取消导入二次确认时不读取', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.BIN'] })
    vi.mocked(ElMessageBox.confirm).mockRejectedValue(new Error('cancel'))
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    expect(ElMessageBox.confirm).toHaveBeenCalledOnce()
    expect(readFile).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.BIN')
  })

  it('确认后只读一次并导入，传输期显示 ProgressDialog', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([7, 8]))
    const pendingImport = deferred<void>()
    vi.mocked(importPolicy).mockReturnValue(pendingImport.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    expect(readFile).toHaveBeenCalledTimes(1)
    expect(importPolicy).toHaveBeenCalledWith('session-token', new Uint8Array([7, 8]))
    expect(wrapper.get('[data-testid="progress"]').attributes('data-visible')).toBe('true')
    pendingImport.resolve()
    await flushPromises()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('导入成功后并行刷新两个 store 并等待全部 settled', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([9]))
    const fileDone = deferred<void>()
    const whitelistDone = deferred<void>()
    const fileLoad = vi.spyOn(useFilePolicyStore(), 'load').mockReturnValue(fileDone.promise)
    const whitelistLoad = vi
      .spyOn(useWhitelistStore(), 'listWhitelist')
      .mockReturnValue(whitelistDone.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    expect(fileLoad).toHaveBeenCalledWith('session-token')
    expect(whitelistLoad).toHaveBeenCalledWith('session-token')
    fileDone.resolve()
    await flushPromises()
    expect(ElMessage.success).not.toHaveBeenCalled()
    whitelistDone.resolve()
    await flushPromises()
    expect(ElMessage.success).toHaveBeenCalledWith(
      '策略已导入：文件访问控制重新映射或拔插后生效，白名单重新拔插后生效',
    )
  })

  it('导入成功但任一刷新失败时等待另一侧且不误报导入失败', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([9]))
    vi.spyOn(useFilePolicyStore(), 'load').mockRejectedValue(new Error('load failed'))
    const whitelistDone = deferred<void>()
    vi.spyOn(useWhitelistStore(), 'listWhitelist').mockReturnValue(whitelistDone.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    expect(ElMessage.error).not.toHaveBeenCalled()
    whitelistDone.resolve()
    await flushPromises()
    expect(ElMessage.warning).toHaveBeenCalledWith('策略已导入，但状态刷新失败，请稍后重试')
  })

  it('导入失败不刷新 store，原始错误统一脱敏', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/private/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([1]))
    vi.mocked(importPolicy).mockRejectedValue(new Error('/private bytes command'))
    const fileLoad = vi.spyOn(useFilePolicyStore(), 'load')
    const whitelistLoad = vi.spyOn(useWhitelistStore(), 'listWhitelist')
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    expect(fileLoad).not.toHaveBeenCalled()
    expect(whitelistLoad).not.toHaveBeenCalled()
    expect(ElMessage.error).toHaveBeenCalledWith('策略导入失败')
    expect(revokeFileAccess).toHaveBeenCalledWith('/private/policy.bin')
  })

  it('断线点击和文件对话框返回后均不通信', async () => {
    useConnectionStore().updateStatus('DISCONNECTED')
    const wrapper = mountPage()
    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    expect(saveFile).not.toHaveBeenCalled()
    expect(ElMessage.warning).toHaveBeenCalled()

    useConnectionStore().updateStatus('CONNECTED')
    const pendingOpen = deferred<{ canceled: boolean; filePaths: string[] }>()
    openFile.mockReturnValue(pendingOpen.promise)
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    useConnectionStore().updateStatus('DISCONNECTED')
    pendingOpen.resolve({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    await flushPromises()
    expect(ElMessageBox.confirm).not.toHaveBeenCalled()
    expect(readFile).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('确认边界断线后不读文件，并阻止导入导出并发', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    const pendingConfirm = deferred<string>()
    vi.mocked(ElMessageBox.confirm).mockReturnValue(pendingConfirm.promise as never)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    expect(saveFile).not.toHaveBeenCalled()
    useConnectionStore().updateStatus('DISCONNECTED')
    pendingConfirm.resolve('confirm')
    await flushPromises()
    expect(readFile).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('导出对话框返回鉴权中状态时撤权且不请求装置', async () => {
    const pendingSave = deferred<{ canceled: boolean; filePath: string }>()
    saveFile.mockReturnValue(pendingSave.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="export-policy"]').trigger('click')

    useConnectionStore().updateStatus('AUTHENTICATING')
    pendingSave.resolve({ canceled: false, filePath: '/tmp/policy.bin' })
    await flushPromises()

    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
    expect(exportPolicy).not.toHaveBeenCalled()
  })

  it('确认返回授权过期状态时不读取文件', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    const pendingConfirm = deferred<string>()
    vi.mocked(ElMessageBox.confirm).mockReturnValue(pendingConfirm.promise as never)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    useConnectionStore().updateStatus('LICENSE_EXPIRED')
    pendingConfirm.resolve('confirm')
    await flushPromises()

    expect(readFile).not.toHaveBeenCalled()
    expect(importPolicy).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('卸载后导出对话框迟到路径只撤权且不继续', async () => {
    const pendingSave = deferred<{ canceled: boolean; filePath: string }>()
    saveFile.mockReturnValue(pendingSave.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    wrapper.unmount()

    pendingSave.resolve({ canceled: false, filePath: '/tmp/late.bin' })
    await flushPromises()

    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/late.bin')
    expect(exportPolicy).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
  })

  it('卸载后导出响应迟到时撤权且不写文件', async () => {
    saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/late.bin' })
    const pendingExport = deferred<{ policyData: Uint8Array }>()
    vi.mocked(exportPolicy).mockReturnValue(pendingExport.promise as never)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await flushPromises()
    wrapper.unmount()

    pendingExport.resolve({ policyData: new Uint8Array([1]) })
    await flushPromises()

    expect(writeFile).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/late.bin')
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
  })

  it('卸载后导入确认迟到时不读文件', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    const pendingConfirm = deferred<string>()
    vi.mocked(ElMessageBox.confirm).mockReturnValue(pendingConfirm.promise as never)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    wrapper.unmount()

    pendingConfirm.resolve('confirm')
    await flushPromises()

    expect(readFile).not.toHaveBeenCalled()
    expect(importPolicy).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('卸载后文件读取迟到时不导入', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    const pendingRead = deferred<Uint8Array>()
    readFile.mockReturnValue(pendingRead.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    wrapper.unmount()

    pendingRead.resolve(new Uint8Array([1]))
    await flushPromises()

    expect(importPolicy).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('卸载后导入响应迟到时不刷新状态或提示', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([1]))
    const pendingImport = deferred<void>()
    vi.mocked(importPolicy).mockReturnValue(pendingImport.promise)
    const fileLoad = vi.spyOn(useFilePolicyStore(), 'load')
    const whitelistLoad = vi.spyOn(useWhitelistStore(), 'listWhitelist')
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()
    wrapper.unmount()

    pendingImport.resolve()
    await flushPromises()

    expect(fileLoad).not.toHaveBeenCalled()
    expect(whitelistLoad).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('文件读取失败时撤销读授权且不泄露路径', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/private/read.bin'] })
    readFile.mockRejectedValue(new Error('/private/read.bin failed'))
    revokeFileAccess.mockRejectedValue(new Error('/private/revoke failed'))
    const wrapper = mountPage()

    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    expect(revokeFileAccess).toHaveBeenCalledWith('/private/read.bin')
    expect(importPolicy).not.toHaveBeenCalled()
    expect(ElMessage.error).toHaveBeenCalledWith('策略文件读取失败')
  })

  it.each(['disconnect', 'token'] as const)('导出响应期 %s 失效时不写入并撤权', async (kind) => {
    saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/session.bin' })
    const pendingExport = deferred<{ policyData: Uint8Array }>()
    vi.mocked(exportPolicy).mockReturnValue(pendingExport.promise as never)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="export-policy"]').trigger('click')
    await flushPromises()

    if (kind === 'disconnect') {
      useConnectionStore().updateStatus('DISCONNECTED')
    } else {
      useSessionStore().token = 'new-session-token'
    }
    pendingExport.resolve({ policyData: new Uint8Array([1]) })
    await flushPromises()

    expect(exportPolicy).toHaveBeenCalledWith('session-token')
    expect(writeFile).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/session.bin')
    if (kind === 'token') {
      expect(ElMessage.warning).not.toHaveBeenCalled()
      expect(ElMessage.success).not.toHaveBeenCalled()
      expect(ElMessage.error).not.toHaveBeenCalled()
    }
  })

  it.each(['confirm', 'read'] as const)('导入 %s 边界会话变更后不继续敏感步骤', async (stage) => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/session.bin'] })
    const pendingConfirm = deferred<string>()
    const pendingRead = deferred<Uint8Array>()
    if (stage === 'confirm') {
      vi.mocked(ElMessageBox.confirm).mockReturnValue(pendingConfirm.promise as never)
    } else {
      readFile.mockReturnValue(pendingRead.promise)
    }
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    useSessionStore().token = 'new-session-token'
    if (stage === 'confirm') {
      pendingConfirm.resolve('confirm')
    } else {
      pendingRead.resolve(new Uint8Array([1]))
    }
    await flushPromises()

    if (stage === 'confirm') {
      expect(readFile).not.toHaveBeenCalled()
    }
    expect(importPolicy).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/session.bin')
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
  })

  it('导入成功后刷新期断线只提示已导入但刷新失败', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([1]))
    const fileDone = deferred<void>()
    const whitelistDone = deferred<void>()
    vi.spyOn(useFilePolicyStore(), 'load').mockReturnValue(fileDone.promise)
    vi.spyOn(useWhitelistStore(), 'listWhitelist').mockReturnValue(whitelistDone.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    useConnectionStore().updateStatus('DISCONNECTED')
    fileDone.resolve()
    whitelistDone.resolve()
    await flushPromises()

    expect(ElMessage.warning).toHaveBeenCalledWith('策略已导入，但状态刷新失败，请稍后重试')
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('导入响应迟到前会话变更时不刷新也不污染新会话提示', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([1]))
    const pendingImport = deferred<void>()
    vi.mocked(importPolicy).mockReturnValue(pendingImport.promise)
    const fileLoad = vi.spyOn(useFilePolicyStore(), 'load')
    const whitelistLoad = vi.spyOn(useWhitelistStore(), 'listWhitelist')
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    useSessionStore().token = 'new-session-token'
    pendingImport.resolve()
    await flushPromises()

    expect(importPolicy).toHaveBeenCalledWith('session-token', new Uint8Array([1]))
    expect(fileLoad).not.toHaveBeenCalled()
    expect(whitelistLoad).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.warning).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('导入响应迟到前断线时不刷新且提示已导入但刷新失败', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([1]))
    const pendingImport = deferred<void>()
    vi.mocked(importPolicy).mockReturnValue(pendingImport.promise)
    const fileLoad = vi.spyOn(useFilePolicyStore(), 'load')
    const whitelistLoad = vi.spyOn(useWhitelistStore(), 'listWhitelist')
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    useConnectionStore().updateStatus('DISCONNECTED')
    pendingImport.resolve()
    await flushPromises()

    expect(fileLoad).not.toHaveBeenCalled()
    expect(whitelistLoad).not.toHaveBeenCalled()
    expect(ElMessage.warning).toHaveBeenCalledWith('策略已导入，但状态刷新失败，请稍后重试')
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it('读取策略期断线时不发起导入并撤销授权', async () => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    const pendingRead = deferred<Uint8Array>()
    readFile.mockReturnValue(pendingRead.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    useConnectionStore().updateStatus('DISCONNECTED')
    pendingRead.resolve(new Uint8Array([1]))
    await flushPromises()

    expect(importPolicy).not.toHaveBeenCalled()
    expect(ElMessage.warning).toHaveBeenCalledWith('装置已断开连接，无法传输策略')
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it.each(['token', 'unmount'] as const)('刷新已启动后 %s 失效不污染全局提示', async (kind) => {
    openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/policy.bin'] })
    readFile.mockResolvedValue(new Uint8Array([1]))
    const fileDone = deferred<void>()
    const whitelistDone = deferred<void>()
    const fileLoad = vi.spyOn(useFilePolicyStore(), 'load').mockReturnValue(fileDone.promise)
    const whitelistLoad = vi
      .spyOn(useWhitelistStore(), 'listWhitelist')
      .mockReturnValue(whitelistDone.promise)
    const wrapper = mountPage()
    await wrapper.get('[data-testid="import-policy"]').trigger('click')
    await flushPromises()

    if (kind === 'token') {
      useSessionStore().token = 'new-session-token'
    } else {
      wrapper.unmount()
    }
    fileDone.resolve()
    whitelistDone.resolve()
    await flushPromises()

    expect(fileLoad).toHaveBeenCalledWith('session-token')
    expect(whitelistLoad).toHaveBeenCalledWith('session-token')
    expect(ElMessage.success).not.toHaveBeenCalled()
    expect(ElMessage.warning).not.toHaveBeenCalled()
    expect(ElMessage.error).not.toHaveBeenCalled()
    expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/policy.bin')
  })

  it.each(['save', 'open', 'export', 'write', 'read', 'import'] as const)(
    '%s reject 前断线时只警告一次且不显示旧错误',
    async (stage) => {
      const pending = deferred<never>()
      if (stage === 'save') {
        saveFile.mockReturnValue(pending.promise)
      } else if (stage === 'open') {
        openFile.mockReturnValue(pending.promise)
      } else if (stage === 'export') {
        saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/reject.bin' })
        vi.mocked(exportPolicy).mockReturnValue(pending.promise)
      } else if (stage === 'write') {
        saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/reject.bin' })
        writeFile.mockReturnValue(pending.promise)
      } else if (stage === 'read') {
        openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/reject.bin'] })
        readFile.mockReturnValue(pending.promise)
      } else {
        openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/reject.bin'] })
        readFile.mockResolvedValue(new Uint8Array([1]))
        vi.mocked(importPolicy).mockReturnValue(pending.promise)
      }
      const wrapper = mountPage()
      const trigger = stage === 'save' || stage === 'export' || stage === 'write'
        ? '[data-testid="export-policy"]'
        : '[data-testid="import-policy"]'
      await wrapper.get(trigger).trigger('click')
      await flushPromises()

      useConnectionStore().updateStatus('DISCONNECTED')
      pending.reject(new Error('/private/old-session failed'))
      await flushPromises()

      expect(ElMessage.error).not.toHaveBeenCalled()
      expect(ElMessage.warning).toHaveBeenCalledTimes(1)
      expect(ElMessage.warning).toHaveBeenCalledWith('装置已断开连接，无法传输策略')
      if (['export', 'write', 'read', 'import'].includes(stage)) {
        expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/reject.bin')
      }
    },
  )

  it.each([
    ['save', 'token'], ['save', 'unmount'],
    ['open', 'token'], ['open', 'unmount'],
    ['export', 'token'], ['export', 'unmount'],
    ['write', 'token'], ['write', 'unmount'],
    ['read', 'token'], ['read', 'unmount'],
    ['import', 'token'], ['import', 'unmount'],
  ] as const)('%s reject 前 %s 失效时静默', async (stage, invalidation) => {
    const pending = deferred<never>()
    if (stage === 'save') {
      saveFile.mockReturnValue(pending.promise)
    } else if (stage === 'open') {
      openFile.mockReturnValue(pending.promise)
    } else if (stage === 'export') {
      saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/stale.bin' })
      vi.mocked(exportPolicy).mockReturnValue(pending.promise)
    } else if (stage === 'write') {
      saveFile.mockResolvedValue({ canceled: false, filePath: '/tmp/stale.bin' })
      writeFile.mockReturnValue(pending.promise)
    } else if (stage === 'read') {
      openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/stale.bin'] })
      readFile.mockReturnValue(pending.promise)
    } else {
      openFile.mockResolvedValue({ canceled: false, filePaths: ['/tmp/stale.bin'] })
      readFile.mockResolvedValue(new Uint8Array([1]))
      vi.mocked(importPolicy).mockReturnValue(pending.promise)
    }
    const wrapper = mountPage()
    const trigger = stage === 'save' || stage === 'export' || stage === 'write'
      ? '[data-testid="export-policy"]'
      : '[data-testid="import-policy"]'
    await wrapper.get(trigger).trigger('click')
    await flushPromises()

    if (invalidation === 'token') {
      useSessionStore().token = 'new-session-token'
    } else {
      wrapper.unmount()
    }
    pending.reject(new Error('/private/stale failed'))
    await flushPromises()

    expect(ElMessage.error).not.toHaveBeenCalled()
    expect(ElMessage.warning).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
    if (stage === 'export' || stage === 'write' || stage === 'read' || stage === 'import') {
      expect(revokeFileAccess).toHaveBeenCalledWith('/tmp/stale.bin')
    }
  })
})
