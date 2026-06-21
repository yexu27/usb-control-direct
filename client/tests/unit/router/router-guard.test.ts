import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { resolveRouteAccess } from '../../../src/renderer/router/guard'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import type { UserRole } from '../../../src/shared/connection-state'

function setSession(role: UserRole, token = 'session-token'): void {
  useSessionStore().setSession({
    token,
    username: role,
    role,
    authStatus: 'authorized',
    authExpireTime: 0,
    deviceDescription: '',
  })
}

describe('resolveRouteAccess', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('允许进入登录前公共页面', () => {
    expect(resolveRouteAccess({ guest: true })).toBe(true)
  })

  it('未登录时跳转登录页', () => {
    expect(resolveRouteAccess({ requiresAuth: true })).toBe('/login')
  })

  it('授权未完成时跳转授权页', () => {
    setSession('operator')
    useConnectionStore().updateStatus('AUTH_REQUIRED')

    expect(resolveRouteAccess({ requiresAuth: false })).toBe('/license')
  })

  it('有效临时会话在未授权状态可进入授权页', () => {
    setSession('operator', 'temporary-token')
    useConnectionStore().updateStatus('AUTH_REQUIRED')

    expect(resolveRouteAccess({ licenseFlow: true })).toBe(true)
  })

  it('有效临时会话在授权到期状态可进入授权页续期', () => {
    setSession('admin', 'temporary-token')
    useConnectionStore().updateStatus('LICENSE_EXPIRED')

    expect(resolveRouteAccess({ licenseFlow: true })).toBe(true)
  })

  it('无临时会话访问授权页时跳转登录页', () => {
    useConnectionStore().updateStatus('AUTH_REQUIRED')

    expect(resolveRouteAccess({ licenseFlow: true })).toBe('/login')
  })

  it('已授权用户直接访问授权页时跳转角色默认页', () => {
    setSession('operator')
    useConnectionStore().updateStatus('CONNECTED')

    expect(resolveRouteAccess({ licenseFlow: true })).toBe('/file-access')
  })

  it('角色无权访问时跳转该角色默认页', () => {
    setSession('auditor')
    useConnectionStore().updateStatus('CONNECTED')

    expect(resolveRouteAccess({ requiresAuth: true, roles: ['admin'] })).toBe('/logs')
  })

  it('角色有权访问时允许导航', () => {
    setSession('admin')
    useConnectionStore().updateStatus('CONNECTED')

    expect(resolveRouteAccess({ requiresAuth: true, roles: ['admin'] })).toBe(true)
  })
})
