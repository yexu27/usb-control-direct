import type { usb_control } from '../../shared/proto/usb_control'

interface OperationLogDisplayEntry extends Omit<usb_control.IOperationLogEntry, 'result'> {
  result?: string | number | null
}

const ACTION_LABELS: Record<string, string> = {
  login: '用户登录',
  logout: '用户登出',
  login_failed: '用户登录失败',
  password_change: '修改密码',
  password_reset: '重置密码',
  user_create: '新建用户',
  user_delete: '删除用户',
  whitelist_add: '添加白名单设备',
  whitelist_remove: '删除白名单设备',
  whitelist_update: '修改白名单权限',
  file_policy_update: '修改文件访问控制策略',
  blacklist_add: '添加文件类型黑名单',
  blacklist_remove: '删除文件类型黑名单',
  policy_import: '导入策略',
  policy_export: '导出策略',
  system_upgrade: '系统升级',
  virusdb_upgrade: '病毒库升级',
  auth_upload: '上传授权文件',
  machine_code_download: '下载机器码',
  device_desc_update: '修改设备描述',
  log_clean: '清理日志',
  log_export: '导出日志',
}

const LOG_TYPE_LABELS: Record<string, string> = {
  login_auth: '登录认证',
  user_management: '用户管理',
  security_config: '安全配置变更',
  auth_management: '授权管理',
  system_management: '系统管理',
  program_upgrade: '程序升级',
  log_management: '日志管理',
}

interface OperationChange {
  beforeText: string
  afterText: string
}

type OperationValue = Record<string, unknown>

export function formatOperationLogType(value: string | null | undefined): string {
  const logType = text(value)
  return LOG_TYPE_LABELS[logType] ?? logType
}

export function buildOperationLogContent(entry: OperationLogDisplayEntry): string {
  const actionType = text(entry.actionType)
  const actionText = ACTION_LABELS[actionType] ?? actionType
  const resultText = formatOperationResult(entry.result)
  const failReason = text(entry.failReason)

  if (resultText === '失败') {
    return appendFailure(formatOperationSubject(actionType, actionText, entry), failReason)
  }

  if (actionType === 'system_upgrade' || actionType === 'virusdb_upgrade') {
    return `${actionText}${formatVersionTarget(entry)}，${resultText}`
  }

  const change = formatOperationChange(entry.beforeValue, entry.afterValue)
  if (change != null) {
    return `${formatOperationSubject(actionType, actionText, entry)}，${change.beforeText}→${change.afterText}，${resultText}`
  }

  return `${formatOperationSubject(actionType, actionText, entry)}，${resultText}`
}

function formatOperationSubject(
  actionType: string,
  actionText: string,
  entry: OperationLogDisplayEntry,
): string {
  const target = text(entry.target)

  if (actionType === 'login' || actionType === 'logout' || actionType === 'login_failed') {
    return target === '' ? actionText : `${actionText}，用户名：${target}`
  }

  if (actionType === 'machine_code_download') {
    return actionText
  }

  if (target === '') {
    return actionText
  }

  return `${actionText} ${target}`
}

function formatVersionTarget(entry: OperationLogDisplayEntry): string {
  const version = text(entry.relatedVersion) || text(entry.target)
  return version === '' ? '' : `，版本升级至${version}`
}

function appendFailure(subject: string, failReason: string): string {
  if (failReason === '') {
    return `${subject}，失败`
  }
  return `${subject}，失败：${failReason}`
}

function formatOperationResult(result: string | number | null | undefined): '成功' | '失败' {
  return String(result ?? '0') === '0' ? '成功' : '失败'
}

function formatOperationChange(
  beforeValue: string | null | undefined,
  afterValue: string | null | undefined,
): OperationChange | null {
  const before = parseOperationValue(beforeValue)
  const after = parseOperationValue(afterValue)
  if (before == null || after == null) {
    return null
  }

  if ('permission' in before || 'permission' in after) {
    const beforeText = formatPermission(before.permission)
    const afterText = formatPermission(after.permission)
    return beforeText !== '' && afterText !== '' ? { beforeText, afterText } : null
  }

  if ('enabled' in before || 'enabled' in after) {
    const beforeText = formatEnabled(before.enabled)
    const afterText = formatEnabled(after.enabled)
    return beforeText !== '' && afterText !== '' ? { beforeText, afterText } : null
  }

  return null
}

function parseOperationValue(value: string | null | undefined): OperationValue | null {
  const raw = text(value)
  if (raw === '') {
    return null
  }

  try {
    const parsed: unknown = JSON.parse(raw)
    if (parsed != null && typeof parsed === 'object' && !Array.isArray(parsed)) {
      return parsed as OperationValue
    }
  } catch {
    return null
  }

  return null
}

function formatPermission(value: unknown): string {
  const normalized = String(value ?? '').toLowerCase()
  if (normalized === '0' || normalized === '1' || normalized === 'readonly' || normalized === 'read_only') {
    return '只读'
  }
  if (normalized === '2' || normalized === 'readwrite' || normalized === 'read_write') {
    return '读写'
  }
  return ''
}

function formatEnabled(value: unknown): string {
  if (value === true || value === 1 || value === '1' || value === 'true') {
    return '开启'
  }
  if (value === false || value === 0 || value === '0' || value === 'false') {
    return '关闭'
  }
  return ''
}

function text(value: string | null | undefined): string {
  return value?.trim() ?? ''
}
