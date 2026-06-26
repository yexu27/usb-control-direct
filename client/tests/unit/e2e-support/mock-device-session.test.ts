import { describe, expect, it } from 'vitest'
import { MockDevice } from '../../e2e/support/mock-device'

describe('MockDevice session lifecycle', () => {
  it('登出后旧 token 不再允许查询授权状态', async () => {
    const device = new MockDevice({
      authStatus: 'authorized',
      loginFailuresBeforeSuccess: 0,
      role: 'operator',
      uploadedLicenseValid: true,
    })
    await device.start()
    try {
      const login = await device.requestForTest(0x0001, {
        username: 'operator',
        password: 'operator@123',
      })
      const token = String(login.sessionToken)

      await device.requestForTest(0x0009, { sessionToken: token })

      await expect(device.requestForTest(0x0003, { sessionToken: token })).rejects.toThrow(
        '会话已失效',
      )
    } finally {
      await device.stop()
    }
  })
})
