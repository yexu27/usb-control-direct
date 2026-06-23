import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import type { ManagementUsbDevice } from '../../shared/management-usb-device'

const execFileAsync = promisify(execFile)
const CIM_DEVICE_KEYS = ['SerialNumber', 'PNPDeviceID', 'Model', 'Size'] as const

export const POWERSHELL_QUERY = [
  "$ErrorActionPreference = 'Stop'",
  "$devices = @(Get-CimInstance -ClassName Win32_DiskDrive | Where-Object { $_.InterfaceType -eq 'USB' } | Select-Object SerialNumber, PNPDeviceID, Model, Size)",
  'ConvertTo-Json -InputObject $devices -Compress',
].join('; ')

interface UsbQueryOptions {
  timeout: number
  maxBuffer: number
  windowsHide: boolean
}

export type UsbQueryRunner = (
  executable: string,
  args: string[],
  options: UsbQueryOptions,
) => Promise<{ stdout: string }>

function readString(candidate: Record<string, unknown>, key: string): string {
  const value = candidate[key]
  return typeof value === 'string' ? value.trim() : ''
}

function readCapacity(candidate: Record<string, unknown>): number {
  const value = candidate.Size
  if (typeof value === 'number') {
    return Number.isSafeInteger(value) && value >= 0 ? value : 0
  }

  if (typeof value !== 'string') {
    return 0
  }

  const normalizedValue = value.trim()
  if (!/^\d+$/.test(normalizedValue)) {
    return 0
  }

  const capacity = Number(normalizedValue)
  return Number.isSafeInteger(capacity) ? capacity : 0
}

function readUsbIdentifier(pnpDeviceId: string, name: 'VID' | 'PID'): string {
  return new RegExp(`${name}_([0-9a-f]{4})`, 'i').exec(pnpDeviceId)?.[1]?.toLowerCase() ?? ''
}

function isCimDeviceRow(input: unknown): input is Record<string, unknown> {
  return (
    typeof input === 'object' &&
    input != null &&
    !Array.isArray(input) &&
    CIM_DEVICE_KEYS.some((key) => key in input)
  )
}

function parseDevice(input: unknown): ManagementUsbDevice {
  if (!isCimDeviceRow(input)) {
    throw new Error('PowerShell USB 设备数据格式无效')
  }

  const candidate = input
  const serialNumber = readString(candidate, 'SerialNumber')
  const pnpDeviceId = readString(candidate, 'PNPDeviceID')

  return {
    serialNumber,
    vid: readUsbIdentifier(pnpDeviceId, 'VID'),
    pid: readUsbIdentifier(pnpDeviceId, 'PID'),
    deviceName: readString(candidate, 'Model'),
    capacityBytes: readCapacity(candidate),
    deviceType: 'storage',
    addable: serialNumber.length > 0,
    unavailableReason: serialNumber.length > 0 ? '' : '设备标识异常',
  }
}

function parsePowerShellOutput(stdout: string): ManagementUsbDevice[] {
  const output = stdout.trim()
  if (output.length === 0) {
    return []
  }

  const parsed: unknown = JSON.parse(output)
  if (Array.isArray(parsed)) {
    return parsed.map(parseDevice)
  }

  if (!isCimDeviceRow(parsed)) {
    throw new Error('PowerShell USB 设备数据格式无效')
  }

  const devices = [parsed]
  return devices.map(parseDevice)
}

export const execFileQuery: UsbQueryRunner = async (executable, args, options) => {
  const result = await execFileAsync(executable, args, options)
  return { stdout: result.stdout }
}

export async function listWindowsUsbStorageDevices(
  platform: NodeJS.Platform = process.platform,
  runner: UsbQueryRunner = execFileQuery,
): Promise<ManagementUsbDevice[]> {
  if (platform !== 'win32') {
    throw new Error('管理端 USB 设备枚举仅支持 Windows')
  }

  const { stdout } = await runner(
    'powershell.exe',
    ['-NoProfile', '-NonInteractive', '-Command', POWERSHELL_QUERY],
    { timeout: 10_000, maxBuffer: 1024 * 1024, windowsHide: true },
  )
  return parsePowerShellOutput(stdout)
}
