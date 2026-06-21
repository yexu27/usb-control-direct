import { describe, it, expect, beforeEach, vi } from 'vitest'
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

  it('setupListener only subscribes once and mirrors connection state', () => {
    let stateListener: ((status: 'CONNECTED') => void) | undefined
    const onStateChanged = vi.fn((listener: (status: 'CONNECTED') => void) => {
      stateListener = listener
      return vi.fn()
    })
    window.desktopApi = {
      tls: { onStateChanged },
    } as unknown as Window['desktopApi']

    const store = useConnectionStore()
    store.setupListener()
    store.setupListener()
    stateListener?.('CONNECTED')

    expect(onStateChanged).toHaveBeenCalledTimes(1)
    expect(store.status).toBe('CONNECTED')
    expect(store.wasConnected).toBe(true)
  })
})
