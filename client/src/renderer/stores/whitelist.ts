import { ref } from 'vue'
import { defineStore } from 'pinia'
import { usb_control } from '../../shared/proto/usb_control'
import {
  addWhitelist as requestAddWhitelist,
  listWhitelist as requestListWhitelist,
  removeWhitelist as requestRemoveWhitelist,
  updateWhitelist as requestUpdateWhitelist,
  type AddWhitelistInput,
} from '@/services/whitelist-service'

function getErrorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
}

export const useWhitelistStore = defineStore('whitelist', () => {
  const devices = ref<usb_control.WhitelistDevice[]>([])
  const isLoading = ref(false)
  const errorMessage = ref('')
  const pendingSerialNumbers = ref(new Set<string>())
  const pendingOwnership = new Map<string, symbol>()
  let epoch = 0
  let nextLoadRequestId = 0
  let latestLoadRequestId = 0
  let activeLoadCount = 0

  async function listWhitelist(sessionToken: string): Promise<void> {
    const loadEpoch = epoch
    const requestId = ++nextLoadRequestId
    latestLoadRequestId = requestId
    activeLoadCount += 1
    isLoading.value = true
    if (loadEpoch === epoch && requestId === latestLoadRequestId) {
      errorMessage.value = ''
    }
    try {
      const response = await requestListWhitelist(sessionToken)
      if (loadEpoch === epoch && requestId === latestLoadRequestId) {
        devices.value = response.devices.map((device) =>
          new usb_control.WhitelistDevice(device),
        )
      }
    } catch (error: unknown) {
      if (loadEpoch === epoch && requestId === latestLoadRequestId) {
        errorMessage.value = getErrorMessage(error, '白名单加载失败')
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
    serialNumber: string,
    write: () => Promise<void>,
    sessionToken: string,
  ): Promise<void> {
    if (pendingOwnership.has(serialNumber)) {
      return
    }

    const writeEpoch = epoch
    const operationToken = Symbol(serialNumber)
    pendingOwnership.set(serialNumber, operationToken)
    pendingSerialNumbers.value.add(serialNumber)
    try {
      await write()
      if (writeEpoch === epoch) {
        await listWhitelist(sessionToken)
      }
    } finally {
      if (pendingOwnership.get(serialNumber) === operationToken) {
        pendingOwnership.delete(serialNumber)
        pendingSerialNumbers.value.delete(serialNumber)
      }
    }
  }

  async function addWhitelist(
    sessionToken: string,
    input: AddWhitelistInput,
  ): Promise<void> {
    await runWrite(
      input.serialNumber,
      () => requestAddWhitelist(sessionToken, input),
      sessionToken,
    )
  }

  async function updateWhitelist(
    sessionToken: string,
    serialNumber: string,
    permission: string,
    description: string,
  ): Promise<void> {
    await runWrite(
      serialNumber,
      () => requestUpdateWhitelist(sessionToken, serialNumber, permission, description),
      sessionToken,
    )
  }

  async function removeWhitelist(sessionToken: string, serialNumber: string): Promise<void> {
    await runWrite(
      serialNumber,
      () => requestRemoveWhitelist(sessionToken, serialNumber),
      sessionToken,
    )
  }

  function clear(): void {
    epoch += 1
    activeLoadCount = 0
    devices.value = []
    isLoading.value = false
    errorMessage.value = ''
    pendingSerialNumbers.value.clear()
    pendingOwnership.clear()
  }

  return {
    devices,
    isLoading,
    errorMessage,
    pendingSerialNumbers,
    listWhitelist,
    addWhitelist,
    updateWhitelist,
    removeWhitelist,
    clear,
  }
})
