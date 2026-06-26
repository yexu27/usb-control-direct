import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { ConnectionEvent, ConnectionStatus } from '../../shared/connection-state'

export const useConnectionStore = defineStore('connection', () => {
  const status = ref<ConnectionStatus>('DISCONNECTED')
  const deviceIp = ref('')
  const wasConnected = ref(false)
  let transportConnected = false

  const isConnected = computed(() => status.value === 'CONNECTED')
  const isDisconnected = computed(() => status.value === 'DISCONNECTED')

  let unsubscribe: (() => void) | null = null

  function setStatus(newStatus: ConnectionStatus): void {
    if (newStatus === 'CONNECTED') {
      wasConnected.value = true
    }
    if (newStatus === 'DISCONNECTED') {
      transportConnected = false
    }
    status.value = newStatus
  }

  function setupListener(): void {
    if (unsubscribe != null || window.desktopApi?.tls?.onStateChanged == null) {
      return
    }
    unsubscribe = window.desktopApi.tls.onStateChanged((newStatus: ConnectionStatus) => {
      setStatus(newStatus)
    })
  }

  function teardownListener(): void {
    if (unsubscribe != null) {
      unsubscribe()
      unsubscribe = null
    }
  }

  async function connect(ip: string): Promise<void> {
    if (transportConnected && deviceIp.value === ip && status.value === 'AUTHENTICATING') {
      return
    }

    if (transportConnected || !isDisconnected.value) {
      await disconnect()
    }

    deviceIp.value = ip
    await window.desktopApi.tls.connect(ip)
    transportConnected = true
    setStatus('AUTHENTICATING')
  }

  async function disconnect(clearDeviceIp = false): Promise<void> {
    try {
      await window.desktopApi.tls.disconnect()
    } finally {
      transportConnected = false
      setStatus('DISCONNECTED')
      if (clearDeviceIp) {
        deviceIp.value = ''
      }
    }
  }

  async function applyStateEvent(event: ConnectionEvent): Promise<void> {
    const nextStatus = await window.desktopApi.tls.applyStateEvent(event)
    setStatus(nextStatus)
  }

  function updateStatus(newStatus: ConnectionStatus): void {
    setStatus(newStatus)
  }

  async function reconnect(): Promise<boolean> {
    if (deviceIp.value === '') {
      return false
    }
    try {
      await connect(deviceIp.value)
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
    applyStateEvent,
    updateStatus,
    setupListener,
    teardownListener,
  }
})
