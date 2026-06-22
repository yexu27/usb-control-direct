import { describe, expect, it } from 'vitest'
import {
  calculateSha256Hex,
  parseSystemUpgradeVersion,
  parseVirusdbUpgradeVersion,
} from '../../../src/renderer/utils/upgrade-package'

describe('upgrade package utils', () => {
  it('parses system upgrade version from confirmed filename pattern', () => {
    expect(parseSystemUpgradeVersion('C:\\pkg\\usb-control-system-v1.2.3.bin')).toBe('v1.2.3')
  })

  it('rejects invalid system upgrade filename', () => {
    expect(() => parseSystemUpgradeVersion('system.bin')).toThrow('系统升级包文件名格式错误')
    expect(() => parseSystemUpgradeVersion('/tmp/usb-control-system-1.2.3.bin')).toThrow(
      '系统升级包文件名格式错误',
    )
    expect(() => parseSystemUpgradeVersion('usb-control-system-v1.2.bin')).toThrow(
      '系统升级包文件名格式错误',
    )
  })

  it('parses virusdb upgrade version from confirmed filename pattern', () => {
    expect(parseVirusdbUpgradeVersion('D:\\pkg\\usb-control-virusdb-v3.0.0.zip')).toBe('v3.0.0')
  })

  it('rejects invalid virusdb filename', () => {
    expect(() => parseVirusdbUpgradeVersion('virus.zip')).toThrow('病毒库升级包文件名格式错误')
    expect(() => parseVirusdbUpgradeVersion('usb-control-virusdb-3.0.0.zip')).toThrow(
      '病毒库升级包文件名格式错误',
    )
  })

  it('calculates lowercase sha256 hex', async () => {
    await expect(calculateSha256Hex(new Uint8Array([97, 98, 99]))).resolves.toBe(
      'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
    )
  })
})
