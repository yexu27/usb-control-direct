import { app, BrowserWindow } from 'electron'
import { createMainWindow, getMainWindow } from './window'
import { TlsClient } from './tls-client'
import { registerTlsIpc } from './ipc/tls-ipc'

const tlsClient = new TlsClient()

app.whenReady().then(() => {
  registerTlsIpc(tlsClient, getMainWindow)
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
