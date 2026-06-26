import { describe, expect, it } from 'vitest'
import {
  buildUsbAuditContent,
  formatUsbEventType,
} from '../../../src/renderer/utils/usb-audit-log-display'

describe('usb audit log display utils', () => {
  it('formats event type labels from the two-value server protocol', () => {
    expect(formatUsbEventType('insert_success')).toBe('USB插入成功')
    expect(formatUsbEventType('device_remove')).toBe('USB移除成功')
    expect(formatUsbEventType('legacy_value')).toBe('legacy_value')
  })

  it('assembles authorized storage insert content from detail, permission and capacity', () => {
    expect(buildUsbAuditContent({
      eventType: 'insert_success',
      deviceType: 'storage',
      interfaceType: 'mass_storage',
      detail: '授权设备',
      permission: 'readwrite',
      capacityBytes: 32_000_000_000,
    })).toBe('授权设备, 读写, 32GB')
  })

  it('assembles unauthorized storage insert content from detail only', () => {
    expect(buildUsbAuditContent({
      eventType: 'insert_success',
      deviceType: 'storage',
      interfaceType: 'mass_storage',
      detail: '未授权设备',
      permission: '',
      capacityBytes: 0,
    })).toBe('未授权设备')
  })

  it('assembles storage remove content using detail carried by server', () => {
    expect(buildUsbAuditContent({
      eventType: 'device_remove',
      deviceType: 'storage',
      interfaceType: 'mass_storage',
      detail: '授权设备',
      permission: 'readonly',
      capacityBytes: 32_000_000_000,
    })).toBe('授权设备, 只读')
  })

  it('assembles keyboard and mouse content from detail without legacy action words', () => {
    expect(buildUsbAuditContent({
      eventType: 'insert_success',
      deviceType: 'keyboard',
      interfaceType: 'hid_keyboard',
      detail: '键盘',
    })).toBe('键盘')

    expect(buildUsbAuditContent({
      eventType: 'device_remove',
      deviceType: 'mouse',
      interfaceType: 'hid_mouse',
      detail: '鼠标',
    })).toBe('鼠标')
  })

  it('keeps unsupported detail as unknown fallback only', () => {
    expect(buildUsbAuditContent({
      eventType: 'insert_success',
      deviceType: 'unsupported',
      interfaceType: 'unknown',
      detail: '未知设备',
    })).toBe('未知设备')
  })
})
