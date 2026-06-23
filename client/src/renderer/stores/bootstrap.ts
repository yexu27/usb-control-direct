import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { usb_control } from '../../shared/proto/usb_control'
import type { UserRole } from '../../shared/connection-state'
import { useFilePolicyStore } from './file-policy'
import { useWhitelistStore } from './whitelist'
import { getSystemInfo } from '@/services/system-service'
import { listUsers } from '@/services/user-service'

type LoadResult =
  | { success: true }
  | { success: false; error: unknown }

async function captureLoad(load: Promise<void>): Promise<LoadResult> {
  try {
    await load
    return { success: true }
  } catch (error: unknown) {
    return { success: false, error }
  }
}

export const useBootstrapStore = defineStore('bootstrap', () => {
  const systemInfo = ref<usb_control.RspSystemInfo | null>(null)
  const users = ref<usb_control.RspListUsers | null>(null)

  function clear(): void {
    useFilePolicyStore().clear()
    useWhitelistStore().clear()
    systemInfo.value = null
    users.value = null
  }

  async function loadForRole(sessionToken: string, role: UserRole): Promise<void> {
    clear()

    if (role === 'operator') {
      const [filePolicyResult, whitelistResult] = await Promise.all([
        captureLoad(useFilePolicyStore().load(sessionToken)),
        captureLoad(useWhitelistStore().listWhitelist(sessionToken)),
      ])
      if (!filePolicyResult.success) {
        throw filePolicyResult.error
      }
      if (!whitelistResult.success) {
        throw whitelistResult.error
      }
      return
    }

    if (role === 'admin') {
      const [systemInfoResponse, usersResponse] = await Promise.all([
        getSystemInfo(sessionToken),
        listUsers(sessionToken),
      ])
      systemInfo.value = systemInfoResponse
      users.value = usersResponse
    }
  }

  return {
    systemInfo,
    users,
    clear,
    loadForRole,
  }
})
