import { describe, expect, it } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import ProgressDialog from '../../../src/renderer/components/ProgressDialog.vue'

describe('ProgressDialog', () => {
  it('处理期间禁止用户关闭', () => {
    const wrapper = shallowMount(ProgressDialog, {
      props: { visible: true, title: '导入策略' },
      global: {
        stubs: {
          ElDialog: true,
          ElIcon: true,
          Loading: true,
        },
      },
    })

    const dialog = wrapper.get('el-dialog-stub')
    expect(dialog.attributes('close-on-click-modal')).toBe('false')
    expect(dialog.attributes('close-on-press-escape')).toBe('false')
    expect(dialog.attributes('show-close')).toBe('false')
  })
})
