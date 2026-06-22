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
  | 'device_insert'
  | 'device_remove'
  | 'whitelist_denied'
  | 'mapped'
  | 'map_failed'
  | 'file_access_denied'
  | 'keyboard_verify_pass'
  | 'keyboard_verify_fail'
  | 'scan_interrupted'
  | 'mouse_mapped'
  | 'mouse_map_failed'

export type OperationLogCategory =
  | ''
  | 'login_auth'
  | 'user_management'
  | 'security_config'
  | 'auth_management'
  | 'system_management'
  | 'program_upgrade'
  | 'log_management'

export const LOG_TABS: LogTab[] = [
  { value: 'usb_audit', label: 'USB审计日志' },
  { value: 'malware', label: '恶意代码检测日志' },
  { value: 'operation', label: '操作日志' },
]

export const USB_EVENT_TYPE_OPTIONS: DisplayOption<UsbEventType>[] = [
  { value: '', label: '全部事件' },
  { value: 'device_insert', label: '设备插入' },
  { value: 'device_remove', label: '设备移除' },
  { value: 'whitelist_denied', label: '禁止' },
  { value: 'mapped', label: '映射成功' },
  { value: 'map_failed', label: '映射失败' },
  { value: 'file_access_denied', label: '阻断' },
  { value: 'keyboard_verify_pass', label: '验证成功' },
  { value: 'keyboard_verify_fail', label: '验证失败' },
  { value: 'scan_interrupted', label: '扫描中断' },
  { value: 'mouse_mapped', label: '鼠标映射成功' },
  { value: 'mouse_map_failed', label: '鼠标映射失败' },
]

export const OPERATION_LOG_CATEGORY_OPTIONS: DisplayOption<OperationLogCategory>[] = [
  { value: '', label: '全部类型' },
  { value: 'login_auth', label: '登录认证' },
  { value: 'user_management', label: '用户管理' },
  { value: 'security_config', label: '安全配置变更' },
  { value: 'auth_management', label: '授权管理' },
  { value: 'system_management', label: '系统管理' },
  { value: 'program_upgrade', label: '程序升级' },
  { value: 'log_management', label: '日志管理' },
]

export function getLogColumns(logType: LogType): DataTableColumn[] {
  if (logType === 'usb_audit') {
    return [
      { prop: 'time', label: '时间', width: 170 },
      { prop: 'deviceName', label: '设备名称', minWidth: 160 },
      { prop: 'serialNumber', label: '序列号', minWidth: 180 },
      { prop: 'eventType', label: '事件类型', width: 120 },
      { prop: 'content', label: '内容', minWidth: 240 },
    ]
  }

  if (logType === 'malware') {
    return [
      { prop: 'time', label: '时间', width: 170 },
      { prop: 'deviceName', label: '设备名称', minWidth: 160 },
      { prop: 'serialNumber', label: '序列号', minWidth: 180 },
      { prop: 'content', label: '内容', minWidth: 240 },
      { prop: 'virus', label: '病毒', minWidth: 160 },
    ]
  }

  return [
    { prop: 'time', label: '时间', width: 170 },
    { prop: 'username', label: '用户', width: 140 },
    { prop: 'operationType', label: '操作日志类型', width: 160 },
    { prop: 'content', label: '内容', minWidth: 280 },
  ]
}

export function formatLogResult(result: string | number | null | undefined): string {
  const value = String(result ?? '0')

  if (value === '0') {
    return '成功'
  }

  if (value === '1') {
    return '失败'
  }

  return value
}

export function formatOperationLogCategory(value: string): string {
  return OPERATION_LOG_CATEGORY_OPTIONS.find((item) => item.value === value)?.label ?? value
}
