import { ResultCode } from '../../shared/result-codes'

const RESULT_CODE_MESSAGES: Record<number, string | null> = {
  [ResultCode.UNAUTHENTICATED]: null,
  [ResultCode.UNAUTHORIZED]: '装置未授权，请先完成授权',
  [ResultCode.PERMISSION_DENIED]: '当前用户无权执行此操作',
  [ResultCode.ACCOUNT_LOCKED]: '用户已被锁定，请5分钟后重试',
  [ResultCode.VALIDATION_FAILED]: null,
  [ResultCode.DEVICE_BUSY]: '装置正在处理其他操作，请稍后重试',
  [ResultCode.INTERNAL_ERROR]: '装置内部错误，请联系管理员',
  [ResultCode.USER_OR_PASSWORD_ERROR]: '用户名或密码错误',
  [ResultCode.DEVICE_UNAUTHORIZED]: null,
  [ResultCode.LICENSE_FORMAT_ERROR]: '授权文件格式错误，请检查文件',
  [ResultCode.LICENSE_VERIFY_FAILED]: '授权文件校验失败，请确认文件与本装置匹配',
  [ResultCode.LICENSE_EXPIRED]: '授权文件已过有效期，请重新获取授权文件',
  [ResultCode.SERIAL_NUMBER_EMPTY]: '序列号不能为空',
  [ResultCode.ALREADY_EXISTS]: '该设备已在白名单中',
  [ResultCode.NOT_FOUND]: '目标设备不存在',
  [ResultCode.DEVICE_NOT_STORAGE]: '仅支持添加大容量存储设备',
  [ResultCode.DEVICE_SPOOF_SUSPECTED]: '设备描述符异常，疑似伪装设备，禁止添加',
  [ResultCode.DEVICE_UNSUPPORTED]: '不支持的USB设备类型，无法添加',
  [ResultCode.POLICY_KEY_INVALID]: '策略开关标识无效',
  [ResultCode.EXTENSION_FORMAT_ERROR]: '文件后缀格式错误',
  [ResultCode.EXTENSION_EXISTS]: '该文件后缀已在黑名单中',
  [ResultCode.EXTENSION_NOT_FOUND]: '该文件后缀不在黑名单中',
  [ResultCode.DEFAULT_EXTENSION_NO_DELETE]: '内置默认后缀不可删除',
  [ResultCode.POLICY_EXPORT_FAILED]: '策略导出失败，请重试',
  [ResultCode.FORMAT_ERROR_POLICY]: '策略文件格式错误',
  [ResultCode.POLICY_SIGNATURE_ERROR]: '策略文件签名校验失败',
  [ResultCode.POLICY_DIGEST_ERROR]: '策略文件完整性校验失败',
  [ResultCode.POLICY_DECRYPT_ERROR]: '策略文件解密失败',
  [ResultCode.VERSION_INCOMPATIBLE]: '策略文件版本不兼容，请检查文件',
  [ResultCode.POLICY_IMPORT_FAILED]: '策略导入失败，原策略未变更',
  [ResultCode.LOG_RETENTION_VIOLATION]: '半年内的日志不可清理',
  [ResultCode.LOG_TYPE_INVALID]: null,
  [ResultCode.LOG_QUERY_FAILED]: '日志查询失败，请重试',
  [ResultCode.LOG_EXPORT_FAILED]: '日志导出失败，请重试',
  [ResultCode.VERSION_TOO_LOW]: '升级包版本低于当前版本，不允许降版本',
  [ResultCode.FORMAT_ERROR_UPGRADE]: '升级包格式错误',
  [ResultCode.UPGRADE_CHECKSUM_ERROR]: '升级包完整性校验失败',
  [ResultCode.UPGRADE_APPLY_FAILED]: '系统升级失败，已回滚至原版本',
  [ResultCode.VERSION_NUMBER_FORBIDDEN]: '病毒库版本号不合法',
  [ResultCode.VIRUSDB_INTEGRITY_ERROR]: '病毒库文件完整性校验失败',
  [ResultCode.CLAMD_RELOAD_FAILED]: '病毒库更新成功但加载失败，已回滚至旧版本',
  [ResultCode.VIRUSDB_APPLY_FAILED]: '病毒库升级失败，原病毒库未变更',
  [ResultCode.DEVICE_DESC_FORMAT_ERROR]: '设备描述仅允许字母、数字、下划线，最多32位',
  [ResultCode.USERNAME_EXISTS]: '用户名已存在',
  [ResultCode.USERNAME_DELETED_REUSE]: '该用户名已被使用过，不可重复创建',
  [ResultCode.USER_NOT_FOUND]: '目标用户不存在',
  [ResultCode.PASSWORD_COMPLEXITY_ERROR]: null,
  [ResultCode.PASSWORD_CONFIRM_MISMATCH]: '两次输入的密码不一致',
  [ResultCode.OLD_PASSWORD_ERROR]: '旧密码错误',
  [ResultCode.BUILTIN_USER_NO_DELETE]: '内置用户不可删除',
  [ResultCode.SELF_DELETE_FORBIDDEN]: '不允许删除当前登录用户',
}

export function getResultMessage(resultCode: number, errorMessage: string): string | null {
  if (resultCode === ResultCode.SUCCESS) {
    return null
  }

  if (resultCode === ResultCode.UNAUTHENTICATED) {
    return null
  }

  if (resultCode === ResultCode.DEVICE_UNAUTHORIZED) {
    return null
  }

  if (resultCode in RESULT_CODE_MESSAGES) {
    const mapped = RESULT_CODE_MESSAGES[resultCode]
    if (mapped === null) {
      return errorMessage || '操作失败'
    }
    return mapped
  }

  return errorMessage || '未知错误'
}
