import { encodeFrame } from './frame-codec'
import type { TlsResponse } from '../../shared/tls-response'

const RETRYABLE_COMMANDS = new Set([
  0x0100, 0x0102, 0x0200, 0x0400, 0x0500, 0x0600, 0x0003,
])

const FILE_TRANSFER_COMMANDS = new Set([
  0x0007, 0x0300, 0x0302, 0x0402, 0x0502, 0x0503,
])

const DEFAULT_TIMEOUT = 15_000
const FILE_TRANSFER_TIMEOUT = 300_000

interface PendingRequest {
  resolve: (response: TlsResponse) => void
  reject: (error: Error) => void
  timer: ReturnType<typeof setTimeout>
  msgType: number
  payload: Uint8Array
  retried: boolean
}

export class RequestDispatcher {
  private nextSeqId = 1
  private pending = new Map<number, PendingRequest>()
  private writeFn: (frame: Buffer) => void

  constructor(writeFn: (frame: Buffer) => void) {
    this.writeFn = writeFn
  }

  dispatch(msgType: number, payload: Uint8Array, timeout?: number): Promise<TlsResponse> {
    const resolvedTimeout =
      timeout ?? (FILE_TRANSFER_COMMANDS.has(msgType) ? FILE_TRANSFER_TIMEOUT : DEFAULT_TIMEOUT)

    return new Promise<TlsResponse>((resolve, reject) => {
      const seqId = this.nextSeqId++
      this.sendAndTrack(seqId, msgType, payload, resolvedTimeout, resolve, reject, false)
    })
  }

  private sendAndTrack(
    seqId: number,
    msgType: number,
    payload: Uint8Array,
    timeout: number,
    resolve: (response: TlsResponse) => void,
    reject: (error: Error) => void,
    retried: boolean,
  ): void {
    const timer = setTimeout(() => {
      this.pending.delete(seqId)

      if (!retried && RETRYABLE_COMMANDS.has(msgType)) {
        const retrySeqId = this.nextSeqId++
        this.sendAndTrack(retrySeqId, msgType, payload, timeout, resolve, reject, true)
        return
      }

      reject(new Error(`Request timeout: msgType=0x${msgType.toString(16)}, seqId=${seqId}`))
    }, timeout)

    this.pending.set(seqId, { resolve, reject, timer, msgType, payload, retried })

    const frame = encodeFrame(msgType, seqId, payload)
    this.writeFn(frame)
  }

  handleResponse(seqId: number, msgType: number, payload: Uint8Array): void {
    const request = this.pending.get(seqId)
    if (request == null) {
      return
    }

    clearTimeout(request.timer)
    this.pending.delete(seqId)
    request.resolve({ msgType, payload })
  }

  rejectAll(reason: Error): void {
    for (const [, request] of this.pending) {
      clearTimeout(request.timer)
      request.reject(reason)
    }
    this.pending.clear()
  }
}
