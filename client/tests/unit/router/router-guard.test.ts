import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { resolveRouteAccess } from '../../../src/renderer/router/guard'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import type { UserRole } from '../../../src/shared/connection-state'

function setSession(role: UserRole): void {
  useSessionStore().setSession({
    token: 'session-token',
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
