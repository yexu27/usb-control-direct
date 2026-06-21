import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import LoginPage from '../../../src/renderer/pages/LoginPage.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { ServiceError } from '../../../src/renderer/services/send-command'

const push = vi.fn()
const validate = vi.fn().mockResolvedValue(true)

vi.mock('vue-router', () => ({
  useRouter: () => ({ push }),
}))

const ElFormStub = defineComponent({
  setup(_props, { expose, slots }) {
    expose({ validate })
    return () => h('form', slots.default?.())
  },
})

interface LoginPageVm {
  form: { ip: string; username: string; password: string }
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
  errorMessage: string
  handleSubmit: () => Promise<void>
}

function mountPage() {
  return mount(LoginPage, {
    global: {
      stubs: {
        ElForm: ElFormStub,
        ElFormItem: { template: '<label><slot /></label>' },
        ElInput: true,
        ElButton: true,
        ElAlert: true,
        ElIcon: true,
      },
    },
  })
}

describe('LoginPage', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    validate.mockResolvedValue(true)
    window.desktopApi = {
      window: {
        minimize: vi.fn(),
        maximize: vi.fn(),
        close: vi.fn(),
      },
    } as unknown as Window['desktopApi']
  })

  it('初始输入为空且不展示默认 IP、内置账号和角色归属标识', () => {
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm

    expect(component.form).toEqual({ ip: '', username: '', password: '' })
    expect(wrapper.text()).not.toContain('19.19.19.16')
    expect(wrapper.text()).not.toContain('admin@123')
    expect(wrapper.text()).not.toContain('操作员')
    expect(wrapper.text()).not.toContain('审计员')
    expect(wrapper.text()).not.toContain('系统管理员')
  })

  it('拒绝格式错误的 IPv4 地址', () => {
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm
    const validator = component.rules.ip.find((rule) => rule.validator != null)?.validator
    const callback = vi.fn()

    validator?.({}, '192.168.1.999', callback)

    expect(callback).toHaveBeenCalledWith(expect.objectContaining({ message: '装置 IP 格式错误' }))
  })

  it('登录业务错误保留装置返回文案', async () => {
    const session = useSessionStore()
    vi.spyOn(session, 'login').mockRejectedValue(
      new ServiceError('用户名或密码错误', 0x0101, 'business'),
    )
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm
    Object.assign(component.form, {
      ip: '19.19.19.16',
      username: 'admin',
      password: 'wrong-password',
    })

    await component.handleSubmit()

    expect(component.errorMessage).toBe('用户名或密码错误')
    expect(component.isSubmitting).toBe(false)
    expect(wrapper.text()).not.toContain('正在进行身份验证')
  })

  it('证书指纹不匹配时展示版本不兼容提示', async () => {
    const session = useSessionStore()
    vi.spyOn(session, 'login').mockRejectedValue(new Error('版本不兼容，请升级管理端'))
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm
    Object.assign(component.form, {
      ip: '19.19.19.16',
      username: 'admin',
      password: 'admin@123',
    })

    await component.handleSubmit()

    expect(component.errorMessage).toBe('版本不兼容，请升级管理端')
  })

  it('账号锁定时展示统一锁定文案', async () => {
    const session = useSessionStore()
    vi.spyOn(session, 'login').mockRejectedValue(
      new ServiceError('用户已被锁定，请5分钟后重试', 0x0004, 'business'),
    )
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm
    Object.assign(component.form, {
      ip: '19.19.19.16',
      username: 'admin',
      password: 'admin@123',
    })

    await component.handleSubmit()

    expect(component.errorMessage).toBe('用户已被锁定，请5分钟后重试')
  })

  it('提交过程中拒绝重复登录请求', async () => {
    const session = useSessionStore()
    const login = vi.spyOn(session, 'login').mockResolvedValue({
      success: true,
      resultCode: 0,
      errorMessage: '',
    })
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm
    Object.assign(component.form, {
      ip: '19.19.19.16',
      username: 'admin',
      password: 'admin@123',
    })

    await Promise.all([component.handleSubmit(), component.handleSubmit()])

    expect(login).toHaveBeenCalledTimes(1)
  })

  it.each(['unauthorized', 'expired'] as const)('%s 状态进入独立授权页', async (authStatus) => {
    const session = useSessionStore()
    vi.spyOn(session, 'login').mockImplementation(async () => {
      session.setSession({
        token: 'temporary-token',
        username: 'operator',
        role: 'operator',
        authStatus,
        authExpireTime: 0,
        deviceDescription: '',
      })
      return { success: true, resultCode: 0, errorMessage: '' }
    })
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm
    Object.assign(component.form, {
      ip: '19.19.19.16',
      username: 'operator',
      password: 'operator@123',
    })

    await component.handleSubmit()

    expect(push).toHaveBeenCalledWith('/license')
  })

  it('已授权用户按角色进入默认页面', async () => {
    const session = useSessionStore()
    const connection = useConnectionStore()
    vi.spyOn(session, 'login').mockImplementation(async () => {
      session.setSession({
        token: 'token',
        username: 'audit',
        role: 'auditor',
        authStatus: 'authorized',
        authExpireTime: 0,
        deviceDescription: '',
      })
      connection.updateStatus('CONNECTED')
      return { success: true, resultCode: 0, errorMessage: '' }
    })
    const wrapper = mountPage()
    const component = wrapper.vm as unknown as LoginPageVm
    Object.assign(component.form, {
      ip: '19.19.19.16',
      username: 'audit',
      password: 'audit@123',
    })

    await component.handleSubmit()

    expect(push).toHaveBeenCalledWith('/logs')
  })
})
