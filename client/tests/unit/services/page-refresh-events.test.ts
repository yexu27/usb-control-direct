import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  emitPageRefresh,
  onPageRefresh,
  resetPageRefreshListenersForTest,
} from '../../../src/renderer/services/page-refresh-events'

describe('page-refresh-events', () => {
  beforeEach(() => {
    resetPageRefreshListenersForTest()
  })

  it('notifies registered listeners with the refresh reason', () => {
    const listener = vi.fn()

    onPageRefresh(listener)
    emitPageRefresh('reconnect')

    expect(listener).toHaveBeenCalledWith('reconnect')
  })

  it('does not notify unsubscribed listeners', () => {
    const listener = vi.fn()
    const unsubscribe = onPageRefresh(listener)

    unsubscribe()
    emitPageRefresh('reconnect')

    expect(listener).not.toHaveBeenCalled()
  })
})
