import { describe, expect, it, vi } from 'vitest'
import { TlsClient } from '../../../src/main/tls-client'
import { HeartbeatManager } from '../../../src/main/tls/heartbeat'
import { TlsTransport } from '../../../src/main/tls/tls-transport'

class FakeTlsTransport extends TlsTransport {
  connected = false
  disconnectSpy = vi.fn(() => {
    this.connected = false
  })

  override async connect(): Promise<void> {
    this.connected = true
  }

  override disconnect(): void {
    this.disconnectSpy()
  }

  override isConnected(): boolean {
    return this.connected
  }
}

describe('TlsClient', () => {
  it('未连接时发送请求立即失败', async () => {
    const client = new TlsClient()

    await expect(client.send(0x0001, new Uint8Array(0))).rejects.toThrow(
      '装置已断开，请重新连接后再操作',
    )
  })

  it('心跳超时时关闭传输层连接', async () => {
    const transport = new FakeTlsTransport()
    const heartbeat = new HeartbeatManager()
    const client = new TlsClient(transport, heartbeat)
    await client.connect('19.19.19.16', 9600)
    client.transitionState('AUTH_SUCCESS')
    client.transitionState('LICENSE_AUTHORIZED')
    client.transitionState('CONFIG_LOADED')

    heartbeat.onTimeout?.()

    expect(client.getConnectionStatus()).toBe('DISCONNECTED')
    expect(transport.disconnectSpy).toHaveBeenCalledTimes(1)
    await expect(client.send(0x0001, new Uint8Array(0))).rejects.toThrow(
      '装置已断开，请重新连接后再操作',
    )
  })
})
