import { app, BrowserWindow, Menu } from 'electron'
import { createMainWindow, getMainWindow } from './window'
import { TlsClient } from './tls-client'
import { registerTlsIpc } from './ipc/tls-ipc'
import { registerDialogIpc } from './ipc/dialog-ipc'
import { registerWindowIpc } from './ipc/window-ipc'
import { registerUsbIpc } from './ipc/usb-ipc'

const tlsClient = new TlsClient()

app.whenReady().then(() => {
  Menu.setApplicationMenu(null)

  registerTlsIpc(tlsClient, getMainWindow)
  registerDialogIpc(getMainWindow)
  registerWindowIpc(getMainWindow)
  registerUsbIpc(getMainWindow)
  createMainWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createMainWindow()
    }
  })
})

app.on('window-all-closed', () => {
  tlsClient.disconnect()
  app.quit()
})
