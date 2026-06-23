// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'
import AddWhitelistDialog from '../../../src/renderer/components/whitelist/AddWhitelistDialog.vue'

interface CandidateDevice {
  serialNumber: string
  vid: string
  pid: string
  deviceName: string
  capacityBytes: number
  deviceType: 'storage'
  addable: boolean
  unavailableReason: string
}
interface AddWhitelistDialogVm {
  form: {
    permission: 'readonly' | 'readwrite'
  }
}

const candidates: CandidateDevice[] = [
  {
    serialNumber: 'SN-001', vid: '0781', pid: '5591', deviceName: 'SanDisk',
    capacityBytes: 32_000_000_000, deviceType: 'storage', addable: true,
    unavailableReason: '',
  },
  {
    serialNumber: '', vid: '1234', pid: '5678', deviceName: '异常设备',
    capacityBytes: 1, deviceType: 'storage', addable: false,
    unavailableReason: '设备标识异常',
  },
]
let validateForm: () => Promise<boolean> = () => Promise.resolve(true)

const ElDialogStub = defineComponent({
  props: ['modelValue', 'title'], emits: ['update:model-value'],
  template: '<section v-if="modelValue"><h2>{{ title }}</h2><slot/><slot name="footer"/></section>',
})
const ElFormStub = defineComponent({
  setup(_, { expose }) {
    expose({ validate: () => validateForm(), clearValidate: vi.fn() })
  },
  template: '<form><slot/></form>',
})
const ElSelectStub = defineComponent({
  props: ['modelValue', 'disabled'], emits: ['update:modelValue'],
  template: '<select data-testid="candidate-select" :disabled="disabled" :value="modelValue" @change="$emit(\'update:modelValue\', $event.target.value)"><slot/></select>',
})
const ElOptionStub = defineComponent({
  props: ['value', 'label', 'disabled'],
  template: '<option :value="value" :disabled="disabled">{{ label }}<slot/></option>',
})
const ElInputStub = defineComponent({
  props: ['modelValue', 'disabled'], emits: ['update:modelValue'],
  template: '<input :disabled="disabled" :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)"/>',
})
const ElRadioGroupStub = defineComponent({
  inheritAttrs: false,
  props: ['modelValue', 'disabled'],
  emits: ['update:modelValue', 'update:model-value'],
  setup(_props, { emit }) {
    return {
      updateValue: (value: 'readonly' | 'readwrite') => {
        emit('update:modelValue', value)
        emit('update:model-value', value)
      },
    }
  },
  template: `<div>
    <button type="button" data-value="readonly" @click="updateValue('readonly')">只读</button>
    <button type="button" data-value="readwrite" @click="updateValue('readwrite')">读写</button>
  </div>`,
})
const ElRadioStub = defineComponent({
  props: ['value', 'label', 'disabled'], inject: { radioGroup: { default: null } },
  template: '<button type="button" :disabled="disabled" :data-value="value"><slot/></button>',
})

function mountDialog(props: Record<string, unknown> = {}) {
  return mount(AddWhitelistDialog, {
    props: { visible: true, source: 'management', candidates, loading: false, submitting: false, ...props },
    global: { stubs: {
      ElDialog: ElDialogStub, ElForm: ElFormStub, ElFormItem: { template: '<label><slot/></label>' },
      ElAlert: { props: ['title'], template: '<strong>{{ title }}</strong>' },
      ElSelect: ElSelectStub, ElOption: ElOptionStub, ElInput: ElInputStub,
      ElRadioGroup: ElRadioGroupStub, ElRadio: ElRadioStub,
      ElButton: { template: '<button :disabled="$attrs.disabled" @click="$emit(\'click\')"><slot/></button>' },
      ElEmpty: { template: '<div><slot/></div>' },
    } },
  })
}

describe('AddWhitelistDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    validateForm = () => Promise.resolve(true)
  })

  it('显示设备名称和序列号，并禁用空序列号且展示设备标识异常', () => {
    const wrapper = mountDialog()
    expect(wrapper.text()).toContain('SanDisk（SN-001）')
    expect(wrapper.text()).toContain('异常设备（--）')
    expect(wrapper.text()).toContain('设备标识异常')
    const options = wrapper.findAll('option')
    expect(options[1].attributes('disabled')).toBeDefined()
    expect(wrapper.text()).not.toContain('keyboard')
  })

  it('在弹窗内展示添加失败原因', () => {
    const wrapper = mountDialog({ errorMessage: '该设备已在白名单中' })

    expect(wrapper.text()).toContain('该设备已在白名单中')
  })

  it('只允许 readonly/readwrite，提交真实表单且同 tick 只发一次', async () => {
    const wrapper = mountDialog()
    expect(wrapper.findAll('[data-value]').map((item) => item.attributes('data-value'))).toEqual([
      'readonly', 'readwrite',
    ])
    await wrapper.get('[data-testid="candidate-select"]').setValue('SN-001')
    ;(wrapper.vm as unknown as AddWhitelistDialogVm).form.permission = 'readwrite'
    await nextTick()
    const inputs = wrapper.findAll('input')
    await inputs[0].setValue('财务盘')
    const submit = wrapper.get('[data-testid="whitelist-add-submit"]')
    const first = submit.trigger('click')
    const second = submit.trigger('click')
    await Promise.all([first, second])
    expect(wrapper.emitted('submit')).toEqual([[
      { candidate: candidates[0], description: '财务盘', permission: 'readwrite' },
    ]])
  })

  it('关闭时清空选择和表单，异步验证迟到不会提交', async () => {
    let resolveValidation: (valid: boolean) => void = () => {
      throw new Error('验证 Promise 尚未初始化')
    }
    validateForm = () => new Promise((resolve) => { resolveValidation = resolve })
    const wrapper = mountDialog()
    await wrapper.get('[data-testid="candidate-select"]').setValue('SN-001')
    await wrapper.find('input').setValue('旧说明')
    const vm = wrapper.vm as unknown as { handleSubmit: () => Promise<void> }
    const pending = vm.handleSubmit()
    await nextTick()
    expect(wrapper.get('[data-testid="whitelist-add-submit"]').attributes('disabled'))
      .toBeDefined()
    await wrapper.setProps({ visible: false })
    resolveValidation(true)
    await pending
    await wrapper.setProps({ visible: true })
    await nextTick()
    expect((wrapper.get('[data-testid="candidate-select"]').element as HTMLSelectElement).value)
      .toBe('')
    expect((wrapper.find('input').element as HTMLInputElement).value).toBe('')
    expect(wrapper.emitted('submit')).toBeUndefined()
  })

  it('父级请求期间禁用交互', async () => {
    const wrapper = mountDialog({ submitting: true })
    expect(wrapper.get('[data-testid="candidate-select"]').attributes('disabled')).toBeDefined()
    ;(wrapper.vm as unknown as { handleSubmit: () => Promise<void> }).handleSubmit()
    await flushPromises()
    expect(wrapper.emitted('submit')).toBeUndefined()
  })

  it('实例卸载使迟到验证失效，新来源实例表单为空', async () => {
    let rejectValidation: (reason: unknown) => void = () => {
      throw new Error('验证 Promise 尚未初始化')
    }
    validateForm = () => new Promise((_resolve, reject) => { rejectValidation = reject })
    const oldWrapper = mountDialog({ source: 'device' })
    await oldWrapper.get('[data-testid="candidate-select"]').setValue('SN-001')
    const pending = (oldWrapper.vm as unknown as { handleSubmit: () => Promise<void> })
      .handleSubmit()
    await nextTick()
    oldWrapper.unmount()

    const newWrapper = mountDialog({ source: 'management' })
    rejectValidation(new Error('迟到验证失败'))
    await expect(pending).resolves.toBeUndefined()

    expect(oldWrapper.emitted('submit')).toBeUndefined()
    expect((newWrapper.get('[data-testid="candidate-select"]').element as HTMLSelectElement).value)
      .toBe('')
  })
})
