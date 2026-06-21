import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useConnectionStore } from '../../../src/renderer/stores/connection'

describe('useConnectionStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('starts with DISCONNECTED status', () => {
    const store = useConnectionStore()
    expect(store.status).toBe('DISCONNECTED')
    expect(store.isDisconnected).toBe(true)
    expect(store.isConnected).toBe(false)
  })

  it('updateStatus changes status and computed values', () => {
    const store = useConnectionStore()
    store.updateStatus('CONNECTED')
    expect(store.status).toBe('CONNECTED')
    expect(store.isConnected).toBe(true)
    expect(store.isDisconnected).toBe(false)
  })
})
