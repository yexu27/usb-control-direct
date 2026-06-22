import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useFilePolicyStore } from '../../../src/renderer/stores/file-policy'
import { useWhitelistStore } from '../../../src/renderer/stores/whitelist'
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

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
}

function createDeferred<T>(): Deferred<T> {
  let resolve: (value: T) => void = () => {
    throw new Error('deferred 尚未初始化')
  }
  const promise = new Promise<T>((promiseResolve) => {
    resolve = promiseResolve
  })
  return { promise, resolve }
}

async function flushMicrotasks(): Promise<void> {
  for (let index = 0; index < 20; index += 1) {
    await Promise.resolve()
  }
}

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

  it('操作员配置加载失败时清空会话并断开连接', async () => {
    loginMock.mockResolvedValue(usb_control.RspLogin.fromObject({
      success: true,
      sessionToken: 'token',
      username: 'operator',
      role: 'operator',
      authorized: true,
      authStatus: 'authorized',
    }))
    listWhitelistMock.mockResolvedValue(usb_control.RspListWhitelist.fromObject({ devices: [] }))
    getFilePolicyMock.mockRejectedValue(new Error('配置加载失败'))
    const store = useSessionStore()
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'

    await expect(store.login('19.19.19.16', 'operator', 'operator@123')).rejects.toThrow(
      '配置加载失败',
    )

    expect(store.token).toBe('')
    expect(connection.deviceIp).toBe('')
    expect(disconnect).toHaveBeenCalledTimes(1)
    expect(disconnect).toHaveBeenCalledWith()
    expect(applyStateEvent.mock.calls.map(([event]) => event)).toEqual([
      'AUTH_SUCCESS',
      'LICENSE_AUTHORIZED',
      'CONFIG_FAILED',
      'AUTH_FAIL',
    ])
  })

  it.each([
    ['文件策略快速失败', 'file-policy'],
    ['白名单快速失败', 'whitelist'],
  ] as const)('%s 时等待另一路加载结束再清理', async (_name, failedDomain) => {
    const loadError = new Error(`${failedDomain} 加载失败`)
    const filePolicyDeferred = createDeferred<usb_control.RspFilePolicy>()
    const whitelistDeferred = createDeferred<usb_control.RspListWhitelist>()
    loginMock.mockResolvedValue(usb_control.RspLogin.fromObject({
      success: true,
      sessionToken: 'token',
      username: 'operator',
      role: 'operator',
      authorized: true,
      authStatus: 'authorized',
    }))
    getFilePolicyMock.mockImplementation(() =>
      failedDomain === 'file-policy'
        ? Promise.reject(loadError)
        : filePolicyDeferred.promise,
    )
    listWhitelistMock.mockImplementation(() =>
      failedDomain === 'whitelist'
        ? Promise.reject(loadError)
        : whitelistDeferred.promise,
    )
    const store = useSessionStore()
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'

    const loginPromise = store.login('19.19.19.16', 'operator', 'operator@123')
    let isLoginSettled = false
    void loginPromise.then(
      () => {
        isLoginSettled = true
      },
      () => {
        isLoginSettled = true
      },
    )
    await flushMicrotasks()

    expect(getFilePolicyMock).toHaveBeenCalledWith('token')
    expect(listWhitelistMock).toHaveBeenCalledWith('token')
    expect(isLoginSettled).toBe(false)
    expect(disconnect).not.toHaveBeenCalled()

    if (failedDomain === 'file-policy') {
      whitelistDeferred.resolve(
        usb_control.RspListWhitelist.fromObject({
          devices: [usb_control.WhitelistDevice.fromObject({ serialNumber: 'SN-late' })],
        }),
      )
    } else {
      filePolicyDeferred.resolve(
        usb_control.RspFilePolicy.fromObject({
          execControlEnabled: true,
        }),
      )
    }

    await expect(loginPromise).rejects.toBe(loadError)

    expect(useFilePolicyStore().policy).toBeNull()
    expect(useWhitelistStore().devices).toEqual([])
    expect(store.token).toBe('')
    expect(connection.deviceIp).toBe('')
    expect(disconnect).toHaveBeenCalledTimes(1)
    expect(disconnect).toHaveBeenCalledWith()
    expect(applyStateEvent.mock.calls.map(([event]) => event)).toEqual([
      'AUTH_SUCCESS',
      'LICENSE_AUTHORIZED',
      'CONFIG_FAILED',
      'AUTH_FAIL',
    ])
  })

  it('操作员两路加载均失败时按文件策略优先级抛出原始错误', async () => {
    const filePolicyError = new Error('文件策略加载失败')
    const whitelistError = new Error('白名单加载失败')
    loginMock.mockResolvedValue(usb_control.RspLogin.fromObject({
      success: true,
      sessionToken: 'token',
      username: 'operator',
      role: 'operator',
      authorized: true,
      authStatus: 'authorized',
    }))
    getFilePolicyMock.mockRejectedValue(filePolicyError)
    listWhitelistMock.mockRejectedValue(whitelistError)
    const store = useSessionStore()

    await expect(
      store.login('19.19.19.16', 'operator', 'operator@123'),
    ).rejects.toBe(filePolicyError)
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
})
