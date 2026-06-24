// @vitest-environment happy-dom

import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import AppToastHost from '../../../src/renderer/components/AppToastHost.vue'
import { useAppToastStore } from '../../../src/renderer/stores/app-toast'

describe('AppToastHost', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders success toast at app top center', async () => {
    const wrapper = mount(AppToastHost, {
      global: {
        stubs: {
          ElIcon: { template: '<i><slot /></i>' },
          Check: { template: '<span />' },
        },
      },
    })
    const store = useAppToastStore()

    store.showSuccess('策略导出成功')
    await wrapper.vm.$nextTick()

    const toast = wrapper.get('[data-testid="app-toast"]')
    expect(toast.text()).toContain('策略导出成功')
    expect(toast.classes()).toContain('app-toast-success')
  })
})
