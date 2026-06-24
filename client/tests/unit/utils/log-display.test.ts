import { describe, expect, it } from 'vitest'
import {
  LOG_TABS,
  formatLogResult,
  formatOperationLogCategory,
  getLogColumns,
} from '../../../src/renderer/utils/log-display'

describe('log display utils', () => {
  it('defines exactly three PRD log tabs', () => {
    expect(LOG_TABS.map((tab) => tab.label)).toEqual(['USB审计日志', '恶意代码检测日志', '操作日志'])
    expect(LOG_TABS.map((tab) => tab.value)).toEqual(['usb_audit', 'malware', 'operation'])
  })

  it('returns PRD columns by log type', () => {
    expect(getLogColumns('usb_audit').map((column) => column.label)).toEqual([
      '时间',
      '设备名称',
      '序列号',
      '插拔类型',
      '内容',
    ])
    expect(getLogColumns('malware').map((column) => column.label)).toEqual([
      '时间',
      '设备名称',
      '序列号',
      '内容',
      '病毒',
    ])
    expect(getLogColumns('operation').map((column) => column.label)).toEqual([
      '时间',
      '用户',
      '操作日志类型',
      '内容',
    ])
  })

  it('formats protocol result strings and operation log category', () => {
    expect(formatLogResult('0')).toBe('成功')
    expect(formatLogResult('1')).toBe('失败')
    expect(formatLogResult(0)).toBe('成功')
    expect(formatOperationLogCategory('user_management')).toBe('用户管理')
  })
})
