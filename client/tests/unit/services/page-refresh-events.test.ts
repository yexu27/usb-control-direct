import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises } from '@vue/test-utils'
import {
  emitPageRefresh,
  onPageRefresh,
  resetPageRefreshListenersForTest,
} from '../../../src/renderer/services/page-refresh-events'

describe('page-refresh-events', () => {
  beforeEach(() => {
    resetPageRefreshListenersForTest()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('notifies registered listeners with the refresh reason', async () => {
    const listener = vi.fn()

    onPageRefresh(listener)
    emitPageRefresh('reconnect')
    await flushPromises()

    expect(listener).toHaveBeenCalledWith('reconnect')
  })

  it('does not notify unsubscribed listeners', async () => {
    const listener = vi.fn()
    const unsubscribe = onPageRefresh(listener)

    unsubscribe()
    emitPageRefresh('reconnect')
    await flushPromises()

    expect(listener).not.toHaveBeenCalled()
  })

  it('isolates listener errors and continues dispatching', async () => {
    const first = vi.fn(() => {
      throw new Error('listener failed')
    })
    const second = vi.fn()
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})

    onPageRefresh(first)
    onPageRefresh(second)

    expect(() => emitPageRefresh('reconnect')).not.toThrow()
    await flushPromises()
    expect(first).toHaveBeenCalledWith('reconnect')
    expect(second).toHaveBeenCalledWith('reconnect')
    expect(consoleError).toHaveBeenCalled()
  })

  it('isolates async listener rejection and continues dispatching', async () => {
    const first = vi.fn(() => Promise.reject(new Error('async listener failed')))
    const second = vi.fn()
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})

    onPageRefresh(first)
    onPageRefresh(second)

    expect(() => emitPageRefresh('reconnect')).not.toThrow()
    await flushPromises()

    expect(first).toHaveBeenCalledWith('reconnect')
    expect(second).toHaveBeenCalledWith('reconnect')
    expect(consoleError).toHaveBeenCalledWith(
      '页面刷新监听器执行失败',
      expect.any(Error),
    )
  })
})
