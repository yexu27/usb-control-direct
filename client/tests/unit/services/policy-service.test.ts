import { beforeEach, describe, expect, it, vi } from 'vitest'
import { usb_control } from '../../../src/shared/proto/usb_control'
import { exportPolicy, importPolicy } from '../../../src/renderer/services/policy-service'

const tlsSend = vi.fn()

describe('policy-service', () => {
  beforeEach(() => {
    tlsSend.mockReset()
    window.desktopApi = { tls: { send: tlsSend } } as unknown as Window['desktopApi']
  })

  it('导出策略使用 300 秒文件传输超时', async () => {
    const response = usb_control.RspExportPolicy.fromObject({
      success: true,
      resultCode: 0,
      policyData: new Uint8Array([1, 2]),
    })
    tlsSend.mockResolvedValue({
      msgType: 0x0301,
      payload: usb_control.RspExportPolicy.encode(response).finish(),
    })

    await exportPolicy('session-token')

    expect(tlsSend).toHaveBeenCalledWith(
      0x0300,
      expect.any(Uint8Array),
      300_000,
    )
  })

  it('导入策略使用 300 秒文件传输超时且不自动重试', async () => {
    const response = usb_control.RspCommon.fromObject({ success: true, resultCode: 0 })
    tlsSend.mockResolvedValue({
      msgType: 0xff00,
      payload: usb_control.RspCommon.encode(response).finish(),
    })

    await importPolicy('session-token', new Uint8Array([3, 4]))

    expect(tlsSend).toHaveBeenCalledTimes(1)
    expect(tlsSend).toHaveBeenCalledWith(
      0x0302,
      expect.any(Uint8Array),
      300_000,
    )
  })
})
