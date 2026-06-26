import { computed, type ComputedRef } from 'vue'
import { useConnectionStore } from '@/stores/connection'
import { showErrorDialog } from '@/utils/operation-feedback'

const DEFAULT_TITLE = '操作失败'
const DISCONNECTED_MESSAGE = '装置已断开连接，无法操作'

interface ConnectedOperationGuard {
  isBusinessActionDisabled: ComputedRef<boolean>
  canReadFromDevice: () => boolean
  canWriteToDevice: (title?: string) => Promise<boolean>
}

export function useConnectedOperationGuard(): ConnectedOperationGuard {
  const connection = useConnectionStore()
  const isBusinessActionDisabled = computed(() => !connection.isConnected)

  function canReadFromDevice(): boolean {
    return connection.isConnected
  }

  async function canWriteToDevice(title = DEFAULT_TITLE): Promise<boolean> {
    if (connection.isConnected) {
      return true
    }

    await showErrorDialog(title, DISCONNECTED_MESSAGE)
    return false
  }

  return {
    isBusinessActionDisabled,
    canReadFromDevice,
    canWriteToDevice,
  }
}
