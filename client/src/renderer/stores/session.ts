import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { UserRole, AuthStatus } from '../../shared/connection-state'
import { useConnectionStore } from './connection'
import { useBootstrapStore } from './bootstrap'
import {
  login as requestLogin,
  logout as requestLogout,
  queryAuthStatus,
} from '@/services/auth-service'
import { ServiceError } from '@/services/send-command'

const INACTIVITY_TIMEOUT = 5 * 60 * 1000

export interface LoginResult {
  success: boolean
  resultCode: number
  errorMessage: string
}

export type ReconnectValidationResult = 'resumable' | 'login-required'

function parseUserRole(role: string): UserRole {
  if (role === 'admin' || role === 'operator' || role === 'auditor') {
    return role
  }
  throw new Error(`装置返回了未知角色：${role}`)
}

function parseAuthStatus(authStatus: string): AuthStatus {
  if (
    authStatus === 'authorized' ||
    authStatus === 'unauthorized' ||
    authStatus === 'expired' ||
    authStatus === 'failed'
  ) {
    return authStatus
  }
  return ''
}

function shouldKeepConnectionAfterLoginError(error: unknown): boolean {
  return error instanceof ServiceError && error.kind === 'business'
}

export const useSessionStore = defineStore('session', () => {
  const token = ref('')
  const username = ref('')
  const role = ref<UserRole | ''>('')
  const authStatus = ref<AuthStatus>('')
  const authExpireTime = ref(0)
  const deviceDescription = ref('')

  const isLoggedIn = computed(() => token.value !== '')
  const isAuthorized = computed(() => authStatus.value === 'authorized')

  let inactivityTimer: ReturnType<typeof setTimeout> | null = null

  function clearSession(): void {
    token.value = ''
    username.value = ''
    role.value = ''
    authStatus.value = ''
    authExpireTime.value = 0
    deviceDescription.value = ''
    stopInactivityTimer()
  }

  function setSession(data: {
    token: string
    username: string
    role: UserRole
    authStatus: AuthStatus
    authExpireTime: number
    deviceDescription: string
  }): void {
    token.value = data.token
    username.value = data.username
    role.value = data.role
    authStatus.value = data.authStatus
    authExpireTime.value = data.authExpireTime
    deviceDescription.value = data.deviceDescription
  }

  async function finishAuthentication(): Promise<void> {
    const connection = useConnectionStore()

    await connection.applyStateEvent('AUTH_SUCCESS')

    if (authStatus.value === 'unauthorized' || authStatus.value === 'failed') {
      await connection.applyStateEvent('LICENSE_UNAUTHORIZED')
      return
    }

    if (authStatus.value === 'expired') {
      await connection.applyStateEvent('LICENSE_EXPIRED')
      return
    }

    if (authStatus.value !== 'authorized' || role.value === '') {
      await connection.applyStateEvent('LICENSE_UNAUTHORIZED')
      return
    }

    await connection.applyStateEvent('LICENSE_AUTHORIZED')
    await connection.applyStateEvent('CONFIG_LOADED')
    startInactivityTimer()
  }

  async function login(
    ip: string,
    loginUsername: string,
    password: string,
  ): Promise<LoginResult> {
    const connection = useConnectionStore()
    try {
      await connection.connect(ip)
      const response = await requestLogin(loginUsername, password)
      setSession({
        token: response.sessionToken,
        username: response.username,
        role: parseUserRole(response.role),
        authStatus: parseAuthStatus(response.authStatus) || (response.authorized ? 'authorized' : ''),
        authExpireTime: Number(response.authExpireTime),
        deviceDescription: response.deviceDescription,
      })
      await finishAuthentication()
      return { success: true, resultCode: 0, errorMessage: '' }
    } catch (error: unknown) {
      try {
        await connection.applyStateEvent('AUTH_FAIL')
      } catch {
        // 连接错误可能已由主进程状态机处理。
      }
      if (!shouldKeepConnectionAfterLoginError(error)) {
        await connection.disconnect(true).catch(() => {})
      }
      clearSession()
      throw error
    }
  }

  async function logout(): Promise<void> {
    const connection = useConnectionStore()
    const bootstrapStore = useBootstrapStore()
    const currentToken = token.value

    try {
      if (currentToken !== '' && !connection.isDisconnected) {
        await requestLogout(currentToken)
      }
    } catch (error: unknown) {
      if (!(error instanceof ServiceError && error.kind === 'unauthenticated')) {
        console.warn('远端登出未确认，执行本地会话清理', error)
      }
    } finally {
      bootstrapStore.clear()
      clearSession()
      await connection.disconnect(true).catch(() => {})
    }
  }

  async function clearReconnectState(
    connection = useConnectionStore(),
    bootstrapStore = useBootstrapStore(),
  ): Promise<void> {
    bootstrapStore.clear()
    clearSession()
    await connection.disconnect(true).catch(() => {})
  }

  async function validateReconnectSession(): Promise<ReconnectValidationResult> {
    if (token.value === '') {
      await clearReconnectState()
      return 'login-required'
    }

    const connection = useConnectionStore()
    const bootstrapStore = useBootstrapStore()

    try {
      const response = await queryAuthStatus(token.value)
      authStatus.value = parseAuthStatus(response.authStatus) || (response.authorized ? 'authorized' : '')
      authExpireTime.value = Number(response.expireTime)
      deviceDescription.value = response.deviceDescription
      await finishAuthentication()

      if (authStatus.value !== 'authorized' || role.value === '' || !connection.isConnected) {
        await clearReconnectState(connection, bootstrapStore)
        return 'login-required'
      }

      return 'resumable'
    } catch (error: unknown) {
      if (error instanceof ServiceError && error.kind === 'unauthenticated') {
        await clearReconnectState(connection, bootstrapStore)
        return 'login-required'
      }
      throw error
    }
  }

  async function reconnectAndValidate(): Promise<ReconnectValidationResult> {
    if (token.value === '') {
      await clearReconnectState()
      return 'login-required'
    }
    const connection = useConnectionStore()
    const reconnected = await connection.reconnect().catch(() => false)
    if (!reconnected) {
      await clearReconnectState(connection)
      return 'login-required'
    }
    return validateReconnectSession()
  }

  function startInactivityTimer(): void {
    stopInactivityTimer()
    inactivityTimer = setTimeout(() => {
      void logout()
    }, INACTIVITY_TIMEOUT)
  }

  function resetInactivityTimer(): void {
    if (inactivityTimer != null) {
      startInactivityTimer()
    }
  }

  function stopInactivityTimer(): void {
    if (inactivityTimer != null) {
      clearTimeout(inactivityTimer)
      inactivityTimer = null
    }
  }

  return {
    token,
    username,
    role,
    authStatus,
    authExpireTime,
    deviceDescription,
    isLoggedIn,
    isAuthorized,
    login,
    logout,
    reconnectAndValidate,
    clearSession,
    setSession,
    startInactivityTimer,
    resetInactivityTimer,
    stopInactivityTimer,
  }
})
