import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useConnectedOperationGuard } from '../../../src/renderer/composables/use-connected-operation-guard'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { showErrorDialog } from '../../../src/renderer/utils/operation-feedback'
import type { ConnectionStatus } from '../../../src/shared/connection-state'

vi.mock('../../../src/renderer/utils/operation-feedback', () => ({
  showErrorDialog: vi.fn().mockResolvedValue(undefined),
}))

describe('useConnectedOperationGuard', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('CONNECTED 状态允许业务读写且不提示错误', async () => {
    useConnectionStore().updateStatus('CONNECTED')

    const guard = useConnectedOperationGuard()

    expect(guard.isBusinessActionDisabled.value).toBe(false)
    expect(guard.canReadFromDevice()).toBe(true)
    await expect(guard.canWriteToDevice()).resolves.toBe(true)
    expect(showErrorDialog).not.toHaveBeenCalled()
  })

  it.each([
    'DISCONNECTED',
    'AUTH_REQUIRED',
    'LICENSE_EXPIRED',
  ] satisfies ConnectionStatus[])('%s 状态禁用业务操作并拦截读写', async (status) => {
    useConnectionStore().updateStatus(status)

    const guard = useConnectedOperationGuard()

    expect(guard.isBusinessActionDisabled.value).toBe(true)
    expect(guard.canReadFromDevice()).toBe(false)
    await expect(guard.canWriteToDevice()).resolves.toBe(false)
    expect(showErrorDialog).toHaveBeenCalledWith('操作失败', '装置已断开连接，无法操作')
  })
})
