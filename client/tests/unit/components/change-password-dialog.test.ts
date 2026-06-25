import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import ChangePasswordDialog from '../../../src/renderer/components/ChangePasswordDialog.vue'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { changePassword } from '../../../src/renderer/services/user-service'
import { ServiceError } from '../../../src/renderer/services/send-command'
import { showSuccessToast } from '../../../src/renderer/utils/operation-feedback'

vi.mock('../../../src/renderer/services/user-service', () => ({
  changePassword: vi.fn(),
  listUsers: vi.fn(),
}))

vi.mock('../../../src/renderer/utils/operation-feedback', () => ({
  showSuccessToast: vi.fn(),
  errorMessage: (error: unknown, fallback: string) => error instanceof Error ? error.message : fallback,
}))

const validate = vi.fn().mockResolvedValue(true)
const clearValidate = vi.fn()
const changePasswordMock = vi.mocked(changePassword)
const showSuccessToastMock = vi.mocked(showSuccessToast)

const ElFormStub = defineComponent({
  setup(_props, { expose, slots }) {
    expose({ validate, clearValidate })
    return () => h('form', slots.default?.())
  },
})

interface ChangePasswordDialogVm {
  passwordForm: {
    oldPassword: string
    newPassword: string
    confirmPassword: string
  }
  rules: Record<
    string,
    Array<{
      validator?: (
        rule: unknown,
        value: string,
        callback: (error?: Error) => void,
      ) => void
    }>
  >
  isSubmitting: boolean
  submitErrorMessage: string
  close: () => void
  handleSubmit: () => Promise<void>
}

describe('ChangePasswordDialog', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    validate.mockResolvedValue(true)
    useSessionStore().setSession({
      token: 'session-token',
      username: 'operator',
      role: 'operator',
      authStatus: 'authorized',
      authExpireTime: 0,
      deviceDescription: '',
    })
  })

  function mountDialog() {
    return mount(ChangePasswordDialog, {
      props: { visible: true },
      global: {
        stubs: {
          ElDialog: { template: '<section><slot /><slot name="footer" /></section>' },
          ElForm: ElFormStub,
          ElFormItem: { template: '<label><slot /></label>' },
          ElAlert: { props: ['title'], template: '<div data-testid="change-password-error">{{ title }}</div>' },
          ElInput: true,
          ElButton: true,
        },
      },
    })
  }

  it('表单有效时调用修改密码服务并关闭弹窗', async () => {
    changePasswordMock.mockResolvedValue(undefined)
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm
    component.passwordForm.oldPassword = 'old@1234'
    component.passwordForm.newPassword = 'new@1234'
    component.passwordForm.confirmPassword = 'new@1234'

    await component.handleSubmit()

    expect(changePasswordMock).toHaveBeenCalledWith(
      'session-token',
      'old@1234',
      'new@1234',
      'new@1234',
    )
    expect(showSuccessToastMock).toHaveBeenCalledWith('密码修改成功')
    expect(wrapper.emitted('update:visible')).toEqual([[false]])
    expect(component.passwordForm).toEqual({
      oldPassword: '',
      newPassword: '',
      confirmPassword: '',
    })
  })

  it('服务失败时保留弹窗并展示错误', async () => {
    changePasswordMock.mockRejectedValue(new Error('旧密码错误'))
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm

    await component.handleSubmit()

    expect(component.submitErrorMessage).toBe('旧密码错误')
    expect(wrapper.get('[data-testid="change-password-error"]').text()).toBe('旧密码错误')
    expect(wrapper.emitted('update:visible')).toBeUndefined()
  })

  it('会话失效时不额外弹出错误提示', async () => {
    changePasswordMock.mockRejectedValue(new ServiceError('会话已失效', 0x0001, 'unauthenticated'))
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm

    await component.handleSubmit()

    expect(component.submitErrorMessage).toBe('')
    expect(wrapper.find('[data-testid="change-password-error"]').exists()).toBe(false)
  })

  it('表单校验失败时保留弹窗并展示红色错误', async () => {
    validate.mockResolvedValue(false)
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm

    await component.handleSubmit()

    expect(changePasswordMock).not.toHaveBeenCalled()
    expect(component.submitErrorMessage).toBe('请检查密码填写内容')
    expect(wrapper.get('[data-testid="change-password-error"]').text()).toBe('请检查密码填写内容')
  })

  it('弱密码未通过复杂度校验', () => {
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm
    const validator = component.rules.newPassword.find(
      (rule) => rule.validator != null,
    )?.validator
    const callback = vi.fn()

    validator?.({}, '12345678', callback)

    expect(callback).toHaveBeenCalledWith(
      expect.objectContaining({ message: '密码至少包含字母、数字、特殊字符中的两种' }),
    )
  })

  it('两次新密码不一致时未通过校验', () => {
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm
    component.passwordForm.newPassword = 'new@1234'
    const validator = component.rules.confirmPassword.find(
      (rule) => rule.validator != null,
    )?.validator
    const callback = vi.fn()

    validator?.({}, 'different@1234', callback)

    expect(callback).toHaveBeenCalledWith(
      expect.objectContaining({ message: '两次输入的密码不一致' }),
    )
  })

  it('提交过程中拒绝重复请求和关闭操作', async () => {
    changePasswordMock.mockResolvedValue(undefined)
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm
    component.passwordForm.oldPassword = 'old@1234'
    component.passwordForm.newPassword = 'new@1234'
    component.passwordForm.confirmPassword = 'new@1234'

    const firstSubmit = component.handleSubmit()
    component.close()
    const secondSubmit = component.handleSubmit()
    await Promise.all([firstSubmit, secondSubmit])

    expect(changePasswordMock).toHaveBeenCalledTimes(1)
    expect(wrapper.emitted('update:visible')).toEqual([[false]])
  })
})
