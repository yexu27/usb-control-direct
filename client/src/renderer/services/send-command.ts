import { usb_control } from '../../shared/proto/usb_control'
import { ResultCode } from '../../shared/result-codes'
import { getResultMessage } from '@/utils/result-code'
import { emitServiceError } from './service-events'

export const DEFAULT_TIMEOUT = 15_000
export const FILE_TRANSFER_TIMEOUT = 300_000
export const MSG_RSP_COMMON = 0xff00

export type ServiceErrorKind =
  | 'unauthenticated'
  | 'device-unauthorized'
  | 'business'
  | 'protocol'

export class ServiceError extends Error {
  constructor(
    message: string,
    readonly resultCode: number,
    readonly kind: ServiceErrorKind,
  ) {
    super(message)
    this.name = 'ServiceError'
  }
}

function throwServiceError(error: ServiceError): never {
  emitServiceError(error)
  throw error
}

export function handleResultError(resultCode: number, errorMessage: string): void {
  if (resultCode === ResultCode.SUCCESS) {
    return throwServiceError(
      new ServiceError(errorMessage || '装置返回失败但结果码为成功', resultCode, 'protocol'),
    )
  }

  if (resultCode === ResultCode.UNAUTHENTICATED) {
    return throwServiceError(new ServiceError('会话已失效', resultCode, 'unauthenticated'))
  }

  if (resultCode === ResultCode.DEVICE_UNAUTHORIZED) {
    return throwServiceError(new ServiceError('装置未授权', resultCode, 'device-unauthorized'))
  }

  const message = getResultMessage(resultCode, errorMessage) ?? (errorMessage || '操作失败')
  return throwServiceError(new ServiceError(message, resultCode, 'business'))
}

export function handleCommonResponse(response: usb_control.RspCommon): void {
  if (response.success) {
    if (response.resultCode !== ResultCode.SUCCESS) {
      return throwServiceError(
        new ServiceError('装置返回成功但结果码非零', response.resultCode, 'protocol'),
      )
    }
    return
  }
  handleResultError(response.resultCode, response.errorMessage)
}

export async function sendCommand(
  requestMsgType: number,
  payload: Uint8Array,
  expectedResponseMsgType: number,
  timeout: number = DEFAULT_TIMEOUT,
): Promise<Uint8Array> {
  const response = await window.desktopApi.tls.send(requestMsgType, payload, timeout)

  if (response.msgType === MSG_RSP_COMMON) {
    const commonResponse = usb_control.RspCommon.decode(response.payload)
    handleCommonResponse(commonResponse)
    if (expectedResponseMsgType !== MSG_RSP_COMMON) {
      return throwServiceError(
        new ServiceError('装置返回了非预期的通用成功响应', 0, 'protocol'),
      )
    }
    return response.payload
  }

  if (response.msgType !== expectedResponseMsgType) {
    return throwServiceError(
      new ServiceError(
        `响应消息类型不匹配：期望 0x${expectedResponseMsgType.toString(16)}，实际 0x${response.msgType.toString(16)}`,
        0,
        'protocol',
      ),
    )
  }

  return response.payload
}
