import { app, BrowserWindow, Menu } from 'electron'
import { createMainWindow, getMainWindow } from './window'
import { TlsClient } from './tls-client'
import { registerTlsIpc } from './ipc/tls-ipc'
import { registerDialogIpc } from './ipc/dialog-ipc'
import { registerWindowIpc } from './ipc/window-ipc'
import { registerUsbIpc } from './ipc/usb-ipc'
import { listWindowsUsbStorageDevices } from './usb/windows-usb-storage'

const tlsClient = new TlsClient()

app.whenReady().then(() => {
  Menu.setApplicationMenu(null)

  registerTlsIpc(tlsClient, getMainWindow)
  registerDialogIpc(getMainWindow)
  registerWindowIpc(getMainWindow)
  const e2eUsbDevices = __USB_CONTROL_E2E_USB_DEVICES__
  const usbProvider = e2eUsbDevices == null
    ? listWindowsUsbStorageDevices
    : async () => structuredClone(e2eUsbDevices)
  registerUsbIpc(getMainWindow, usbProvider)
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
