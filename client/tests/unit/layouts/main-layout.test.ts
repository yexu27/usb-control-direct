import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { shallowMount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'
import MainLayout from '../../../src/renderer/layouts/MainLayout.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import type { UserRole } from '../../../src/shared/connection-state'

const push = vi.fn()
const currentRoute = { path: '/file-access' }

const ElDropdownStub = defineComponent({
  name: 'ElDropdown',
  emits: ['command'],
  template: '<div><slot /><slot name="dropdown" /></div>',
})
const ElDropdownMenuStub = defineComponent({
  name: 'ElDropdownMenu',
  template: '<div><slot /></div>',
})
const ElDropdownItemStub = defineComponent({
  name: 'ElDropdownItem',
  props: ['command', 'divided'],
  template: '<button type="button" :data-command="command"><slot /></button>',
})
const ChangePasswordDialogStub = defineComponent({
  name: 'ChangePasswordDialog',
  props: ['visible'],
  template: '<div />',
})

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
        ChangePasswordDialog: ChangePasswordDialogStub,
        ElDropdown: ElDropdownStub,
        ElDropdownMenu: ElDropdownMenuStub,
        ElDropdownItem: ElDropdownItemStub,
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

  it('展示品牌、系统名称与当前登录用户名，不暴露装置 IP 或角色', () => {
    setSession('operator')
    const connection = useConnectionStore()
    connection.deviceIp = '19.19.19.16'

    const wrapper = mountLayout()

    expect(wrapper.get('[data-testid="brand-name"]').text()).toBe('安帝科技')
    expect(wrapper.get('[data-testid="brand-en-name"]').text()).toBe('ANDISEC')
    expect(wrapper.get('[data-testid="product-name"]').text()).toBe('USB安全管理系统')
    expect(wrapper.find('[data-testid="device-ip"]').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('19.19.19.16')
    expect(wrapper.text()).not.toContain('操作员')
    const trigger = wrapper.get('[data-testid="user-menu-trigger"]')
    expect(trigger.element.tagName).toBe('BUTTON')
    expect(trigger.attributes('aria-label')).toBe('用户菜单：operator')
    expect(trigger.get('[data-testid="current-username"]').text()).toBe('operator')
    expect(trigger.attributes('title')).toBeUndefined()
  })

  it('用户菜单保留修改密码与登出操作及 command 绑定', async () => {
    setSession('operator')
    const wrapper = mountLayout()

    expect(wrapper.text()).toContain('修改密码')
    expect(wrapper.text()).toContain('登出')
    expect(wrapper.findAllComponents(ElDropdownItemStub).map((item) => item.props('command')))
      .toEqual(['change-password', 'logout'])

    wrapper.getComponent(ElDropdownStub).vm.$emit('command', 'change-password')
    await nextTick()
    expect(wrapper.getComponent(ChangePasswordDialogStub).props('visible')).toBe(true)
  })

  it('用户菜单在同一头部控制容器中位于窗口按钮左侧', () => {
    setSession('operator')
    const wrapper = mountLayout()
    const controls = wrapper.get('[data-testid="header-controls"]')
    const userMenu = controls.get('[data-testid="user-menu-trigger"]').element
    const windowControls = controls.get('[data-testid="window-controls"]').element
    const minimize = controls.get('[data-testid="window-minimize"]').element
    const maximize = controls.get('[data-testid="window-maximize"]').element
    const close = controls.get('[data-testid="window-close"]').element

    expect(userMenu.compareDocumentPosition(windowControls) & Node.DOCUMENT_POSITION_FOLLOWING)
      .toBeTruthy()
    for (const button of [minimize, maximize, close]) {
      expect(userMenu.compareDocumentPosition(button) & Node.DOCUMENT_POSITION_FOLLOWING)
        .toBeTruthy()
    }
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
