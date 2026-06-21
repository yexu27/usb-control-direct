import { describe, expect, it, vi } from 'vitest'

vi.mock('electron', () => ({
  ipcMain: { handle: vi.fn() },
}))

import { parseRendererConnectionEvent } from '../../../src/main/ipc/tls-ipc'

describe('parseRendererConnectionEvent', () => {
  it('allows renderer-owned business state events', () => {
    expect(parseRendererConnectionEvent('AUTH_SUCCESS')).toBe('AUTH_SUCCESS')
    expect(parseRendererConnectionEvent('CONFIG_LOADED')).toBe('CONFIG_LOADED')
  })

  it('rejects transport lifecycle events controlled by the main process', () => {
    expect(() => parseRendererConnectionEvent('CONNECT_SUCCESS')).toThrow(
      '连接状态事件无效',
    )
    expect(() => parseRendererConnectionEvent('NETWORK_ERROR')).toThrow(
      '连接状态事件无效',
    )
  })
})
