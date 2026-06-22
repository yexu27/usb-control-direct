import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { shallowMount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import MainLayout from '../../../src/renderer/layouts/MainLayout.vue'
import { useSessionStore } from '../../../src/renderer/stores/session'

const push = vi.fn()
const currentRoute = { path: '/file-access' }

const ElDropdownStub = defineComponent({
  name: 'ElDropdown',
  emits: ['command'],
  template: '<div><slot /><slot name="dropdown" /></div>',
})

vi.mock('vue-router', () => ({
  useRouter: () => ({ push }),
  useRoute: () => currentRoute,
}))

function mountLayout() {
  const session = useSessionStore()
  session.setSession({
    token: 'session-token',
    username: 'operator',
    role: 'operator',
    authStatus: 'authorized',
    authExpireTime: 0,
    deviceDescription: '',
  })

  return shallowMount(MainLayout, {
    global: {
      stubs: {
        RouterView: true,
        ChangePasswordDialog: true,
        ElDropdown: ElDropdownStub,
        ElDropdownMenu: true,
        ElDropdownItem: true,
        ElIcon: true,
      },
    },
  })
}

describe('MainLayout visual contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    push.mockReset()
    currentRoute.path = '/file-access'
  })

  it('uses the BrowserWindow content area as the app shell with internal content scrolling', () => {
    const wrapper = mountLayout()

    expect(wrapper.find('.app-desktop').exists()).toBe(true)
    expect(wrapper.find('.app-window-shell').exists()).toBe(true)
    expect(wrapper.find('.main-body').exists()).toBe(true)

    const content = wrapper.get('.main-content')
    expect(content.classes()).toContain('app-content-scroll')

    const shell = wrapper.get('.app-window-shell')
    const header = shell.get('.main-header')
    const body = shell.get('.main-body')
    const sidebar = body.get('.main-sidebar')

    expect(header.element.parentElement).toBe(shell.element)
    expect(body.element.parentElement).toBe(shell.element)
    expect(sidebar.element.parentElement).toBe(body.element)
    expect(content.element.parentElement).toBe(body.element)
  })

  it('moves connection status into the sidebar footer instead of a window-height footer', () => {
    const wrapper = mountLayout()

    expect(wrapper.find('.main-footer').exists()).toBe(false)
    expect(wrapper.find('.sidebar-footer [data-testid="connection-status"]').exists()).toBe(true)
  })
})
