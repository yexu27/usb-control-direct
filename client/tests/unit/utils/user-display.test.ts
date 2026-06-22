import { describe, expect, it } from 'vitest'
import {
  USER_ROLE_OPTIONS,
  formatCreatedAt,
  formatUserRole,
  formatUserStatus,
} from '../../../src/renderer/utils/user-display'

describe('user display utils', () => {
  it('defines PRD role options', () => {
    expect(USER_ROLE_OPTIONS).toEqual([
      { value: 'admin', label: '系统管理员' },
      { value: 'operator', label: '操作员' },
      { value: 'auditor', label: '审计员' },
    ])
  })

  it('formats role and status labels', () => {
    expect(formatUserRole('admin')).toBe('系统管理员')
    expect(formatUserRole('operator')).toBe('操作员')
    expect(formatUserRole('auditor')).toBe('审计员')
    expect(formatUserRole('custom')).toBe('custom')
    expect(formatUserStatus('locked')).toBe('锁定')
    expect(formatUserStatus('active')).toBe('正常')
  })

  it('formats built-in and unix second created time', () => {
    const localDate = new Date(2026, 0, 1, 8, 30, 10)
    const unixSeconds = Math.floor(localDate.getTime() / 1000)

    expect(formatCreatedAt(0)).toBe('内置')
    expect(formatCreatedAt('0')).toBe('内置')
    expect(formatCreatedAt(unixSeconds)).toBe('2026-01-01')
  })
})
