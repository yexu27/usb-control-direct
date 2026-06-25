import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, shallowMount } from '@vue/test-utils'
import { ElMessage } from 'element-plus'
import ConnectionAlert from '../../../src/renderer/components/ConnectionAlert.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'

const push = vi.hoisted(() => vi.fn())
const emitPageRefresh = vi.hoisted(() => vi.fn())

vi.mock('vue-router', () => ({
  useRouter: () => ({ push }),
}))

vi.mock('element-plus', () => ({
  ElMessage: { success: vi.fn(), error: vi.fn() },
}))

vi.mock('../../../src/renderer/services/page-refresh-events', () => ({
  emitPageRefresh,
}))

function mountAlert() {
  return mount(ConnectionAlert, {
    global: {
      stubs: {
        ElAlert: { template: '<section><slot /></section>' },
        ElButton: {
          props: ['loading', 'disabled'],
          template: '<button :disabled="disabled" data-testid="connection-reconnect" @click="$emit(\'click\')"><slot /></button>',
        },
      },
    },
  })
}

describe('ConnectionAlert', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('初始未连接时不显示断线提示', () => {
    const wrapper = shallowMount(ConnectionAlert, {
      global: { stubs: { ElAlert: true } },
    })

    expect(wrapper.find('el-alert-stub').exists()).toBe(false)
  })

  it('曾经连接成功后断线时显示提示', () => {
    const connection = useConnectionStore()
    connection.wasConnected = true
    connection.updateStatus('DISCONNECTED')
    const wrapper = shallowMount(ConnectionAlert, {
      global: { stubs: { ElAlert: true } },
    })

    expect(wrapper.get('el-alert-stub').attributes('title')).toBe(
      'USB 管控装置已断开连接，请检查网络或设备连接。',
    )
  })

  it('点击重新连接后恢复会话并提示成功', async () => {
    const connection = useConnectionStore()
    connection.wasConnected = true
    connection.updateStatus('DISCONNECTED')
    const reconnect = vi.spyOn(useSessionStore(), 'reconnectAndValidate').mockResolvedValue(true)
    const wrapper = mountAlert()

    await wrapper.get('[data-testid="connection-reconnect"]').trigger('click')
    await flushPromises()

    expect(reconnect).toHaveBeenCalledTimes(1)
    expect(emitPageRefresh).toHaveBeenCalledWith('reconnect')
    expect(ElMessage.success).toHaveBeenCalledWith('USB 管控装置重新连接成功')
    expect(push).not.toHaveBeenCalled()
  })

  it('会话失效时清理后跳转登录页', async () => {
    const connection = useConnectionStore()
    connection.wasConnected = true
    connection.updateStatus('DISCONNECTED')
    vi.spyOn(useSessionStore(), 'reconnectAndValidate').mockResolvedValue(false)
    const wrapper = mountAlert()

    await wrapper.get('[data-testid="connection-reconnect"]').trigger('click')
    await flushPromises()

    expect(push).toHaveBeenCalledWith('/login')
    expect(emitPageRefresh).not.toHaveBeenCalled()
  })

  it('重连后进入授权态时跳转授权页且不刷新业务页面', async () => {
    const connection = useConnectionStore()
    connection.wasConnected = true
    connection.updateStatus('DISCONNECTED')
    vi.spyOn(useSessionStore(), 'reconnectAndValidate').mockImplementation(async () => {
      connection.updateStatus('AUTH_REQUIRED')
      return true
    })
    const wrapper = mountAlert()

    await wrapper.get('[data-testid="connection-reconnect"]').trigger('click')
    await flushPromises()

    expect(push).toHaveBeenCalledWith('/license')
    expect(emitPageRefresh).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
  })

  it('建链失败时留在当前页并展示错误', async () => {
    const connection = useConnectionStore()
    connection.wasConnected = true
    connection.updateStatus('DISCONNECTED')
    vi.spyOn(useSessionStore(), 'reconnectAndValidate')
      .mockRejectedValue(new Error('USB 管控装置重新连接失败，请检查网络或设备连接。'))
    const wrapper = mountAlert()

    await wrapper.get('[data-testid="connection-reconnect"]').trigger('click')
    await flushPromises()

    expect(push).not.toHaveBeenCalled()
    expect(emitPageRefresh).not.toHaveBeenCalled()
    expect(ElMessage.error).toHaveBeenCalledWith('USB 管控装置重新连接失败，请检查网络或设备连接。')
  })

  it('重连进行中时防止重复点击', async () => {
    const connection = useConnectionStore()
    connection.wasConnected = true
    connection.updateStatus('DISCONNECTED')
    let resolveReconnect!: (value: boolean) => void
    const reconnect = vi.spyOn(useSessionStore(), 'reconnectAndValidate')
      .mockReturnValue(new Promise((resolve) => { resolveReconnect = resolve }))
    const wrapper = mountAlert()
    const button = wrapper.get('[data-testid="connection-reconnect"]')

    await button.trigger('click')
    await button.trigger('click')

    expect(reconnect).toHaveBeenCalledTimes(1)
    resolveReconnect(true)
    await flushPromises()
  })
})
