import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAppToastStore } from '../../../src/renderer/stores/app-toast'

describe('useAppToastStore', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
  })

  it('shows one success toast and hides it after 2 seconds', () => {
    const store = useAppToastStore()

    store.showSuccess('策略导出成功')

    expect(store.visible).toBe(true)
    expect(store.message).toBe('策略导出成功')
    expect(store.type).toBe('success')

    vi.advanceTimersByTime(1_999)
    expect(store.visible).toBe(true)

    vi.advanceTimersByTime(1)
    expect(store.visible).toBe(false)
  })

  it('replaces existing toast instead of stacking', () => {
    const store = useAppToastStore()

    store.showSuccess('用户创建成功')
    vi.advanceTimersByTime(1_000)
    store.showSuccess('密码已重置')
    vi.advanceTimersByTime(1_500)

    expect(store.visible).toBe(true)
    expect(store.message).toBe('密码已重置')

    vi.advanceTimersByTime(500)
    expect(store.visible).toBe(false)
  })
})
