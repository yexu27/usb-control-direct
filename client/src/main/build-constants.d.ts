import type { ManagementUsbDevice } from '../shared/management-usb-device'

declare global {
  const __USB_CONTROL_CERT_FINGERPRINT__: string
  const __USB_CONTROL_E2E_USB_DEVICES__: ManagementUsbDevice[] | null
}

export {}
