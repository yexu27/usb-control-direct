import { ipcMain, type BrowserWindow } from 'electron'
import { IpcChannels } from '../../shared/ipc-channels'
import type { TlsClient } from '../tls-client'

const DEFAULT_PORT = 9600

export function registerTlsIpc(
  tlsClient: TlsClient,
  getMainWindow: () => BrowserWindow | null,
): void {
  ipcMain.handle(IpcChannels.tlsConnect, async (_event, ip: string) => {
    await tlsClient.connect(ip, DEFAULT_PORT)
  })

  ipcMain.handle(IpcChannels.tlsDisconnect, () => {
    tlsClient.disconnect()
  })

  ipcMain.handle(
    IpcChannels.tlsSend,
    async (_event, msgType: number, payload: Uint8Array, timeout?: number) => {
      return tlsClient.send(msgType, payload, timeout)
    },
  )

  tlsClient.on('state-change', (_from, to, _event) => {
    const mainWindow = getMainWindow()
    if (mainWindow != null && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send(IpcChannels.connectionStateChanged, to)
    }
  })
}
