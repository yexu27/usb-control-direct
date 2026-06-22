export interface ManagementUsbDevice {
  serialNumber: string
  vid: string
  pid: string
  deviceName: string
  capacityBytes: number
  deviceType: 'storage'
  addable: boolean
  unavailableReason: string
}
