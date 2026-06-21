import { ipcMain, type BrowserWindow } from 'electron'
import { IpcChannels } from '../../shared/ipc-channels'

export function registerWindowIpc(getMainWindow: () => BrowserWindow | null): void {
  ipcMain.handle(IpcChannels.windowMinimize, () => {
    const win = getMainWindow()
    if (win != null && !win.isDestroyed()) {
      win.minimize()
    }
  })

  ipcMain.handle(IpcChannels.windowMaximize, () => {
    const win = getMainWindow()
    if (win != null && !win.isDestroyed()) {
      if (win.isMaximized()) {
        win.unmaximize()
      } else {
        win.maximize()
      }
    }
  })

  ipcMain.handle(IpcChannels.windowClose, () => {
    const win = getMainWindow()
    if (win != null && !win.isDestroyed()) {
      win.close()
    }
  })
}
