// @vitest-environment happy-dom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { emitPageRefresh, resetPageRefreshListenersForTest } from '../../../src/renderer/services/page-refresh-events'
import { useDeviceBackedPageRefresh } from '../../../src/renderer/composables/use-device-backed-page-refresh'

function mountWithRefresh(refresh: () => Promise<void>): ReturnType<typeof mount> {
  const component = defineComponent({
    name: 'DeviceBackedPage',
    setup() {
      useDeviceBackedPageRefresh(refresh)
      return {}
    },
    template: '<div>page</div>',
  })

  return mount(component)
}

describe('use-device-backed-page-refresh', () => {
  beforeEach(() => {
    resetPageRefreshListenersForTest()
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('mount 时刷新失败会被捕获并记录日志', async () => {
    const error = new Error('mount failed')
    const refresh = vi.fn(() => Promise.reject(error))
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})

    const wrapper = mountWithRefresh(refresh)
    await flushPromises()

    expect(refresh).toHaveBeenCalledTimes(1)
    expect(consoleError).toHaveBeenCalledWith('页面数据刷新失败', error)
    wrapper.unmount()
  })

  it('重连事件触发的刷新失败也会被捕获并记录日志', async () => {
    const error = new Error('reconnect refresh failed')
    const refresh = vi.fn(() => Promise.reject(error))
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})

    const wrapper = mountWithRefresh(refresh)
    await flushPromises()
    expect(refresh).toHaveBeenCalledTimes(1)

    consoleError.mockClear()
    emitPageRefresh('reconnect')
    await flushPromises()

    expect(refresh).toHaveBeenCalledTimes(2)
    expect(consoleError).toHaveBeenCalledWith('页面数据刷新失败', error)
    wrapper.unmount()
  })
})
