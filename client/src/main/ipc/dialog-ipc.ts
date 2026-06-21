import { ipcMain, dialog, type BrowserWindow } from 'electron'
import { IpcChannels } from '../../shared/ipc-channels'

export function registerDialogIpc(getMainWindow: () => BrowserWindow | null): void {
  ipcMain.handle(IpcChannels.dialogOpenFile, async (_event, options: unknown) => {
    const mainWindow = getMainWindow()
    if (mainWindow == null) {
      return { canceled: true, filePaths: [] }
    }
    const parsed = options as { title?: string; filters?: Array<{ name: string; extensions: string[] }> }
    return dialog.showOpenDialog(mainWindow, {
      title: parsed?.title,
      filters: parsed?.filters,
      properties: ['openFile'],
    })
  })

  ipcMain.handle(IpcChannels.dialogSaveFile, async (_event, options: unknown) => {
    const mainWindow = getMainWindow()
    if (mainWindow == null) {
      return { canceled: true }
    }
    const parsed = options as { title?: string; defaultPath?: string; filters?: Array<{ name: string; extensions: string[] }> }
    return dialog.showSaveDialog(mainWindow, {
      title: parsed?.title,
      defaultPath: parsed?.defaultPath,
      filters: parsed?.filters,
    })
  })
}
