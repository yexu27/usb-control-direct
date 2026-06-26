import { describe, expect, it } from 'vitest'
import { buildOperationLogContent } from '../../../src/renderer/utils/operation-log-display'

describe('operation log display utils', () => {
  it('按动作类型、目标和结果组装登录成功内容', () => {
    expect(buildOperationLogContent({
      actionType: 'login',
      target: 'admin',
      result: '0',
    })).toBe('用户登录，用户名：admin，成功')
  })

  it('失败时展示失败原因，不展示原始 result', () => {
    expect(buildOperationLogContent({
      actionType: 'login_failed',
      target: 'admin',
      result: '1',
      failReason: '密码错误',
    })).toBe('用户登录失败，用户名：admin，失败：密码错误')
  })

  it('按 beforeValue 和 afterValue 组装白名单权限变更内容', () => {
    expect(buildOperationLogContent({
      actionType: 'whitelist_update',
      target: 'ABC123',
      beforeValue: '{"permission":"readonly"}',
      afterValue: '{"permission":"readwrite"}',
      result: '0',
    })).toBe('修改白名单权限 ABC123，只读→读写，成功')
  })

  it('兼容服务端传数字权限值', () => {
    expect(buildOperationLogContent({
      actionType: 'whitelist_update',
      target: 'ABC123',
      beforeValue: '{"permission":1}',
      afterValue: '{"permission":2}',
      result: 0,
    })).toBe('修改白名单权限 ABC123，只读→读写，成功')
  })

  it('按 enabled 变更组装文件访问控制策略内容', () => {
    expect(buildOperationLogContent({
      actionType: 'file_policy_update',
      target: 'exec_control',
      beforeValue: '{"enabled":false}',
      afterValue: '{"enabled":true}',
      result: '0',
    })).toBe('修改文件访问控制策略 exec_control，关闭→开启，成功')
  })

  it('系统和病毒库升级优先使用关联版本或目标作为版本号', () => {
    expect(buildOperationLogContent({
      actionType: 'system_upgrade',
      target: 'v2.1.0',
      result: '0',
    })).toBe('系统升级，版本升级至v2.1.0，成功')
    expect(buildOperationLogContent({
      actionType: 'virusdb_upgrade',
      relatedVersion: 'V3.0.0.3',
      result: '0',
    })).toBe('病毒库升级，版本升级至V3.0.0.3，成功')
  })

  it('未知动作保留原始 actionType 但仍使用中文结果', () => {
    expect(buildOperationLogContent({
      actionType: 'custom_action',
      target: 'target-1',
      result: '7',
      failReason: '',
    })).toBe('custom_action target-1，失败')
  })
})
