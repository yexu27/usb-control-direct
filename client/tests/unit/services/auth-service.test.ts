import { beforeEach, describe, expect, it, vi } from 'vitest'
import { usb_control } from '../../../src/shared/proto/usb_control'
import { login, logout } from '../../../src/renderer/services/auth-service'

const tlsSend = vi.fn()

describe('auth-service', () => {
  beforeEach(() => {
    tlsSend.mockReset()
    window.desktopApi = {
      tls: { send: tlsSend },
    } as unknown as Window['desktopApi']
  })

  it('编码登录请求并解码登录响应', async () => {
    const response = usb_control.RspLogin.fromObject({
      success: true,
      sessionToken: 'token',
      username: 'operator',
      role: 'operator',
      authStatus: 'authorized',
    })
    tlsSend.mockResolvedValue({
      msgType: 0x0002,
      payload: usb_control.RspLogin.encode(response).finish(),
    })

    const result = await login('operator', 'operator@123')

    expect(result.sessionToken).toBe('token')
    const requestPayload = tlsSend.mock.calls[0][1] as Uint8Array
    expect(usb_control.CmdLogin.decode(requestPayload)).toMatchObject({
      username: 'operator',
      password: 'operator@123',
    })
  })

  it('登出使用通用响应', async () => {
    const response = usb_control.RspCommon.fromObject({ success: true, resultCode: 0 })
    tlsSend.mockResolvedValue({
      msgType: 0xff00,
      payload: usb_control.RspCommon.encode(response).finish(),
    })

    await logout('token')

    expect(tlsSend.mock.calls[0][0]).toBe(0x0009)
  })
})
