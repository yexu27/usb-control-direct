import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, type PropType } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import AddBlacklistDialog from '../../../src/renderer/components/file-policy/AddBlacklistDialog.vue'

const validate = vi.fn().mockResolvedValue(true)
const clearValidate = vi.fn()

function createDeferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

const ElFormStub = defineComponent({
  setup(_props, { expose, slots }) {
    expose({ validate, clearValidate })
    return () => h('form', slots.default?.())
  },
})

const ElDialogStub = defineComponent({
  props: {
    modelValue: { type: Boolean, required: true },
    beforeClose: Function as PropType<(done: () => void) => void>,
  },
  emits: ['update:modelValue'],
  setup(props, { emit, slots }) {
    function requestClose(): void {
      const done = () => emit('update:modelValue', false)
      if (props.beforeClose != null) {
        props.beforeClose(done)
      } else {
        done()
      }
    }
    return () => h('section', [
      h('button', { 'data-testid': 'dialog-close', onClick: requestClose }, '关闭'),
      slots.default?.(),
      slots.footer?.(),
    ])
  },
})

const ElInputStub = defineComponent({
  inheritAttrs: false,
  props: {
    modelValue: { type: String, required: true },
    disabled: Boolean,
  },
  emits: ['update:modelValue'],
  setup(props, { attrs, emit }) {
    return () => h('input', {
      ...attrs,
      value: props.modelValue,
      disabled: props.disabled,
      onInput: (event: Event) => {
        emit('update:modelValue', (event.target as HTMLInputElement).value)
      },
    })
  },
})

const ElButtonStub = defineComponent({
  inheritAttrs: false,
  props: { disabled: Boolean, loading: Boolean },
  setup(props, { attrs, slots }) {
    return () => h('button', { ...attrs, disabled: props.disabled }, slots.default?.())
  },
})

interface DialogVm {
  rules: Record<string, Array<{
    validator?: (
      rule: unknown,
      value: string,
      callback: (error?: Error) => void,
    ) => void
  }>>
}

function mountDialog(submitting = false) {
  return mount(AddBlacklistDialog, {
    props: { visible: true, submitting },
    global: {
      stubs: {
        ElDialog: ElDialogStub,
        ElForm: ElFormStub,
        ElFormItem: { template: '<label><slot /></label>' },
        ElAlert: { props: ['title'], template: '<strong>{{ title }}</strong>' },
        ElInput: ElInputStub,
        ElButton: ElButtonStub,
      },
    },
  })
}

describe('AddBlacklistDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    validate.mockResolvedValue(true)
  })

  it('通过输入与确定按钮提交一个规范化条目', async () => {
    validate.mockResolvedValue(true)
    const wrapper = mountDialog()

    await wrapper.get('[data-testid="blacklist-extension-input"]').setValue('  .DoC_X  ')
    await wrapper.get('[data-testid="blacklist-description-input"]').setValue('Office 文档')
    await wrapper.get('[data-testid="blacklist-submit"]').trigger('click')
    await flushPromises()

    expect(validate).toHaveBeenCalledTimes(1)
    expect(wrapper.emitted('submit')).toEqual([[
      { extension: '.doc_x', description: 'Office 文档' },
    ]])
  })

  it('在弹窗内展示提交失败原因', () => {
    const wrapper = mount(AddBlacklistDialog, {
      props: {
        visible: true,
        submitting: false,
        errorMessage: '该文件后缀已在黑名单中',
      },
      global: {
        stubs: {
          ElDialog: ElDialogStub,
          ElForm: ElFormStub,
          ElFormItem: { template: '<label><slot /></label>' },
          ElAlert: { props: ['title'], template: '<strong>{{ title }}</strong>' },
          ElInput: ElInputStub,
          ElButton: ElButtonStub,
        },
      },
    })

    expect(wrapper.text()).toContain('该文件后缀已在黑名单中')
  })

  it('扩展名必须含点号且仅允许字母数字、下划线和连字符', () => {
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as DialogVm
    const validator = component.rules.extension.find((rule) => rule.validator)?.validator
    const validCallback = vi.fn()
    const invalidCallback = vi.fn()

    validator?.({}, ' .Tar-GZ ', validCallback)
    validator?.({}, '.tar.gz', invalidCallback)

    expect(validCallback).toHaveBeenCalledWith(undefined)
    expect(invalidCallback).toHaveBeenCalledWith(
      expect.objectContaining({ message: '请输入正确的文件后缀' }),
    )
  })

  it('Element Plus 表单验证失败后释放锁并允许重试', async () => {
    validate.mockResolvedValue(false)
    const wrapper = mountDialog()
    const submit = wrapper.get('[data-testid="blacklist-submit"]')
    await submit.trigger('click')
    await flushPromises()

    expect(wrapper.emitted('submit')).toBeUndefined()
    validate.mockResolvedValue(true)
    await submit.trigger('click')
    await flushPromises()
    expect(validate).toHaveBeenCalledTimes(2)
    expect(wrapper.emitted('submit')).toHaveLength(1)
  })

  it('取消关闭后重置表单输入', async () => {
    const wrapper = mountDialog()
    await wrapper.get('[data-testid="blacklist-extension-input"]').setValue('.zip')
    await wrapper.get('[data-testid="blacklist-description-input"]').setValue('压缩包')

    await wrapper.get('[data-testid="blacklist-cancel"]').trigger('click')
    expect(wrapper.emitted('update:visible')).toEqual([[false]])
    await wrapper.setProps({ visible: false })
    await wrapper.setProps({ visible: true })

    const extensionInput = wrapper.get('[data-testid="blacklist-extension-input"]')
      .element as HTMLInputElement
    const descriptionInput = wrapper.get('[data-testid="blacklist-description-input"]')
      .element as HTMLInputElement
    expect(extensionInput.value).toBe('')
    expect(descriptionInput.value).toBe('')
  })

  it('提交中禁用输入、取消和确定，并拒绝弹窗关闭', async () => {
    const wrapper = mountDialog(true)
    const extensionInput = wrapper.get('[data-testid="blacklist-extension-input"]')

    expect(extensionInput.attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid="blacklist-cancel"]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid="blacklist-submit"]').attributes('disabled')).toBeDefined()
    await wrapper.get('[data-testid="dialog-close"]').trigger('click')
    await wrapper.get('[data-testid="blacklist-cancel"]').trigger('click')
    await wrapper.get('[data-testid="blacklist-submit"]').trigger('click')

    expect(wrapper.emitted('update:visible')).toBeUndefined()
    expect(wrapper.emitted('submit')).toBeUndefined()
  })

  it('同一 tick 连续点击确定仅执行一次验证和提交', async () => {
    const deferred = createDeferred<boolean>()
    validate.mockReturnValue(deferred.promise)
    const wrapper = mountDialog()
    await wrapper.get('[data-testid="blacklist-extension-input"]').setValue('.zip')
    const submit = wrapper.get('[data-testid="blacklist-submit"]')

    const firstClick = submit.trigger('click')
    const secondClick = submit.trigger('click')
    await Promise.all([firstClick, secondClick])
    expect(validate).toHaveBeenCalledTimes(1)
    expect(wrapper.emitted('submit')).toBeUndefined()

    deferred.resolve(true)
    await flushPromises()

    expect(wrapper.emitted('submit')).toHaveLength(1)
  })

  it('验证抛错后释放本地锁并允许重试', async () => {
    const deferred = createDeferred<boolean>()
    validate.mockReturnValueOnce(deferred.promise).mockResolvedValueOnce(true)
    const wrapper = mountDialog()
    await wrapper.get('[data-testid="blacklist-extension-input"]').setValue('.zip')
    const submit = wrapper.get('[data-testid="blacklist-submit"]')

    await submit.trigger('click')
    deferred.reject(new Error('验证异常'))
    await flushPromises()
    expect(wrapper.emitted('submit')).toBeUndefined()

    await submit.trigger('click')
    await flushPromises()

    expect(validate).toHaveBeenCalledTimes(2)
    expect(wrapper.emitted('submit')).toHaveLength(1)
  })

  it('外部关闭重置验证锁，旧验证结果不再提交', async () => {
    const deferred = createDeferred<boolean>()
    validate.mockReturnValue(deferred.promise)
    const wrapper = mountDialog()
    await wrapper.get('[data-testid="blacklist-extension-input"]').setValue('.zip')

    await wrapper.get('[data-testid="blacklist-submit"]').trigger('click')
    await wrapper.setProps({ visible: false })
    deferred.resolve(true)
    await flushPromises()

    expect(wrapper.emitted('submit')).toBeUndefined()
    await wrapper.setProps({ visible: true })
    expect(wrapper.get('[data-testid="blacklist-submit"]').attributes('disabled')).toBeUndefined()
  })
})
