import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { HeartbeatManager } from '../../../src/main/tls/heartbeat'

describe('HeartbeatManager', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('calls sendFn every 30 seconds after start', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const hb = new HeartbeatManager()
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    expect(sendFn).toHaveBeenCalledTimes(1)

    vi.advanceTimersByTime(30_000)
    expect(sendFn).toHaveBeenCalledTimes(2)

    hb.stop()
  })

  it('resets miss counter on heartbeat response', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const onTimeout = vi.fn()
    const hb = new HeartbeatManager()
    hb.onTimeout = onTimeout
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)
    hb.onHeartbeatResponse()

    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)
    hb.onHeartbeatResponse()

    expect(onTimeout).not.toHaveBeenCalled()
    hb.stop()
  })

  it('triggers onTimeout after 3 consecutive misses', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const onTimeout = vi.fn()
    const hb = new HeartbeatManager()
    hb.onTimeout = onTimeout
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)
    vi.advanceTimersByTime(30_000)

    expect(onTimeout).toHaveBeenCalledTimes(1)
    hb.stop()
  })

  it('stops sending after stop is called', () => {
    const sendFn = vi.fn().mockResolvedValue(undefined)
    const hb = new HeartbeatManager()
    hb.start(sendFn)

    vi.advanceTimersByTime(30_000)
    expect(sendFn).toHaveBeenCalledTimes(1)

    hb.stop()
    vi.advanceTimersByTime(60_000)
    expect(sendFn).toHaveBeenCalledTimes(1)
  })
})
