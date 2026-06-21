import { beforeEach, describe, expect, it, vi } from 'vitest'
import { usb_control } from '../../../src/shared/proto/usb_control'
import {
  getMachineCode,
  login,
  logout,
  uploadLicense,
} from '../../../src/renderer/services/auth-service'

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

  it('获取机器码并解码二维码 PNG 字节', async () => {
    const response = usb_control.RspMachineCode.fromObject({
      machineCode: 'USB-DEVICE-RK3568-001',
      qrcodePng: new Uint8Array([0x89, 0x50, 0x4e, 0x47]),
    })
    tlsSend.mockResolvedValue({
      msgType: 0x0006,
      payload: usb_control.RspMachineCode.encode(response).finish(),
    })

    const result = await getMachineCode('temporary-token')

    expect(result.machineCode).toBe('USB-DEVICE-RK3568-001')
    expect(result.qrcodePng).toEqual(new Uint8Array([0x89, 0x50, 0x4e, 0x47]))
    expect(tlsSend.mock.calls[0][0]).toBe(0x0005)
    const requestPayload = tlsSend.mock.calls[0][1] as Uint8Array
    expect(usb_control.CmdGetMachineCode.decode(requestPayload).sessionToken).toBe(
      'temporary-token',
    )
  })

  it('上传授权文件使用文件传输超时并解码响应', async () => {
    const response = usb_control.RspUploadLicense.fromObject({
      success: true,
      expireTime: 1_893_455_999,
      resultCode: 0,
    })
    tlsSend.mockResolvedValue({
      msgType: 0x0008,
      payload: usb_control.RspUploadLicense.encode(response).finish(),
    })
    const licenseData = new Uint8Array([1, 2, 3])

    const result = await uploadLicense('temporary-token', licenseData)

    expect(result.expireTime).toBe(1_893_455_999)
    expect(tlsSend.mock.calls[0][0]).toBe(0x0007)
    expect(tlsSend.mock.calls[0][2]).toBe(300_000)
    const requestPayload = tlsSend.mock.calls[0][1] as Uint8Array
    expect(usb_control.CmdUploadLicense.decode(requestPayload)).toMatchObject({
      sessionToken: 'temporary-token',
      licenseData,
    })
  })
})
