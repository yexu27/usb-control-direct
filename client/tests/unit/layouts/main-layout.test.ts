import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { shallowMount } from '@vue/test-utils'
import MainLayout from '../../../src/renderer/layouts/MainLayout.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import type { UserRole } from '../../../src/shared/connection-state'

const push = vi.fn()
const currentRoute = { path: '/file-access' }

vi.mock('vue-router', () => ({
  useRouter: () => ({ push }),
  useRoute: () => currentRoute,
}))

function setSession(role: UserRole): void {
  const session = useSessionStore()
  session.setSession({
    token: 'session-token',
    username: role === 'auditor' ? 'audit' : role,
    role,
    authStatus: 'authorized',
    authExpireTime: 0,
    deviceDescription: '',
  })
}

function mountLayout() {
  return shallowMount(MainLayout, {
    global: {
      stubs: {
        RouterView: true,
        ChangePasswordDialog: true,
        ElDropdown: true,
        ElDropdownMenu: true,
        ElDropdownItem: true,
        ElIcon: true,
      },
    },
  })
}

describe('MainLayout', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    push.mockReset()
    currentRoute.path = '/file-access'
  })

  it('展示品牌、系统名称、装置 IP 和当前用户', () => {
    setSession('operator')
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'

    const wrapper = mountLayout()

    expect(wrapper.get('[data-testid="brand-name"]').text()).toBe('安帝科技')
    expect(wrapper.get('[data-testid="brand-en-name"]').text()).toBe('ANDISEC')
    expect(wrapper.get('[data-testid="product-name"]').text()).toBe('USB安全管理系统')
    expect(wrapper.get('[data-testid="device-ip"]').text()).toContain('19.19.19.16')
    expect(wrapper.get('[data-testid="current-user"]').text()).toContain('operator')
  })

  it.each([
    ['operator', ['文件访问控制', 'U盘设备控制', '策略管理']],
    ['auditor', ['日志管理']],
    ['admin', ['系统管理', '用户管理']],
  ] as const)('%s 仅展示该角色可访问的菜单', (role, expectedMenus) => {
    setSession(role)

    const wrapper = mountLayout()
    const menuLabels = wrapper.findAll('[data-testid="nav-item"]').map((item) => item.text())

    expect(menuLabels).toEqual(expectedMenus)
  })

  it('展示当前连接状态', () => {
    setSession('operator')
    const connection = useConnectionStore()
    connection.updateStatus('CONNECTED')

    const wrapper = mountLayout()

    expect(wrapper.get('[data-testid="connection-status"]').text()).toBe('已连接')
    expect(wrapper.get('[data-testid="connection-dot"]').classes()).toContain('connected')
  })
})
