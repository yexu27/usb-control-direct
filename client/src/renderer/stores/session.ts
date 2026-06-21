import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { UserRole, AuthStatus } from '../../shared/connection-state'
import { useConnectionStore } from './connection'

const INACTIVITY_TIMEOUT = 5 * 60 * 1000

export interface LoginResult {
  success: boolean
  resultCode: number
  errorMessage: string
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

  /**
   * 发起登录请求。
   * 注意：此方法为占位，实际 protobuf 编解码将在 Task 11 Services 层实现。
   */
  async function login(_ip: string, _loginUsername: string, _password: string): Promise<LoginResult> {
    throw new Error('Not implemented — see Task 11 services layer')
  }

  /**
   * 发起登出请求。
   * 注意：此方法为占位，实际 protobuf 编解码将在 Task 11 Services 层实现。
   */
  async function logout(): Promise<void> {
    throw new Error('Not implemented — see Task 11 services layer')
  }

  /**
   * 校验当前会话有效性。
   * 注意：此方法为占位，实际 protobuf 编解码将在 Task 11 Services 层实现。
   */
  async function validateSession(): Promise<boolean> {
    throw new Error('Not implemented — see Task 11 services layer')
  }

  function startInactivityTimer(): void {
    stopInactivityTimer()
    inactivityTimer = setTimeout(() => {
      const connection = useConnectionStore()
      clearSession()
      if (connection.isConnected) {
        connection.disconnect()
      }
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
    validateSession,
    clearSession,
    setSession,
    startInactivityTimer,
    resetInactivityTimer,
    stopInactivityTimer,
  }
})
