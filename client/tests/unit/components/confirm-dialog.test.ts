import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ConfirmDialog from '../../../src/renderer/components/ConfirmDialog.vue'

const stubs = {
  ElDialog: {
    template: '<section><slot /><slot name="footer" /></section>',
  },
  ElButton: {
    template: '<button @click="$emit(\'click\')"><slot /></button>',
  },
}

describe('ConfirmDialog', () => {
  it('确认时发送 confirm 并关闭', async () => {
    const wrapper = mount(ConfirmDialog, {
      props: { visible: true, title: '确认', message: '是否继续？' },
      global: { stubs },
    })

    await wrapper.get('button:last-child').trigger('click')

    expect(wrapper.emitted('confirm')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')).toEqual([[false]])
  })

  it('取消时发送 cancel 并关闭', async () => {
    const wrapper = mount(ConfirmDialog, {
      props: { visible: true, title: '确认', message: '是否继续？' },
      global: { stubs },
    })

    await wrapper.get('button:first-child').trigger('click')

    expect(wrapper.emitted('cancel')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')).toEqual([[false]])
  })
})
