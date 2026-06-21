import type { BrowserWindow, IpcMainInvokeEvent } from 'electron'

export function assertTrustedSender(
  event: IpcMainInvokeEvent,
  mainWindow: BrowserWindow | null,
): asserts mainWindow is BrowserWindow {
  if (mainWindow == null || mainWindow.isDestroyed()) {
    throw new Error('主窗口不可用')
  }

  if (
    event.sender !== mainWindow.webContents ||
    event.senderFrame !== mainWindow.webContents.mainFrame
  ) {
    throw new Error('拒绝来自非主窗口的 IPC 请求')
  }
}
