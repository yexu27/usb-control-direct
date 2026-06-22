import { beforeEach, describe, expect, it, vi } from 'vitest'
import { listManagementUsbStorageDevices } from '../../../src/renderer/services/management-usb-service'

const listStorageDevices = vi.fn()

describe('management-usb-service', () => {
  beforeEach(() => {
    listStorageDevices.mockReset()
    window.desktopApi = {
      usb: { listStorageDevices },
    } as unknown as Window['desktopApi']
  })

  it('代理 preload 的存储设备枚举接口', async () => {
    const devices = [{ serialNumber: 'SN-1' }]
    listStorageDevices.mockResolvedValue(devices)

    await expect(listManagementUsbStorageDevices()).resolves.toBe(devices)
    expect(listStorageDevices).toHaveBeenCalledOnce()
  })
})
