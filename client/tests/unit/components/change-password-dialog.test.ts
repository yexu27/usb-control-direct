import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import ChangePasswordDialog from '../../../src/renderer/components/ChangePasswordDialog.vue'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { changePassword } from '../../../src/renderer/services/user-service'
import { ElMessage } from 'element-plus'
import { ServiceError } from '../../../src/renderer/services/send-command'

vi.mock('../../../src/renderer/services/user-service', () => ({
  changePassword: vi.fn(),
  listUsers: vi.fn(),
}))

vi.mock('element-plus', () => ({
  ElMessage: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

const validate = vi.fn().mockResolvedValue(true)
const clearValidate = vi.fn()
const changePasswordMock = vi.mocked(changePassword)

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
    expect(ElMessage.success).toHaveBeenCalledWith('密码修改成功')
    expect(wrapper.emitted('update:visible')).toEqual([[false]])
  })

  it('服务失败时保留弹窗并展示错误', async () => {
    changePasswordMock.mockRejectedValue(new Error('旧密码错误'))
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm

    await component.handleSubmit()

    expect(ElMessage.error).toHaveBeenCalledWith('旧密码错误')
    expect(wrapper.emitted('update:visible')).toBeUndefined()
  })

  it('会话失效时不额外弹出错误提示', async () => {
    changePasswordMock.mockRejectedValue(new ServiceError('会话已失效', 0x0001, 'unauthenticated'))
    const wrapper = mountDialog()
    const component = wrapper.vm as unknown as ChangePasswordDialogVm

    await component.handleSubmit()

    expect(ElMessage.error).not.toHaveBeenCalled()
  })
})
