import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { usb_control } from '../../shared/proto/usb_control'
import {
  addBlacklistExtension,
  getFilePolicy,
  removeBlacklistExtension,
  updateSwitch,
} from '@/services/file-policy-service'

export type FilePolicyKey =
  | 'exec_control'
  | 'auto_read_control'
  | 'file_type_blacklist_control'

function getErrorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
}

export const useFilePolicyStore = defineStore('file-policy', () => {
  const policy = ref<usb_control.RspFilePolicy | null>(null)
  const isLoading = ref(false)
  const errorMessage = ref('')
  const pendingKeys = ref(new Set<string>())
  const pendingOwnership = new Map<string, symbol>()
  let epoch = 0
  let nextLoadRequestId = 0
  let latestLoadRequestId = 0
  let activeLoadCount = 0

  async function load(sessionToken: string): Promise<void> {
    const loadEpoch = epoch
    const requestId = ++nextLoadRequestId
    latestLoadRequestId = requestId
    activeLoadCount += 1
    isLoading.value = true
    if (loadEpoch === epoch && requestId === latestLoadRequestId) {
      errorMessage.value = ''
    }
    try {
      const response = await getFilePolicy(sessionToken)
      if (loadEpoch === epoch && requestId === latestLoadRequestId) {
        policy.value = response
      }
    } catch (error: unknown) {
      if (loadEpoch === epoch && requestId === latestLoadRequestId) {
        errorMessage.value = getErrorMessage(error, '文件访问策略加载失败')
      }
      throw error
    } finally {
      if (loadEpoch === epoch) {
        activeLoadCount -= 1
        isLoading.value = activeLoadCount > 0
      }
    }
  }

  async function runWrite(
    pendingKey: string,
    write: () => Promise<void>,
    sessionToken: string,
  ): Promise<void> {
    if (pendingOwnership.has(pendingKey)) {
      return
    }

    const writeEpoch = epoch
    const operationToken = Symbol(pendingKey)
    pendingOwnership.set(pendingKey, operationToken)
    pendingKeys.value.add(pendingKey)
    try {
      await write()
      if (writeEpoch === epoch) {
        await load(sessionToken)
      }
    } finally {
      if (pendingOwnership.get(pendingKey) === operationToken) {
        pendingOwnership.delete(pendingKey)
        pendingKeys.value.delete(pendingKey)
      }
    }
  }

  async function setSwitch(
    sessionToken: string,
    policyKey: FilePolicyKey,
    enabled: boolean,
  ): Promise<void> {
    await runWrite(
      policyKey,
      () => updateSwitch(sessionToken, policyKey, enabled),
      sessionToken,
    )
  }

  async function addExtension(
    sessionToken: string,
    extension: string,
    description: string,
  ): Promise<void> {
    const pendingKey = `extension:${extension}`
    await runWrite(
      pendingKey,
      () => addBlacklistExtension(sessionToken, extension, description),
      sessionToken,
    )
  }

  async function removeExtension(sessionToken: string, extension: string): Promise<void> {
    const pendingKey = `extension:${extension}`
    await runWrite(
      pendingKey,
      () => removeBlacklistExtension(sessionToken, extension),
      sessionToken,
    )
  }

  function clear(): void {
    epoch += 1
    activeLoadCount = 0
    policy.value = null
    isLoading.value = false
    errorMessage.value = ''
    pendingKeys.value.clear()
    pendingOwnership.clear()
  }

  return {
    policy,
    isLoading,
    errorMessage,
    pendingKeys,
    load,
    setSwitch,
    addExtension,
    removeExtension,
    clear,
  }
})
