// @vitest-environment happy-dom

import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import EditWhitelistDialog from '../../../src/renderer/components/whitelist/EditWhitelistDialog.vue'

const ElFormStub = defineComponent({
  setup(_, { expose }) { expose({ validate: () => Promise.resolve(true), clearValidate: vi.fn() }) },
  template: '<form><slot/></form>',
})

function mountDialog(props: Record<string, unknown> = {}) {
  return mount(EditWhitelistDialog, {
    props: {
      visible: true, serialNumber: 'SN-EDIT', currentDescription: '旧说明',
      currentPermission: 'readwrite', submitting: false, ...props,
    },
    global: { stubs: {
      ElDialog: { props: ['modelValue'], template: '<section v-if="modelValue"><slot/><slot name="footer"/></section>' },
      ElForm: ElFormStub, ElFormItem: { template: '<label><slot/></label>' },
      ElInput: {
        props: ['modelValue', 'disabled'], emits: ['update:modelValue'],
        template: '<input :disabled="disabled" :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)"/>',
      },
      ElRadioGroup: {
        props: ['modelValue'], emits: ['update:modelValue'],
        template: `<div>
          <button type="button" data-value="readonly" @click="$emit('update:modelValue', 'readonly')">只读</button>
          <button type="button" data-value="readwrite" @click="$emit('update:modelValue', 'readwrite')">读写</button>
        </div>`,
      },
      ElRadio: { props: ['value'], template: '<button type="button" :data-value="value"><slot/></button>' },
      ElButton: { template: '<button :disabled="$attrs.disabled"><slot/></button>' },
    } },
  })
}

describe('EditWhitelistDialog', () => {
  it('序列号只读且权限只提供 readonly/readwrite', () => {
    const wrapper = mountDialog()
    expect(wrapper.findAll('input')[0].attributes('disabled')).toBeDefined()
    expect(wrapper.findAll('[data-value]').map((item) => item.attributes('data-value'))).toEqual([
      'readonly', 'readwrite',
    ])
  })

  it('空说明合法提交并防止同 tick 重复提交', async () => {
    const wrapper = mountDialog()
    await wrapper.findAll('input')[1].setValue('')
    await wrapper.get('[data-value="readonly"]').trigger('click')
    const submit = wrapper.get('[data-testid="whitelist-edit-submit"]')
    await Promise.all([submit.trigger('click'), submit.trigger('click')])
    expect(wrapper.emitted('submit')).toEqual([[
      { description: '', permission: 'readonly' },
    ]])
  })

  it('关闭后按最新目标重置表单且提交中禁用', async () => {
    const wrapper = mountDialog()
    await wrapper.findAll('input')[1].setValue('临时值')
    await wrapper.setProps({ visible: false })
    await wrapper.setProps({ serialNumber: 'SN-NEXT', currentDescription: '新说明', currentPermission: 'readonly' })
    await wrapper.setProps({ visible: true })
    expect((wrapper.findAll('input')[0].element as HTMLInputElement).value).toBe('SN-NEXT')
    expect((wrapper.findAll('input')[1].element as HTMLInputElement).value).toBe('新说明')
    await wrapper.setProps({ submitting: true })
    ;(wrapper.vm as unknown as { handleSubmit: () => Promise<void> }).handleSubmit()
    expect(wrapper.emitted('submit')).toBeUndefined()
  })
})
