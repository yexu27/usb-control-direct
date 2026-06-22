import { ipcMain, type BrowserWindow } from 'electron'
import type { ManagementUsbDevice } from '../../shared/management-usb-device'
import { IpcChannels } from '../../shared/ipc-channels'
import { listWindowsUsbStorageDevices } from '../usb/windows-usb-storage'
import { assertTrustedSender } from './trusted-sender'

type UsbStorageProvider = () => Promise<ManagementUsbDevice[]>

export function registerUsbIpc(
  getMainWindow: () => BrowserWindow | null,
  provider: UsbStorageProvider = listWindowsUsbStorageDevices,
): void {
  ipcMain.handle(IpcChannels.usbListStorageDevices, async (event) => {
    assertTrustedSender(event, getMainWindow())
    return provider()
  })
}
