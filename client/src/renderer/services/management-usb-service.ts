import type { ManagementUsbDevice } from '../../shared/management-usb-device'

export function listManagementUsbStorageDevices(): Promise<ManagementUsbDevice[]> {
  return window.desktopApi.usb.listStorageDevices()
}
