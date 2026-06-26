import { describe, expect, it } from 'vitest'
import { filterMockLogsForQuery } from '../../e2e/support/mock-device'

const usbAuditLogs = [{
  id: 1,
  deviceName: 'Kingston DataTraveler',
  deviceSn: 'USB-AUDIT-001',
  eventType: 'insert_success',
  detail: '白名单设备映射成功',
}, {
  id: 2,
  deviceName: 'Blocked USB',
  deviceSn: 'USB-AUDIT-002',
  eventType: 'insert_failed',
  detail: '未授权设备禁止接入',
}]

const malwareLogs = [{
  id: 1,
  deviceName: 'SanDisk',
  deviceSn: 'USB-MALWARE-001',
  virusName: 'EICAR-Test-File',
  filePath: '/mnt/usb/eicar.com',
  detail: '发现病毒并阻断',
}, {
  id: 2,
  deviceName: 'Clean USB',
  deviceSn: 'USB-MALWARE-002',
  virusName: '',
  filePath: '/mnt/usb/readme.txt',
  detail: '扫描通过',
}]

const operationLogs = [{
  id: 1,
  username: 'admin',
  role: 'admin',
  logCategory: 'user_management',
  actionType: 'user_create',
  target: 'new_operator',
  detail: '新建用户成功',
}, {
  id: 2,
  username: 'audit',
  role: 'auditor',
  logCategory: 'login_auth',
  actionType: 'login',
  target: 'audit',
  detail: '审计员登录成功',
}]

describe('mock-device log filter', () => {
  it('按 USB 事件类型和关键字过滤 USB 审计日志', () => {
    const result = filterMockLogsForQuery(usbAuditLogs, {
      keyword: 'Blocked',
      eventType: 'insert_failed',
    })

    expect(result.map((entry) => entry.id)).toEqual([2])
  })

  it('按关键字过滤恶意代码检测日志', () => {
    const result = filterMockLogsForQuery(malwareLogs, {
      keyword: 'EICAR',
      eventType: '',
    })

    expect(result.map((entry) => entry.id)).toEqual([1])
  })

  it('按关键字过滤操作日志', () => {
    const result = filterMockLogsForQuery(operationLogs, {
      keyword: '审计员',
      eventType: '',
    })

    expect(result.map((entry) => entry.id)).toEqual([2])
  })
})
