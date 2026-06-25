import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { RequestDispatcher } from '../../../src/main/tls/request-dispatcher'

describe('RequestDispatcher', () => {
  let dispatcher: RequestDispatcher
  let writtenFrames: Buffer[]

  beforeEach(() => {
    vi.useFakeTimers()
    writtenFrames = []
    dispatcher = new RequestDispatcher((frame: Buffer) => {
      writtenFrames.push(frame)
    })
  })

  afterEach(() => {
    dispatcher.rejectAll(new Error('cleanup'))
    vi.useRealTimers()
  })

  it('assigns incrementing sequence IDs', async () => {
    const p1 = dispatcher.dispatch(0x0001, new Uint8Array(0))
    const p2 = dispatcher.dispatch(0x0002, new Uint8Array(0))

    dispatcher.handleResponse(1, 0x0002, new Uint8Array(0))
    dispatcher.handleResponse(2, 0x0003, new Uint8Array(0))

    await p1
    await p2
    expect(writtenFrames.length).toBe(2)
  })

  it('resolves when matching response arrives', async () => {
    const promise = dispatcher.dispatch(0x0001, new Uint8Array([0x01]))
    dispatcher.handleResponse(1, 0x0002, new Uint8Array([0x02]))
    const result = await promise
    expect(result).toEqual({
      msgType: 0x0002,
      payload: new Uint8Array([0x02]),
    })
  })

  it('rejects on timeout for non-retryable command', async () => {
    const promise = dispatcher.dispatch(0x0104, new Uint8Array(0), 1000)
    vi.advanceTimersByTime(1001)
    await expect(promise).rejects.toThrow('timeout')
  })

  it('retries once for read-only query command on timeout', async () => {
    const promise = dispatcher.dispatch(0x0100, new Uint8Array(0), 1000)

    vi.advanceTimersByTime(1001)
    expect(writtenFrames.length).toBe(2)

    dispatcher.handleResponse(2, 0x0101, new Uint8Array([0x0a]))
    const result = await promise
    expect(result).toEqual({
      msgType: 0x0101,
      payload: new Uint8Array([0x0a]),
    })
  })

  it('rejects after retry also times out', async () => {
    const promise = dispatcher.dispatch(0x0100, new Uint8Array(0), 1000)

    vi.advanceTimersByTime(1001)
    vi.advanceTimersByTime(1001)

    await expect(promise).rejects.toThrow('timeout')
  })

  it('rejectAll rejects all pending requests', async () => {
    const p1 = dispatcher.dispatch(0x0001, new Uint8Array(0))
    const p2 = dispatcher.dispatch(0x0002, new Uint8Array(0))

    dispatcher.rejectAll(new Error('disconnected'))

    await expect(p1).rejects.toThrow('disconnected')
    await expect(p2).rejects.toThrow('disconnected')
  })

  it('冻结后拒绝新增请求并保留明确错误', async () => {
    dispatcher.freeze(new Error('正在登出，拒绝新请求'))

    await expect(dispatcher.dispatch(0x0200, new Uint8Array())).rejects.toThrow(
      '正在登出，拒绝新请求',
    )
    expect(writtenFrames.length).toBe(0)
  })

  it('ignores late response after timeout', () => {
    dispatcher.dispatch(0x0104, new Uint8Array(0), 1000).catch(() => {})
    vi.advanceTimersByTime(1001)

    expect(() => {
      dispatcher.handleResponse(1, 0xff00, new Uint8Array(0))
    }).not.toThrow()
  })
})
