import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useUiStore } from '../../../src/renderer/stores/ui'

describe('useUiStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('toggles global loading', () => {
    const store = useUiStore()
    expect(store.isGlobalLoading).toBe(false)
    store.showLoading()
    expect(store.isGlobalLoading).toBe(true)
    store.hideLoading()
    expect(store.isGlobalLoading).toBe(false)
  })

  it('adds and removes toasts', () => {
    const store = useUiStore()
    store.showToast('操作成功', 'success')
    expect(store.toastQueue.length).toBe(1)
    expect(store.toastQueue[0].message).toBe('操作成功')

    store.removeToast(store.toastQueue[0].id)
    expect(store.toastQueue.length).toBe(0)
  })
})
