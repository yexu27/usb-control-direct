import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSessionStore } from '../../../src/renderer/stores/session'

describe('useSessionStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('starts with empty session', () => {
    const store = useSessionStore()
    expect(store.isLoggedIn).toBe(false)
    expect(store.token).toBe('')
  })

  it('setSession populates all fields', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 1234567890,
      deviceDescription: 'test-device',
    })
    expect(store.isLoggedIn).toBe(true)
    expect(store.isAuthorized).toBe(true)
    expect(store.username).toBe('admin')
    expect(store.role).toBe('admin')
  })

  it('clearSession resets all fields', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 1234567890,
      deviceDescription: 'test-device',
    })
    store.clearSession()
    expect(store.isLoggedIn).toBe(false)
    expect(store.token).toBe('')
    expect(store.username).toBe('')
  })

  it('inactivity timer clears session after 5 minutes', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    store.startInactivityTimer()

    vi.advanceTimersByTime(5 * 60 * 1000)
    expect(store.isLoggedIn).toBe(false)
  })

  it('resetInactivityTimer postpones timeout', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    store.startInactivityTimer()

    vi.advanceTimersByTime(4 * 60 * 1000)
    store.resetInactivityTimer()

    vi.advanceTimersByTime(4 * 60 * 1000)
    expect(store.isLoggedIn).toBe(true)

    vi.advanceTimersByTime(60 * 1000)
    expect(store.isLoggedIn).toBe(false)
  })

  it('stopInactivityTimer cancels pending timeout', () => {
    const store = useSessionStore()
    store.setSession({
      token: 'abc123',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    store.startInactivityTimer()
    store.stopInactivityTimer()

    vi.advanceTimersByTime(6 * 60 * 1000)
    expect(store.isLoggedIn).toBe(true)
  })
})
