// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { ElMessageBox } from 'element-plus'
import UsersPage from '../../../src/renderer/pages/UsersPage.vue'
import {
  createUser,
  deleteUser,
  listUsers,
  resetPassword,
} from '../../../src/renderer/services/user-service'
import { useConnectionStore } from '../../../src/renderer/stores/connection'
import { useSessionStore } from '../../../src/renderer/stores/session'
import { showErrorDialog, showSuccessToast } from '../../../src/renderer/utils/operation-feedback'

vi.mock('../../../src/renderer/services/user-service', () => ({
  listUsers: vi.fn(),
  createUser: vi.fn(),
  deleteUser: vi.fn(),
  resetPassword: vi.fn(),
}))

vi.mock('element-plus', () => ({
  ElMessageBox: { confirm: vi.fn(() => Promise.resolve()) },
}))

vi.mock('../../../src/renderer/utils/operation-feedback', () => ({
  errorMessage: (error: unknown, fallback: string) => (error instanceof Error ? error.message : fallback),
  showErrorDialog: vi.fn(() => Promise.resolve()),
  showSuccessToast: vi.fn(),
}))

let pinia: Pinia

const DataTableStub = defineComponent({
  name: 'DataTable',
  props: ['columns', 'data', 'total', 'page', 'pageSize', 'loading', 'error'],
  setup(props, { slots }) {
    return () => h('section', { 'data-testid': 'users-table' }, [
      slots.filters?.(),
      h('div', { 'data-testid': 'columns' }, props.columns.map((column: { label: string }) => column.label).join('|')),
      props.data.map((row: { username: string }) =>
        h('div', { key: row.username, 'data-testid': `user-row-${row.username}` }, [
          h('span', row.username),
          slots.role?.({ row }),
          slots.status?.({ row }),
          slots.createdAt?.({ row }),
          slots.actions?.({ row }),
        ]),
      ),
    ])
  },
})

const ElInputStub = defineComponent({
  props: ['modelValue'],
  emits: ['update:modelValue'],
  template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)">',
})

const ElSelectStub = defineComponent({
  props: ['modelValue'],
  emits: ['update:modelValue'],
  template: '<select :value="modelValue" @change="$emit(\'update:modelValue\', $event.target.value)"><slot /></select>',
})

const ElOptionStub = defineComponent({
  props: ['label', 'value'],
  template: '<option :value="value">{{ label }}</option>',
})

const ElFormStub = defineComponent({
  setup(_props, { expose, slots }) {
    expose({
      validate: () => Promise.resolve(true),
      clearValidate: vi.fn(),
    })
    return () => h('form', slots.default?.())
  },
})

function mountPage() {
  return mount(UsersPage, {
    global: {
      plugins: [pinia],
      stubs: {
        ConnectionAlert: { template: '<aside />' },
        DataTable: DataTableStub,
        ElCard: { template: '<section><slot /></section>' },
        ElButton: { emits: ['click'], template: '<button type="button" @click="$emit(\'click\')"><slot /></button>' },
        ElDialog: { props: ['modelValue'], template: '<section v-if="modelValue"><slot /><slot name="footer" /></section>' },
        ElAlert: { props: ['title'], template: '<strong v-bind="$attrs">{{ title }}</strong>' },
        ElForm: ElFormStub,
        ElFormItem: { template: '<label><slot /></label>' },
        ElInput: ElInputStub,
        ElSelect: ElSelectStub,
        ElOption: ElOptionStub,
        ElTag: { template: '<span v-bind="$attrs"><slot /></span>' },
      },
    },
  })
}

function seedStores(): void {
  useSessionStore().setSession({
    token: 'token',
    username: 'admin',
    role: 'admin',
    authStatus: 'authorized',
    authExpireTime: 0,
    deviceDescription: '',
  })
  useConnectionStore().updateStatus('CONNECTED')
}

describe('UsersPage', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    seedStores()
    vi.clearAllMocks()
    vi.mocked(listUsers).mockResolvedValue({
      users: [
        { username: 'admin', role: 'admin', status: 'active', isBuiltin: true, createdAt: 0 },
        { username: 'zhang_wei', role: 'operator', status: 'locked', isBuiltin: false, createdAt: 1_767_225_610 },
      ],
    } as never)
  })

  it('loads users with role, status, builtin marker, and protected builtin delete action', async () => {
    const wrapper = mountPage()

    await flushPromises()

    expect(listUsers).toHaveBeenCalledWith('token')
    expect(wrapper.text()).toContain('用户管理')
    expect(wrapper.text()).toContain('admin')
    expect(wrapper.text()).toContain('zhang_wei')
    expect(wrapper.text()).toContain('内置')
    expect(wrapper.text()).toContain('锁定')
    expect(wrapper.findAll('.app-role-badge')).toHaveLength(2)
    expect(wrapper.find('.app-role-admin').exists()).toBe(true)
    expect(wrapper.find('.app-role-operator').exists()).toBe(true)
    expect(wrapper.find('[data-testid="delete-user-admin"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="delete-user-zhang_wei"]').exists()).toBe(true)
  })

  it('creates a user after validating username, password, and confirm password', async () => {
    vi.mocked(createUser).mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="create-user-open"]').trigger('click')
    await wrapper.get('[data-testid="create-username"]').setValue('new_operator')
    await wrapper.get('[data-testid="create-role"]').setValue('operator')
    await wrapper.get('[data-testid="create-password"]').setValue('NewPass@123')
    await wrapper.get('[data-testid="create-confirm-password"]').setValue('NewPass@123')
    await wrapper.get('[data-testid="create-user-submit"]').trigger('click')
    await flushPromises()

    expect(createUser).toHaveBeenCalledWith('token', 'new_operator', 'operator', 'NewPass@123', 'NewPass@123')
    expect(showSuccessToast).toHaveBeenCalledWith('用户创建成功')
    expect(listUsers).toHaveBeenCalledTimes(2)
  })

  it('keeps create dialog open and shows inline service error when creating user fails', async () => {
    vi.mocked(createUser).mockRejectedValue(new Error('用户名已存在'))
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="create-user-open"]').trigger('click')
    await wrapper.get('[data-testid="create-username"]').setValue('admin')
    await wrapper.get('[data-testid="create-role"]').setValue('operator')
    await wrapper.get('[data-testid="create-password"]').setValue('NewPass@123')
    await wrapper.get('[data-testid="create-confirm-password"]').setValue('NewPass@123')
    await wrapper.get('[data-testid="create-user-submit"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="create-user-error"]').text()).toContain('用户名已存在')
    expect(wrapper.find('[data-testid="create-user-submit"]').exists()).toBe(true)
    expect(showSuccessToast).not.toHaveBeenCalledWith('用户创建成功')
  })

  it('deletes non-builtin users only after confirmation', async () => {
    vi.mocked(deleteUser).mockResolvedValue(undefined)
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm' as never)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="delete-user-zhang_wei"]').trigger('click')
    await flushPromises()

    expect(ElMessageBox.confirm).toHaveBeenCalledWith(
      expect.stringContaining('zhang_wei'),
      expect.any(String),
      expect.any(Object),
    )
    expect(deleteUser).toHaveBeenCalledWith('token', 'zhang_wei')
    expect(showSuccessToast).toHaveBeenCalledWith('用户已删除')
    expect(listUsers).toHaveBeenCalledTimes(2)
  })

  it('shows centered error dialog when deleting user fails', async () => {
    vi.mocked(deleteUser).mockRejectedValue(new Error('用户不存在'))
    vi.mocked(ElMessageBox.confirm).mockResolvedValue('confirm' as never)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="delete-user-zhang_wei"]').trigger('click')
    await flushPromises()

    expect(showErrorDialog).toHaveBeenCalledWith('用户删除失败', '用户不存在')
  })

  it('resets password after validating complexity and matching confirmation', async () => {
    vi.mocked(resetPassword).mockResolvedValue(undefined)
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="reset-password-zhang_wei"]').trigger('click')
    await wrapper.get('[data-testid="reset-password"]').setValue('Reset@123')
    await wrapper.get('[data-testid="reset-confirm-password"]').setValue('Reset@123')
    await wrapper.get('[data-testid="reset-password-submit"]').trigger('click')
    await flushPromises()

    expect(resetPassword).toHaveBeenCalledWith('token', 'zhang_wei', 'Reset@123', 'Reset@123')
    expect(showSuccessToast).toHaveBeenCalledWith('密码已重置')
    expect(listUsers).toHaveBeenCalledTimes(2)
  })

  it('keeps reset dialog open and shows inline service error when reset fails', async () => {
    vi.mocked(resetPassword).mockRejectedValue(new Error('密码复杂度不符合要求'))
    const wrapper = mountPage()
    await flushPromises()

    await wrapper.get('[data-testid="reset-password-zhang_wei"]').trigger('click')
    await wrapper.get('[data-testid="reset-password"]').setValue('short')
    await wrapper.get('[data-testid="reset-confirm-password"]').setValue('short')
    await wrapper.get('[data-testid="reset-password-submit"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="reset-password-error"]').text()).toContain('密码复杂度不符合要求')
    expect(wrapper.find('[data-testid="reset-password-submit"]').exists()).toBe(true)
    expect(showSuccessToast).not.toHaveBeenCalledWith('密码已重置')
  })
})
