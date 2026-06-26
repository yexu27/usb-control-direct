import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('electron', () => ({
  ipcMain: { handle: vi.fn() },
}))

import { ipcMain } from 'electron'
import { IpcChannels } from '../../../src/shared/ipc-channels'
import { parseRendererConnectionEvent, registerTlsIpc } from '../../../src/main/ipc/tls-ipc'

const handleMock = vi.mocked(ipcMain.handle)

beforeEach(() => {
  handleMock.mockClear()
})

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

describe('registerTlsIpc', () => {
  it('returns the status produced by the apply state handler', async () => {
    const tlsClient = {
      connect: vi.fn(),
      disconnect: vi.fn(),
      send: vi.fn(),
      transitionState: vi.fn().mockReturnValue('CONNECTED'),
      on: vi.fn(),
    }
    const mainFrame = {}
    const webContents = { mainFrame, send: vi.fn() }
    const mainWindow = {
      isDestroyed: vi.fn().mockReturnValue(false),
      webContents,
    }
    registerTlsIpc(
      tlsClient as unknown as Parameters<typeof registerTlsIpc>[0],
      () => mainWindow as unknown as ReturnType<Parameters<typeof registerTlsIpc>[1]>,
    )
    const applyStateHandler = handleMock.mock.calls.find(
      ([channel]) => channel === IpcChannels.tlsApplyStateEvent,
    )?.[1]

    expect(applyStateHandler?.({
      sender: webContents,
      senderFrame: mainFrame,
    }, 'CONFIG_LOADED')).toBe('CONNECTED')
    expect(tlsClient.transitionState).toHaveBeenCalledWith('CONFIG_LOADED')
  })
})
