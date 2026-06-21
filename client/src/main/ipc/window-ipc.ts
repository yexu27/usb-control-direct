import { ipcMain, type BrowserWindow } from 'electron'
import { IpcChannels } from '../../shared/ipc-channels'
import { assertTrustedSender } from './trusted-sender'

export function registerWindowIpc(getMainWindow: () => BrowserWindow | null): void {
  ipcMain.handle(IpcChannels.windowMinimize, (event) => {
    const win = getMainWindow()
    assertTrustedSender(event, win)
    win.minimize()
  })

  ipcMain.handle(IpcChannels.windowMaximize, (event) => {
    const win = getMainWindow()
    assertTrustedSender(event, win)
    if (win.isMaximized()) {
      win.unmaximize()
    } else {
      win.maximize()
    }
  })

  ipcMain.handle(IpcChannels.windowClose, (event) => {
    const win = getMainWindow()
    assertTrustedSender(event, win)
    win.close()
  })
}
