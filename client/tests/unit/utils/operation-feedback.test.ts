import { describe, expect, it, vi } from 'vitest'
import { ElMessage } from 'element-plus'
import { alertAction } from '@/utils/confirm-action'
import { errorMessage, showErrorDialog, showSuccessToast } from '@/utils/operation-feedback'

vi.mock('element-plus', async () => {
  const actual = await vi.importActual<typeof import('element-plus')>('element-plus')
  return {
    ...actual,
    ElMessage: {
      success: vi.fn(),
    },
  }
})

vi.mock('@/utils/confirm-action', () => ({
  alertAction: vi.fn().mockResolvedValue(undefined),
}))

describe('operation-feedback', () => {
  it('shows success toast at top center for 2 seconds', () => {
    showSuccessToast('操作成功')

    expect(ElMessage.success).toHaveBeenCalledWith({
      message: '操作成功',
      duration: 2000,
      offset: 24,
      showClose: false,
      grouping: true,
    })
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
