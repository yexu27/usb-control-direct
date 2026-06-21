import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { login, logout, queryAuthStatus } from '../../../src/renderer/services/auth-service'
import { listWhitelist } from '../../../src/renderer/services/whitelist-service'
import { getFilePolicy } from '../../../src/renderer/services/file-policy-service'
import { usb_control } from '../../../src/shared/proto/usb_control'

vi.mock('../../../src/renderer/services/auth-service', () => ({
  login: vi.fn(),
  logout: vi.fn(),
  queryAuthStatus: vi.fn(),
}))

vi.mock('../../../src/renderer/services/whitelist-service', () => ({
  listWhitelist: vi.fn(),
}))

vi.mock('../../../src/renderer/services/file-policy-service', () => ({
  getFilePolicy: vi.fn(),
}))

vi.mock('../../../src/renderer/services/system-service', () => ({
  getSystemInfo: vi.fn(),
}))

vi.mock('../../../src/renderer/services/user-service', () => ({
  listUsers: vi.fn(),
}))

const loginMock = vi.mocked(login)
const logoutMock = vi.mocked(logout)
const queryAuthStatusMock = vi.mocked(queryAuthStatus)
const listWhitelistMock = vi.mocked(listWhitelist)
const getFilePolicyMock = vi.mocked(getFilePolicy)
const connect = vi.fn().mockResolvedValue(undefined)
const disconnect = vi.fn().mockResolvedValue(undefined)
const applyStateEvent = vi.fn().mockResolvedValue(undefined)

describe('useSessionStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
    vi.clearAllMocks()
    window.desktopApi = {
      tls: {
        connect,
        disconnect,
        applyStateEvent,
      },
    } as unknown as Window['desktopApi']
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

  it('登录成功后按操作员角色加载配置并进入 CONNECTED', async () => {
    loginMock.mockResolvedValue(usb_control.RspLogin.fromObject({
      success: true,
      sessionToken: 'token',
      username: 'operator',
      role: 'operator',
      authorized: true,
      authExpireTime: 0,
      deviceDescription: 'USB_DEVICE',
      resultCode: 0,
      errorMessage: '',
      authStatus: 'authorized',
    }))
    listWhitelistMock.mockResolvedValue(usb_control.RspListWhitelist.fromObject({ devices: [] }))
    getFilePolicyMock.mockResolvedValue(usb_control.RspFilePolicy.fromObject({
      execControlEnabled: false,
      autoReadControlEnabled: false,
      fileTypeBlacklistEnabled: false,
      blacklist: [],
    }))
    const store = useSessionStore()

    await store.login('19.19.19.16', 'operator', 'operator@123')

    expect(store.token).toBe('token')
    expect(listWhitelistMock).toHaveBeenCalledWith('token')
    expect(getFilePolicyMock).toHaveBeenCalledWith('token')
    expect(applyStateEvent.mock.calls.map(([event]) => event)).toEqual([
      'AUTH_SUCCESS',
      'LICENSE_AUTHORIZED',
      'CONFIG_LOADED',
    ])
  })

  it('主动登出远端失败时仍清空本地状态并断开连接', async () => {
    const store = useSessionStore()
    store.setSession({
      token: 'token',
      username: 'admin',
      role: 'admin',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    useConnectionStore().updateStatus('CONNECTED')
    logoutMock.mockRejectedValue(new Error('network error'))

    await store.logout()

    expect(store.isLoggedIn).toBe(false)
    expect(disconnect).toHaveBeenCalledTimes(1)
  })

  it('重连校验授权状态后恢复会话', async () => {
    const store = useSessionStore()
    store.setSession({
      token: 'token',
      username: 'auditor',
      role: 'auditor',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    queryAuthStatusMock.mockResolvedValue(usb_control.RspAuthStatus.fromObject({
      authorized: true,
      expireTime: 0,
      deviceDescription: 'USB_DEVICE',
      authStatus: 'authorized',
    }))

    await expect(store.validateSession()).resolves.toBe(true)
    expect(applyStateEvent).toHaveBeenCalledWith('AUTH_SUCCESS')
  })
})
