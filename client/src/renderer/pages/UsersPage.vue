<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref } from 'vue'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import DataTable from '@/components/DataTable.vue'
import type { DataTableColumn } from '@/components/data-table'
import {
  createUser,
  deleteUser,
  listUsers,
  resetPassword,
} from '@/services/user-service'
import { useConnectionStore } from '@/stores/connection'
import { useSessionStore } from '@/stores/session'
import { validatePasswordComplexity } from '@/utils/password-validator'
import { confirmAction } from '@/utils/confirm-action'
import {
  USER_ROLE_OPTIONS,
  formatCreatedAt,
  formatUserRole,
  formatUserStatus,
  type RoleValue,
} from '@/utils/user-display'
import type { usb_control } from '../../shared/proto/usb_control'

interface UserRow {
  username: string
  role: string
  status: string
  isBuiltin: boolean
  createdAt: number | string | { toString(): string }
}

interface CreateUserForm {
  username: string
  role: RoleValue
  password: string
  confirmPassword: string
}

interface ResetPasswordForm {
  username: string
  password: string
  confirmPassword: string
}

const USERNAME_PATTERN = /^[A-Za-z0-9_]{1,32}$/
const DISCONNECTED_MESSAGE = 'USB 管控装置已断开连接，操作失败。'

const session = useSessionStore()
const connection = useConnectionStore()
const users = ref<UserRow[]>([])
const isLoading = ref(false)
const errorMessage = ref('')
const createDialogVisible = ref(false)
const resetDialogVisible = ref(false)
const createSubmitting = ref(false)
const resetSubmitting = ref(false)
const createFormRef = ref<FormInstance | null>(null)
const resetFormRef = ref<FormInstance | null>(null)

const createForm = reactive<CreateUserForm>({
  username: '',
  role: 'operator',
  password: '',
  confirmPassword: '',
})

const resetForm = reactive<ResetPasswordForm>({
  username: '',
  password: '',
  confirmPassword: '',
})

const createRules: FormRules<CreateUserForm> = {
  username: [{
    validator: (_rule, value: string, callback) => {
      const message = validateUsername(String(value ?? '').trim())
      callback(message === '' ? undefined : new Error(message))
    },
    trigger: ['blur', 'change'],
  }],
  password: [{
    validator: (_rule, value: string, callback) => {
      const result = validatePasswordComplexity(String(value ?? ''))
      callback(result.valid ? undefined : new Error(result.message))
    },
    trigger: ['blur', 'change'],
  }],
  confirmPassword: [{
    validator: (_rule, value: string, callback) => {
      callback(value === createForm.password ? undefined : new Error('两次输入的密码不一致，请再次确认'))
    },
    trigger: ['blur', 'change'],
  }],
}

const resetRules: FormRules<ResetPasswordForm> = {
  password: [{
    validator: (_rule, value: string, callback) => {
      const result = validatePasswordComplexity(String(value ?? ''))
      callback(result.valid ? undefined : new Error(result.message))
    },
    trigger: ['blur', 'change'],
  }],
  confirmPassword: [{
    validator: (_rule, value: string, callback) => {
      callback(value === resetForm.password ? undefined : new Error('两次输入的密码不一致，请再次确认'))
    },
    trigger: ['blur', 'change'],
  }],
}

const columns = computed<DataTableColumn[]>(() => [
  { prop: 'username', label: '用户名', minWidth: 160 },
  { prop: 'role', label: '角色', minWidth: 140, slot: 'role' },
  { prop: 'status', label: '状态', minWidth: 120, slot: 'status' },
  { prop: 'createdAt', label: '创建时间', minWidth: 150, slot: 'createdAt' },
  { prop: 'actions', label: '操作', minWidth: 180, slot: 'actions', fixed: 'right' },
])

onMounted(() => {
  void loadUsers()
})

function canOperate(): boolean {
  if (connection.isConnected) {
    return true
  }
  ElMessage.warning(DISCONNECTED_MESSAGE)
  return false
}

function showError(error: unknown, fallback: string): void {
  ElMessage.error(error instanceof Error ? error.message : fallback)
}

function mapUserItem(item: usb_control.IUserItem): UserRow {
  return {
    username: item.username ?? '',
    role: item.role ?? '',
    status: item.status ?? '',
    isBuiltin: item.isBuiltin ?? false,
    createdAt: item.createdAt ?? 0,
  }
}

async function loadUsers(): Promise<void> {
  if (!canOperate()) {
    return
  }
  isLoading.value = true
  errorMessage.value = ''
  try {
    const response = await listUsers(session.token)
    users.value = response.users.map(mapUserItem)
  } catch (error: unknown) {
    users.value = []
    errorMessage.value = error instanceof Error ? error.message : '用户列表加载失败'
  } finally {
    isLoading.value = false
  }
}

function resetCreateForm(): void {
  createForm.username = ''
  createForm.role = 'operator'
  createForm.password = ''
  createForm.confirmPassword = ''
  createFormRef.value?.clearValidate()
}

function openCreateDialog(): void {
  resetCreateForm()
  createDialogVisible.value = true
  void nextTick(() => createFormRef.value?.clearValidate())
}

function openResetDialog(username: string): void {
  resetForm.username = username
  resetForm.password = ''
  resetForm.confirmPassword = ''
  resetDialogVisible.value = true
  void nextTick(() => resetFormRef.value?.clearValidate())
}

function validateUsername(username: string): string {
  if (!USERNAME_PATTERN.test(username)) {
    return '用户名仅允许字母、数字、下划线，最长32位'
  }
  return ''
}

async function submitCreateUser(): Promise<void> {
  if (createFormRef.value == null || createSubmitting.value || !canOperate()) {
    return
  }
  const valid = await createFormRef.value.validate().catch(() => false)
  if (!valid) {
    return
  }
  const username = createForm.username.trim()
  createSubmitting.value = true
  try {
    await createUser(
      session.token,
      username,
      createForm.role,
      createForm.password,
      createForm.confirmPassword,
    )
    ElMessage.success('用户创建成功')
    createDialogVisible.value = false
    await loadUsers()
  } catch (error: unknown) {
    showError(error, '用户创建失败')
  } finally {
    createSubmitting.value = false
  }
}

async function handleDeleteUser(username: string): Promise<void> {
  if (!canOperate()) {
    return
  }
  try {
    await confirmAction({
      message: `是否要删除 ${username} 用户？`,
      title: '确认删除用户',
      confirmButtonText: '删除',
      type: 'warning',
    })
  } catch {
    return
  }
  try {
    await deleteUser(session.token, username)
    ElMessage.success('用户已删除')
    await loadUsers()
  } catch (error: unknown) {
    showError(error, '用户删除失败')
  }
}

async function submitResetPassword(): Promise<void> {
  if (resetFormRef.value == null || resetSubmitting.value || !canOperate()) {
    return
  }
  const valid = await resetFormRef.value.validate().catch(() => false)
  if (!valid) {
    return
  }
  resetSubmitting.value = true
  try {
    await resetPassword(
      session.token,
      resetForm.username,
      resetForm.password,
      resetForm.confirmPassword,
    )
    ElMessage.success('密码已重置')
    resetDialogVisible.value = false
    await loadUsers()
  } catch (error: unknown) {
    showError(error, '密码重置失败')
  } finally {
    resetSubmitting.value = false
  }
}
</script>

<template>
  <div class="users-page app-page">
    <header class="page-header app-page-header">
      <div>
        <h1 class="app-page-title">用户管理</h1>
        <p class="app-page-desc">管理系统管理员、操作员和审计员账号</p>
      </div>
    </header>
    <ConnectionAlert />

    <el-card shadow="never" class="users-card app-card">
      <DataTable
        :columns="columns"
        :data="users"
        :loading="isLoading"
        :error="errorMessage"
        :total="users.length"
        :page="1"
        :page-size="20"
        empty-text="暂无用户"
      >
        <template #filters>
          <div class="user-toolbar app-filter-actions">
            <el-button type="primary" data-testid="create-user-open" @click="openCreateDialog">
              新建用户
            </el-button>
          </div>
        </template>
        <template #role="{ row }">
          <el-tag
            class="app-role-badge"
            :class="`app-role-${row.role}`"
            size="small"
          >
            {{ formatUserRole(row.role) }}
          </el-tag>
        </template>
        <template #status="{ row }">
          <el-tag
            class="app-status-badge"
            :class="row.status === 'locked' ? 'app-status-locked' : 'app-status-active'"
            :type="row.status === 'locked' ? 'danger' : 'success'"
            size="small"
          >
            {{ formatUserStatus(row.status) }}
          </el-tag>
        </template>
        <template #createdAt="{ row }">
          {{ formatCreatedAt(row.createdAt) }}
        </template>
        <template #actions="{ row }">
          <div class="row-actions app-row-actions">
            <el-button
              :data-testid="`reset-password-${row.username}`"
              plain
              @click="openResetDialog(row.username)"
            >
              重置密码
            </el-button>
            <el-button
              v-if="!row.isBuiltin"
              :data-testid="`delete-user-${row.username}`"
              type="danger"
              plain
              @click="handleDeleteUser(row.username)"
            >
              删除
            </el-button>
          </div>
        </template>
      </DataTable>
    </el-card>

    <el-dialog v-model="createDialogVisible" title="新建用户" width="520px">
      <el-form
        ref="createFormRef"
        :model="createForm"
        :rules="createRules"
        label-position="top"
      >
        <el-form-item label="用户名" prop="username">
          <el-input v-model="createForm.username" data-testid="create-username" maxlength="32" />
        </el-form-item>
        <el-form-item label="角色" prop="role">
          <el-select v-model="createForm.role" data-testid="create-role">
            <el-option
              v-for="option in USER_ROLE_OPTIONS"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="密码" prop="password">
          <el-input
            v-model="createForm.password"
            data-testid="create-password"
            type="password"
            show-password
          />
        </el-form-item>
        <el-form-item label="确认密码" prop="confirmPassword">
          <el-input
            v-model="createForm.confirmPassword"
            data-testid="create-confirm-password"
            type="password"
            show-password
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createDialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          data-testid="create-user-submit"
          :loading="createSubmitting"
          @click="submitCreateUser"
        >
          创建
        </el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="resetDialogVisible" title="重置密码" width="520px">
      <el-form
        ref="resetFormRef"
        :model="resetForm"
        :rules="resetRules"
        label-position="top"
      >
        <el-form-item label="用户名">
          <el-input :model-value="resetForm.username" readonly />
        </el-form-item>
        <el-form-item label="新密码" prop="password">
          <el-input
            v-model="resetForm.password"
            data-testid="reset-password"
            type="password"
            show-password
          />
        </el-form-item>
        <el-form-item label="确认密码" prop="confirmPassword">
          <el-input
            v-model="resetForm.confirmPassword"
            data-testid="reset-confirm-password"
            type="password"
            show-password
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="resetDialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          data-testid="reset-password-submit"
          :loading="resetSubmitting"
          @click="submitResetPassword"
        >
          保存
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.users-page {
  display: flex;
  flex-direction: column;
  gap: $spacing-5;
}

.page-header {
  h1 {
    margin: 0;
    color: $text-primary;
    font-size: $font-xxl;
    font-weight: $font-weight-semibold;
  }

  p {
    margin: $spacing-1 0 0;
    color: $text-secondary;
  }
}

.users-card {
  border-color: $border-color;
}

.user-toolbar {
  display: flex;
  justify-content: flex-end;
  width: 100%;
}

.row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: $spacing-2;
}
</style>
