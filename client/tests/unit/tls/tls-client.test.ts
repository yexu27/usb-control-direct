import { describe, expect, it } from 'vitest'
import { TlsClient } from '../../../src/main/tls-client'

describe('TlsClient', () => {
  it('未连接时发送请求立即失败', async () => {
    const client = new TlsClient()

    await expect(client.send(0x0001, new Uint8Array(0))).rejects.toThrow(
      '装置已断开，请重新连接后再操作',
    )
  })
})
