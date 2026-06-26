import { USB_EVENT_TYPE_OPTIONS } from '@/utils/log-display'
import type { usb_control } from '../../shared/proto/usb_control'

export type UsbAuditDeviceKind = 'storage' | 'keyboard' | 'mouse' | 'unsupported'

interface LongLike {
  toNumber?: () => number
}

export function formatUsbEventType(value: string): string {
  return USB_EVENT_TYPE_OPTIONS.find((item) => item.value === value)?.label ?? value
}

export function buildUsbAuditContent(entry: usb_control.IUsbAuditLogEntry): string {
  const deviceKind = getUsbAuditDeviceKind(entry)

  if (deviceKind === 'keyboard') {
    return entry.detail?.trim() || '键盘'
  }

  if (deviceKind === 'mouse') {
    return entry.detail?.trim() || '鼠标'
  }

  if (deviceKind !== 'storage') {
    return entry.detail?.trim() || '未知设备'
  }

  const category = getStorageDeviceCategory(entry)
  if (category === '未授权设备') {
    return category
  }

  const parts = [category]
  const permission = formatUsbAuditPermission(entry.permission)
  const capacity = formatUsbAuditCapacity(entry.capacityBytes)

  if (permission !== '') {
    parts.push(permission)
  }

  if (entry.eventType === 'insert_success' && capacity !== '') {
    parts.push(capacity)
  }

  return parts.join(', ')
}

export function getUsbAuditDeviceKind(entry: usb_control.IUsbAuditLogEntry): UsbAuditDeviceKind {
  const detail = String(entry.detail ?? '').trim()
  const deviceType = String(entry.deviceType ?? '').toLowerCase()
  const interfaceType = String(entry.interfaceType ?? '').toLowerCase()

  if (deviceType === 'storage' || interfaceType === 'mass_storage' || detail.includes('授权设备')) {
    return 'storage'
  }
  if (deviceType === 'keyboard' || interfaceType === 'hid_keyboard' || detail === '键盘') {
    return 'keyboard'
  }
  if (deviceType === 'mouse' || interfaceType === 'hid_mouse' || detail === '鼠标') {
    return 'mouse'
  }
  return 'unsupported'
}

export function formatUsbAuditPermission(permission: string | null | undefined): string {
  const value = String(permission ?? '').toLowerCase()
  if (value === 'readwrite' || value === 'read_write' || value === 'rw' || value === '2') {
    return '读写'
  }
  if (value === 'readonly' || value === 'read_only' || value === 'ro' || value === '1') {
    return '只读'
  }
  return ''
}

export function formatUsbAuditCapacity(value: number | LongLike | null | undefined): string {
  const bytes = toSafeNumber(value)
  if (bytes <= 0) {
    return ''
  }

  const gb = bytes / 1_000_000_000
  if (gb >= 1) {
    return `${formatCapacityNumber(gb)}GB`
  }

  const mb = bytes / 1_000_000
  if (mb >= 1) {
    return `${formatCapacityNumber(mb)}MB`
  }

  return `${bytes}B`
}

function getStorageDeviceCategory(entry: usb_control.IUsbAuditLogEntry): string {
  const detail = String(entry.detail ?? '').trim()
  if (detail.includes('未授权')) {
    return '未授权设备'
  }
  return '授权设备'
}

function toSafeNumber(value: number | LongLike | null | undefined): number {
  if (typeof value === 'number') {
    return Number.isFinite(value) ? value : 0
  }
  if (value != null && typeof value.toNumber === 'function') {
    const numeric = value.toNumber()
    return Number.isFinite(numeric) ? numeric : 0
  }
  return 0
}

function formatCapacityNumber(value: number): string {
  return Number.isInteger(value) ? String(value) : String(Number(value.toFixed(1)))
}
