import { describe, expect, it, vi } from 'vitest'
import {
  POWERSHELL_QUERY,
  listWindowsUsbStorageDevices,
  type UsbQueryRunner,
} from '../../../src/main/usb/windows-usb-storage'

function createRunner(stdout: string): UsbQueryRunner {
  return vi.fn().mockResolvedValue({ stdout })
}

describe('listWindowsUsbStorageDevices', () => {
  it('解析 PowerShell 单对象并规范化设备字段', async () => {
    const runner = createRunner(
      JSON.stringify({
        SerialNumber: '  SN-001  ',
        PNPDeviceID: 'USBSTOR\\DISK&VEN_TEST&VID_1234&PID_ABCD',
        Model: 'USB Flash Disk',
        Size: '64000000000',
      }),
    )

    await expect(listWindowsUsbStorageDevices('win32', runner)).resolves.toEqual([
      {
        serialNumber: 'SN-001',
        vid: '1234',
        pid: 'abcd',
        deviceName: 'USB Flash Disk',
        capacityBytes: 64_000_000_000,
        deviceType: 'storage',
        addable: true,
        unavailableReason: '',
      },
    ])
  })

  it('解析数组并在 PNP 标识缺失时保留空 VID/PID', async () => {
    const runner = createRunner(
      JSON.stringify([
        { SerialNumber: 'SN-1', PNPDeviceID: '', Model: 'Disk 1', Size: 1024 },
        { SerialNumber: 'SN-2', PNPDeviceID: null, Model: 'Disk 2', Size: 2048 },
      ]),
    )

    const devices = await listWindowsUsbStorageDevices('win32', runner)

    expect(devices).toHaveLength(2)
    expect(devices.map(({ vid, pid }) => ({ vid, pid }))).toEqual([
      { vid: '', pid: '' },
      { vid: '', pid: '' },
    ])
  })

  it('将空输出解析为空设备列表', async () => {
    await expect(listWindowsUsbStorageDevices('win32', createRunner('  '))).resolves.toEqual([])
  })

  it('将空序列号设备标记为不可添加', async () => {
    const runner = createRunner(
      JSON.stringify([{ SerialNumber: ' ', PNPDeviceID: 'VID_1234&PID_ABCD', Model: 'Disk', Size: 1 }]),
    )

    await expect(listWindowsUsbStorageDevices('win32', runner)).resolves.toMatchObject([
      { serialNumber: '', addable: false, unavailableReason: '设备标识异常' },
    ])
  })

  it.each([
    [true],
    [1.5],
    [-1],
    [Number.MAX_SAFE_INTEGER + 1],
    ['1.5'],
    ['-1'],
    ['NaN'],
    ['Infinity'],
    [String(Number.MAX_SAFE_INTEGER + 1)],
    ['12MB'],
  ])('将非法容量 %j 解析为 0', async (size) => {
    const runner = createRunner(
      JSON.stringify([{ SerialNumber: 'SN-1', Model: 'Disk', Size: size }]),
    )

    await expect(listWindowsUsbStorageDevices('win32', runner)).resolves.toMatchObject([
      { capacityBytes: 0 },
    ])
  })

  it('将 JSON 数值溢出的 Infinity 容量解析为 0', async () => {
    const runner = createRunner('[{"SerialNumber":"SN-1","Size":1e309}]')

    await expect(listWindowsUsbStorageDevices('win32', runner)).resolves.toMatchObject([
      { capacityBytes: 0 },
    ])
  })

  it('接受安全整数与纯十进制数字字符串容量', async () => {
    const runner = createRunner(
      JSON.stringify([
        { SerialNumber: 'SN-1', Size: Number.MAX_SAFE_INTEGER },
        { SerialNumber: 'SN-2', Size: ' 001024 ' },
      ]),
    )

    await expect(listWindowsUsbStorageDevices('win32', runner)).resolves.toMatchObject([
      { capacityBytes: Number.MAX_SAFE_INTEGER },
      { capacityBytes: 1024 },
    ])
  })

  it('拒绝畸形 JSON', async () => {
    await expect(listWindowsUsbStorageDevices('win32', createRunner('{bad json'))).rejects.toThrow()
  })

  it.each([null, 'device', 1, true, {}])('拒绝无效顶层 JSON 形状 %j', async (value) => {
    await expect(
      listWindowsUsbStorageDevices('win32', createRunner(JSON.stringify(value))),
    ).rejects.toThrow('PowerShell USB 设备数据格式无效')
  })

  it('拒绝包含非对象元素的设备数组', async () => {
    await expect(
      listWindowsUsbStorageDevices('win32', createRunner(JSON.stringify([{ SerialNumber: 'SN-1' }, 1]))),
    ).rejects.toThrow('PowerShell USB 设备数据格式无效')
  })

  it('拒绝包含空对象的设备数组', async () => {
    await expect(
      listWindowsUsbStorageDevices('win32', createRunner(JSON.stringify([{}]))),
    ).rejects.toThrow('PowerShell USB 设备数据格式无效')
  })

  it('拒绝包含未知键对象的设备数组', async () => {
    await expect(
      listWindowsUsbStorageDevices('win32', createRunner(JSON.stringify([{ foo: 'bar' }]))),
    ).rejects.toThrow('PowerShell USB 设备数据格式无效')
  })

  it('透传 runner 超时错误', async () => {
    const timeoutError = new Error('query timed out')
    const runner: UsbQueryRunner = vi.fn().mockRejectedValue(timeoutError)

    await expect(listWindowsUsbStorageDevices('win32', runner)).rejects.toBe(timeoutError)
  })

  it('拒绝在非 Windows 平台枚举且不调用 runner', async () => {
    const runner = createRunner('[]')

    await expect(listWindowsUsbStorageDevices('darwin', runner)).rejects.toThrow(
      '管理端 USB 设备枚举仅支持 Windows',
    )
    expect(runner).not.toHaveBeenCalled()
  })

  it('使用固定 PowerShell 命令与受限执行选项', async () => {
    const runner = createRunner('[]')

    await listWindowsUsbStorageDevices('win32', runner)

    expect(runner).toHaveBeenCalledWith(
      'powershell.exe',
      ['-NoProfile', '-NonInteractive', '-Command', POWERSHELL_QUERY],
      { timeout: 10_000, maxBuffer: 1024 * 1024, windowsHide: true },
    )
  })
})
