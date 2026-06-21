import { beforeEach, describe, expect, it, vi } from 'vitest'
import { usb_control } from '../../../src/shared/proto/usb_control'
import { ResultCode } from '../../../src/shared/result-codes'
import {
  MSG_RSP_COMMON,
  ServiceError,
  sendCommand,
} from '../../../src/renderer/services/send-command'
import { onServiceError } from '../../../src/renderer/services/service-events'

const tlsSend = vi.fn()

describe('sendCommand', () => {
  beforeEach(() => {
    tlsSend.mockReset()
    window.desktopApi = {
      tls: { send: tlsSend },
    } as unknown as Window['desktopApi']
  })

  it('返回匹配的专用响应载荷', async () => {
    const payload = new Uint8Array([0x01])
    tlsSend.mockResolvedValue({ msgType: 0x0101, payload })

    await expect(sendCommand(0x0100, new Uint8Array(0), 0x0101)).resolves.toEqual(payload)
    expect(tlsSend).toHaveBeenCalledWith(0x0100, new Uint8Array(0), 15_000)
  })

  it('查询命令收到 RspCommon 时按全局结果码失败', async () => {
    const response = usb_control.RspCommon.fromObject({
      success: false,
      resultCode: ResultCode.PERMISSION_DENIED,
      errorMessage: '',
    })
    tlsSend.mockResolvedValue({
      msgType: MSG_RSP_COMMON,
      payload: usb_control.RspCommon.encode(response).finish(),
    })

    await expect(sendCommand(0x0100, new Uint8Array(0), 0x0101)).rejects.toMatchObject({
      kind: 'business',
      resultCode: ResultCode.PERMISSION_DENIED,
      message: '当前用户无权执行此操作',
    })
  })

  it('将 UNAUTHENTICATED 分类为静默会话失效', async () => {
    const listener = vi.fn()
    const unsubscribe = onServiceError(listener)
    const response = usb_control.RspCommon.fromObject({
      success: false,
      resultCode: ResultCode.UNAUTHENTICATED,
      errorMessage: '',
    })
    tlsSend.mockResolvedValue({
      msgType: MSG_RSP_COMMON,
      payload: usb_control.RspCommon.encode(response).finish(),
    })

    await expect(sendCommand(0x0600, new Uint8Array(0), 0x0601)).rejects.toMatchObject({
      name: 'ServiceError',
      kind: 'unauthenticated',
    } satisfies Partial<ServiceError>)
    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'unauthenticated' }),
    )
    unsubscribe()
  })

  it('拒绝非预期响应消息类型', async () => {
    tlsSend.mockResolvedValue({ msgType: 0x0501, payload: new Uint8Array(0) })

    await expect(sendCommand(0x0100, new Uint8Array(0), 0x0101)).rejects.toMatchObject({
      kind: 'protocol',
    })
  })
})
