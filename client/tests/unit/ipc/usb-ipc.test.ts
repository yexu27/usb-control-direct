import { beforeEach, describe, expect, it, vi } from 'vitest'

const { handle } = vi.hoisted(() => ({ handle: vi.fn() }))

vi.mock('electron', () => ({
  ipcMain: { handle },
}))

import { registerUsbIpc } from '../../../src/main/ipc/usb-ipc'
import { IpcChannels } from '../../../src/shared/ipc-channels'

describe('registerUsbIpc', () => {
  beforeEach(() => {
    handle.mockReset()
  })

  it('注册存储设备枚举 handler 并调用 provider', async () => {
    const provider = vi.fn().mockResolvedValue([{ serialNumber: 'SN-1' }])
    const mainFrame = {}
    const webContents = { mainFrame }
    const mainWindow = { isDestroyed: () => false, webContents }
    registerUsbIpc(() => mainWindow as never, provider)
    const handler = handle.mock.calls[0][1]

    await expect(handler({ sender: webContents, senderFrame: mainFrame })).resolves.toEqual([
      { serialNumber: 'SN-1' },
    ])
    expect(handle).toHaveBeenCalledWith(IpcChannels.usbListStorageDevices, expect.any(Function))
    expect(provider).toHaveBeenCalledOnce()
  })

  it('在调用 provider 前拒绝不可信 sender', async () => {
    const provider = vi.fn()
    const mainFrame = {}
    const webContents = { mainFrame }
    const mainWindow = { isDestroyed: () => false, webContents }
    registerUsbIpc(() => mainWindow as never, provider)
    const handler = handle.mock.calls[0][1]

    await expect(handler({ sender: {}, senderFrame: {} })).rejects.toThrow(
      '拒绝来自非主窗口的 IPC 请求',
    )
    expect(provider).not.toHaveBeenCalled()
  })
})
