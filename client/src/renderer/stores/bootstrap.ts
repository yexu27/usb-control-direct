import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { usb_control } from '../../shared/proto/usb_control'
import type { UserRole } from '../../shared/connection-state'
import { listWhitelist } from '@/services/whitelist-service'
import { getFilePolicy } from '@/services/file-policy-service'
import { getSystemInfo } from '@/services/system-service'
import { listUsers } from '@/services/user-service'

export const useBootstrapStore = defineStore('bootstrap', () => {
  const whitelist = ref<usb_control.RspListWhitelist | null>(null)
  const filePolicy = ref<usb_control.RspFilePolicy | null>(null)
  const systemInfo = ref<usb_control.RspSystemInfo | null>(null)
  const users = ref<usb_control.RspListUsers | null>(null)

  function clear(): void {
    whitelist.value = null
    filePolicy.value = null
    systemInfo.value = null
    users.value = null
  }

  async function loadForRole(sessionToken: string, role: UserRole): Promise<void> {
    clear()

    if (role === 'operator') {
      const [whitelistResponse, filePolicyResponse] = await Promise.all([
        listWhitelist(sessionToken),
        getFilePolicy(sessionToken),
      ])
      whitelist.value = whitelistResponse
      filePolicy.value = filePolicyResponse
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
    whitelist,
    filePolicy,
    systemInfo,
    users,
    clear,
    loadForRole,
  }
})
