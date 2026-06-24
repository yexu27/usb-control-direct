import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { alertAction } from '@/utils/confirm-action'
import { useAppToastStore } from '@/stores/app-toast'
import { errorMessage, showErrorDialog, showSuccessToast } from '@/utils/operation-feedback'

vi.mock('@/utils/confirm-action', () => ({
  alertAction: vi.fn().mockResolvedValue(undefined),
}))

describe('operation-feedback', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('shows success toast through app toast store', () => {
    showSuccessToast('操作成功')

    const toast = useAppToastStore()
    expect(toast.visible).toBe(true)
    expect(toast.message).toBe('操作成功')
    expect(toast.type).toBe('success')
  })

  it('shows centered error dialog', async () => {
    await showErrorDialog('操作失败', '失败原因')

    expect(alertAction).toHaveBeenCalledWith({
      title: '操作失败',
      message: '失败原因',
      type: 'error',
    })
  })

  it('extracts Error message before fallback', () => {
    expect(errorMessage(new Error('服务端失败'), '默认失败')).toBe('服务端失败')
    expect(errorMessage('bad', '默认失败')).toBe('默认失败')
  })
})
