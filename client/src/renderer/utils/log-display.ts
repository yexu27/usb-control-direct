import type { DataTableColumn } from '@/components/data-table'

export type LogType = 'usb_audit' | 'malware' | 'operation'

export interface LogTab {
  value: LogType
  label: string
}

export interface DisplayOption<TValue extends string = string> {
  value: TValue
  label: string
}

export type UsbEventType =
  | ''
  | 'insert_success'
  | 'insert_failed'
  | 'device_remove'

export const LOG_TABS: LogTab[] = [
  { value: 'usb_audit', label: 'USB审计日志' },
  { value: 'malware', label: '恶意代码检测日志' },
  { value: 'operation', label: '操作日志' },
]

export const USB_EVENT_TYPE_OPTIONS: DisplayOption<UsbEventType>[] = [
  { value: '', label: '全部事件' },
  { value: 'insert_success', label: 'USB插入成功' },
  { value: 'insert_failed', label: 'USB插入失败' },
  { value: 'device_remove', label: 'USB拔出' },
]

export function getLogColumns(logType: LogType): DataTableColumn[] {
  if (logType === 'usb_audit') {
    return [
      { prop: 'time', label: '时间', width: 190 },
      { prop: 'deviceName', label: '设备名称', minWidth: 160 },
      { prop: 'serialNumber', label: '序列号', minWidth: 210, slot: 'serialNumber' },
      { prop: 'eventType', label: '插拔类型', width: 140, slot: 'eventType' },
      { prop: 'content', label: '内容', minWidth: 260 },
    ]
  }

  if (logType === 'malware') {
    return [
      { prop: 'time', label: '时间', width: 190 },
      { prop: 'deviceName', label: '设备名称', minWidth: 150 },
      { prop: 'serialNumber', label: '序列号', minWidth: 210, slot: 'serialNumber' },
      { prop: 'content', label: '内容', minWidth: 360 },
      { prop: 'virus', label: '病毒', minWidth: 160 },
    ]
  }

  return [
    { prop: 'time', label: '时间', width: 190 },
    { prop: 'username', label: '用户', width: 160 },
    { prop: 'logCategory', label: '操作日志类型', width: 160 },
    { prop: 'content', label: '内容', minWidth: 420 },
  ]
}
