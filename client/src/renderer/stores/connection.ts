import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { ConnectionStatus } from '../../shared/connection-state'

export const useConnectionStore = defineStore('connection', () => {
  const status = ref<ConnectionStatus>('DISCONNECTED')
  const deviceIp = ref('')
  const wasConnected = ref(false)

  const isConnected = computed(() => status.value === 'CONNECTED')
  const isDisconnected = computed(() => status.value === 'DISCONNECTED')

  let unsubscribe: (() => void) | null = null

  function setupListener(): void {
    if (window.desktopApi?.tls?.onStateChanged == null) {
      return
    }
    unsubscribe = window.desktopApi.tls.onStateChanged((newStatus: ConnectionStatus) => {
      if (newStatus === 'CONNECTED') {
        wasConnected.value = true
      }
      status.value = newStatus
    })
  }

  function teardownListener(): void {
    if (unsubscribe != null) {
      unsubscribe()
      unsubscribe = null
    }
  }

  async function connect(ip: string): Promise<void> {
    deviceIp.value = ip
    await window.desktopApi.tls.connect(ip)
  }

  function disconnect(): void {
    window.desktopApi.tls.disconnect()
  }

  function updateStatus(newStatus: ConnectionStatus): void {
    status.value = newStatus
  }

  async function reconnect(): Promise<boolean> {
    if (deviceIp.value === '') {
      return false
    }
    try {
      await window.desktopApi.tls.connect(deviceIp.value)
      return true
    } catch {
      return false
    }
  }

  return {
    status,
    deviceIp,
    wasConnected,
    isConnected,
    isDisconnected,
    connect,
    disconnect,
    reconnect,
    updateStatus,
    setupListener,
    teardownListener,
  }
})
