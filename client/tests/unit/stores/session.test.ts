import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { login, logout, queryAuthStatus } from '../../../src/renderer/services/auth-service'
import { listWhitelist } from '../../../src/renderer/services/whitelist-service'
import { getFilePolicy } from '../../../src/renderer/services/file-policy-service'
import { ServiceError } from '../../../src/renderer/services/send-command'
import { usb_control } from '../../../src/shared/proto/usb_control'

vi.mock('../../../src/renderer/services/auth-service', () => ({
  login: vi.fn(),
  logout: vi.fn(),
  queryAuthStatus: vi.fn(),
}))

vi.mock('../../../src/renderer/services/whitelist-service', () => ({
  addWhitelist: vi.fn(),
  listWhitelist: vi.fn(),
  removeWhitelist: vi.fn(),
  updateWhitelist: vi.fn(),
}))

vi.mock('../../../src/renderer/services/file-policy-service', () => ({
  addBlacklistExtension: vi.fn(),
  getFilePolicy: vi.fn(),
  removeBlacklistExtension: vi.fn(),
  updateSwitch: vi.fn(),
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

  it('登录成功后只建立会话状态，不预拉页面业务数据', async () => {
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
    listWhitelistMock.mockRejectedValue(new Error('白名单不应在登录阶段加载'))
    getFilePolicyMock.mockRejectedValue(new Error('文件策略不应在登录阶段加载'))
    const store = useSessionStore()

    await store.login('19.19.19.16', 'operator', 'operator@123')

    expect(store.token).toBe('token')
    expect(store.username).toBe('operator')
    expect(store.role).toBe('operator')
    expect(listWhitelistMock).not.toHaveBeenCalled()
    expect(getFilePolicyMock).not.toHaveBeenCalled()
    expect(applyStateEvent.mock.calls.map(([event]) => event)).toEqual([
      'AUTH_SUCCESS',
      'LICENSE_AUTHORIZED',
      'CONFIG_LOADED',
    ])
  })

  it('业务页面数据接口失败不会阻断登录流程', async () => {
    loginMock.mockResolvedValue(usb_control.RspLogin.fromObject({
      success: true,
      sessionToken: 'token',
      username: 'admin',
      role: 'admin',
      authorized: true,
      authStatus: 'authorized',
    }))
    listWhitelistMock.mockRejectedValue(new Error('business data failed'))
    getFilePolicyMock.mockRejectedValue(new Error('business data failed'))
    const store = useSessionStore()

    await expect(store.login('19.19.19.16', 'admin', 'admin@123')).resolves.toEqual({
      success: true,
      resultCode: 0,
      errorMessage: '',
    })
    expect(store.token).toBe('token')
    expect(listWhitelistMock).not.toHaveBeenCalled()
    expect(getFilePolicyMock).not.toHaveBeenCalled()
    expect(disconnect).not.toHaveBeenCalled()
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
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'
    connection.updateStatus('CONNECTED')
    logoutMock.mockRejectedValue(new Error('network error'))

    await store.logout()

    expect(store.isLoggedIn).toBe(false)
    expect(connection.deviceIp).toBe('')
    expect(disconnect).toHaveBeenCalledTimes(1)
    expect(disconnect).toHaveBeenCalledWith()
  })

  it('认证失败后重试密码时复用当前 TLS 连接', async () => {
    loginMock
      .mockRejectedValueOnce(new Error('用户名或密码错误'))
      .mockRejectedValueOnce(new Error('用户名或密码错误'))
    const store = useSessionStore()

    await expect(store.login('19.19.19.16', 'admin', 'wrong-1')).rejects.toThrow()
    await expect(store.login('19.19.19.16', 'admin', 'wrong-2')).rejects.toThrow()

    expect(connect).toHaveBeenCalledTimes(1)
    expect(loginMock).toHaveBeenCalledTimes(2)
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

  it('断线后重新建链并校验当前会话', async () => {
    const store = useSessionStore()
    store.setSession({
      token: 'token',
      username: 'auditor',
      role: 'auditor',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'
    connection.updateStatus('DISCONNECTED')
    queryAuthStatusMock.mockResolvedValue(usb_control.RspAuthStatus.fromObject({
      authorized: true,
      expireTime: 123,
      deviceDescription: 'USB_DEVICE',
      authStatus: 'authorized',
    }))

    await expect(store.reconnectAndValidate()).resolves.toBe(true)

    expect(connect).toHaveBeenCalledWith('19.19.19.16')
    expect(queryAuthStatusMock).toHaveBeenCalledWith('token')
    expect(store.deviceDescription).toBe('USB_DEVICE')
    expect(applyStateEvent.mock.calls.map(([event]) => event)).toEqual([
      'AUTH_SUCCESS',
      'LICENSE_AUTHORIZED',
      'CONFIG_LOADED',
    ])
  })

  it('重连后发现会话失效时清空状态并断开连接', async () => {
    const store = useSessionStore()
    store.setSession({
      token: 'token',
      username: 'auditor',
      role: 'auditor',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'
    connection.updateStatus('DISCONNECTED')
    queryAuthStatusMock.mockRejectedValue(new ServiceError('会话已失效', 0x1001, 'unauthenticated'))

    await expect(store.reconnectAndValidate()).resolves.toBe(false)

    expect(connect).toHaveBeenCalledWith('19.19.19.16')
    expect(store.token).toBe('')
    expect(connection.deviceIp).toBe('')
    expect(disconnect).toHaveBeenCalledTimes(1)
    expect(disconnect).toHaveBeenCalledWith()
  })

  it('重新建链失败时保留当前会话状态', async () => {
    const store = useSessionStore()
    store.setSession({
      token: 'token',
      username: 'auditor',
      role: 'auditor',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: 'USB_DEVICE',
    })
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'
    connection.updateStatus('DISCONNECTED')
    connect.mockRejectedValueOnce(new Error('connect failed'))

    await expect(store.reconnectAndValidate()).rejects.toThrow(
      'USB 管控装置重新连接失败，请检查网络或设备连接。',
    )

    expect(queryAuthStatusMock).not.toHaveBeenCalled()
    expect(store.token).toBe('token')
    expect(store.deviceDescription).toBe('USB_DEVICE')
    expect(connection.deviceIp).toBe('19.19.19.16')
  })
})
