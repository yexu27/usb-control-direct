<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import DataTable from '@/components/DataTable.vue'
import AddBlacklistDialog from '@/components/file-policy/AddBlacklistDialog.vue'
import type { DataTableColumn } from '@/components/data-table'
import { useConnectionStore } from '@/stores/connection'
import { useFilePolicyStore, type FilePolicyKey } from '@/stores/file-policy'
import { useSessionStore } from '@/stores/session'
import { confirmAction } from '@/utils/confirm-action'
import { errorMessage, showErrorDialog, showSuccessToast } from '@/utils/operation-feedback'

const SUCCESS_MESSAGE = '修改成功，重新拔插或重新映射后生效'
const PAGE_SIZE = 20

interface BlacklistFormValue {
  extension: string
  description: string
}

const session = useSessionStore()
const connection = useConnectionStore()
const filePolicy = useFilePolicyStore()
const addDialogVisible = ref(false)
const isAdding = ref(false)
const addErrorMessage = ref('')
const page = ref(1)
const pageSize = ref(PAGE_SIZE)
const removingExtensions = ref(new Set<string>())

const columns: DataTableColumn[] = [
  { prop: 'extension', label: '后缀名', width: 180 },
  { prop: 'description', label: '说明', minWidth: 240 },
  { prop: 'actions', label: '操作', width: 100, slot: 'actions' },
]

const blacklist = computed(() => filePolicy.policy?.blacklist ?? [])
const pageRows = computed(() => {
  const start = (page.value - 1) * pageSize.value
  return blacklist.value.slice(start, start + pageSize.value)
})

onMounted(() => {
  void refreshPolicy()
})

function isPending(key: string): boolean {
  return filePolicy.pendingKeys.has(key)
}

function normalizeExtension(extension: string): string {
  return extension.trim().toLowerCase()
}

function isRemoving(extension: string): boolean {
  const normalizedExtension = normalizeExtension(extension)
  return (
    removingExtensions.value.has(normalizedExtension) ||
    isPending(`extension:${normalizedExtension}`)
  )
}

async function showError(error: unknown, fallback: string): Promise<void> {
  await showErrorDialog(fallback, errorMessage(error, fallback))
}

async function refreshPolicy(): Promise<void> {
  if (!connection.isConnected) {
    return
  }
  try {
    await filePolicy.load(session.token)
  } catch {
    // Store errorMessage is rendered by DataTable; no modal on passive page refresh.
  }
}

async function canWrite(): Promise<boolean> {
  if (connection.isConnected) {
    return true
  }
  await showErrorDialog('操作失败', '装置已断开连接，无法修改策略')
  return false
}

async function changeSwitch(key: FilePolicyKey, enabled: boolean): Promise<boolean> {
  if (!(await canWrite())) {
    return false
  }
  try {
    await filePolicy.setSwitch(session.token, key, enabled)
    return true
  } catch (error: unknown) {
    await showError(error, '开关修改失败')
    return false
  }
}

async function handleAdd(value: BlacklistFormValue): Promise<void> {
  if (isAdding.value) {
    return
  }
  isAdding.value = true
  if (!(await canWrite())) {
    isAdding.value = false
    return
  }
  addErrorMessage.value = ''
  try {
    await filePolicy.addExtension(session.token, value.extension, value.description)
    addDialogVisible.value = false
    showSuccessToast(SUCCESS_MESSAGE)
  } catch (error: unknown) {
    addErrorMessage.value = errorMessage(error, '黑名单添加失败')
  } finally {
    isAdding.value = false
  }
}

async function handleRemove(extension: string): Promise<void> {
  const normalizedExtension = normalizeExtension(extension)
  if (removingExtensions.value.has(normalizedExtension)) {
    return
  }
  removingExtensions.value.add(normalizedExtension)
  try {
    if (!(await canWrite())) {
      return
    }
    try {
      await confirmAction({
        message: `确定删除黑名单条目 ${normalizedExtension} 吗？`,
        title: '删除确认',
        confirmButtonText: '删除',
        type: 'warning',
      })
    } catch {
      return
    }
    if (!(await canWrite())) {
      return
    }
    try {
      await filePolicy.removeExtension(session.token, normalizedExtension)
      showSuccessToast(SUCCESS_MESSAGE)
    } catch (error: unknown) {
      await showError(error, '黑名单删除失败')
    }
  } finally {
    removingExtensions.value.delete(normalizedExtension)
  }
}

function handlePolicyCheckboxChange(key: FilePolicyKey, enabled: boolean): void {
  void changeSwitch(key, enabled)
}

function openAddDialog(): void {
  addErrorMessage.value = ''
  addDialogVisible.value = true
}

function changePage(nextPage: number): void {
  page.value = nextPage
}

function changePageSize(nextPageSize: number): void {
  pageSize.value = nextPageSize
  page.value = 1
}
</script>

<template>
  <div class="file-access-page app-page">
    <header class="page-header app-page-header">
      <div>
        <h1 class="app-page-title">文件访问控制</h1>
        <p class="app-page-desc">管理移动存储设备的文件访问策略</p>
      </div>
    </header>
    <ConnectionAlert />

    <div class="policy-list" data-testid="file-policy-list">
      <section
        class="policy-section"
        data-testid="file-policy-section"
        data-policy="exec_control"
      >
        <el-card shadow="never" class="policy-card app-card" data-testid="file-policy-card">
          <label class="app-checkbox-row">
            <el-checkbox
              data-testid="exec-control-switch"
              aria-label="可执行程序访问控制"
              :model-value="filePolicy.policy?.execControlEnabled ?? false"
              :disabled="isPending('exec_control')"
              @change="(enabled) => handlePolicyCheckboxChange('exec_control', enabled === true)"
            />
            <span class="app-checkbox-copy">
              <span class="app-checkbox-title">可执行程序访问控制</span>
              <span class="app-checkbox-desc">禁止访问移动存储设备中的可执行文件</span>
              <span class="policy-detail">
                可执行程序指对以下程序进行控制：dll、exe、PE、ELF。默认：未勾选（允许访问）。
              </span>
              <span class="policy-detail">
                ⓘ 系统内置4种可执行程序类型，不可删除，只能通过上方复选框控制启用/禁用。
              </span>
            </span>
          </label>
        </el-card>
      </section>

      <section
        class="policy-section"
        data-testid="file-policy-section"
        data-policy="auto_read_control"
      >
        <el-card shadow="never" class="policy-card app-card" data-testid="file-policy-card">
          <label class="app-checkbox-row">
            <el-checkbox
              data-testid="auto-read-control-switch"
              aria-label="介质自动读取控制"
              :model-value="filePolicy.policy?.autoReadControlEnabled ?? false"
              :disabled="isPending('auto_read_control')"
              @change="(enabled) => handlePolicyCheckboxChange('auto_read_control', enabled === true)"
            />
            <span class="app-checkbox-copy">
              <span class="app-checkbox-title">介质自动读取功能控制</span>
              <span class="app-checkbox-desc">启用介质自动读取功能控制</span>
              <span class="policy-detail">
                介质自动读取指对U盘中 autorun 配置文件进行控制。启用后含 .sh 文件的U盘映射后，.sh文件内容显示为空（不可读）；禁用后 .sh 文件可正常读写。仅对U盘根目录下的 .sh 后缀文件生效。默认：未勾选（禁用）。
              </span>
            </span>
          </label>
        </el-card>
      </section>

      <section
        class="policy-section"
        data-testid="file-policy-section"
        data-policy="file_type_blacklist_control"
      >
        <el-card shadow="never" class="policy-card app-card" data-testid="file-policy-card">
          <label class="app-checkbox-row">
            <el-checkbox
              data-testid="blacklist-control-switch"
              aria-label="文件类型黑名单控制"
              :model-value="filePolicy.policy?.fileTypeBlacklistEnabled ?? false"
              :disabled="isPending('file_type_blacklist_control')"
              @change="(enabled) => handlePolicyCheckboxChange('file_type_blacklist_control', enabled === true)"
            />
            <span class="app-checkbox-copy file-policy-copy">
              <span class="app-checkbox-title">文件类型访问控制</span>
              <span class="app-checkbox-desc">启用文件类型访问控制</span>
              <span class="policy-detail">
                通过后缀类型管理移动存储设备中的文件。启用并添加文件类型黑名单后，移动存储设备完成映射后禁止访问黑名单中对应后缀类型的文件。默认：未勾选。
              </span>
            </span>
          </label>
          <div
            class="blacklist-panel prototype-table-shell"
            data-testid="blacklist-table-shell"
          >
            <div class="blacklist-panel-header">
              <span>文件类型黑名单</span>
              <el-button
                type="primary"
                data-testid="add-blacklist-trigger"
                :disabled="isAdding"
                @click="openAddDialog"
              >
                + 添加
              </el-button>
            </div>
            <DataTable
              class="blacklist-table prototype-table"
              :columns="columns"
              :data="pageRows"
              :loading="filePolicy.isLoading"
              :error="filePolicy.errorMessage"
              :total="blacklist.length"
              :page="page"
              :page-size="pageSize"
              empty-text="暂无黑名单条目"
              :show-default-pagination="false"
              @page-change="changePage"
              @page-size-change="changePageSize"
            >
              <template #actions="{ row }">
                <el-button
                  class="prototype-outline-action prototype-outline-danger"
                  :data-extension="row.extension"
                  :disabled="isRemoving(row.extension)"
                  :loading="isRemoving(row.extension)"
                  @click="handleRemove(row.extension)"
                >
                  删除
                </el-button>
              </template>
            </DataTable>
          </div>
        </el-card>
      </section>
    </div>
    <div class="policy-bottom-note" data-testid="file-policy-bottom-note">
      ⓘ 勾选后立即启用，取消勾选立即禁用；添加或删除黑名单后，需重新拔插或重新映射后生效。
    </div>

    <AddBlacklistDialog
      v-model:visible="addDialogVisible"
      :submitting="isAdding"
      :error-message="addErrorMessage"
      @submit="handleAdd"
    />
  </div>
</template>

<style scoped lang="scss">
@use '@/styles/tokens' as *;

.file-access-page {
  display: flex;
  flex-direction: column;
  gap: $spacing-5;
}

.page-header {
  h1 {
    margin: 0;
    color: var(--andi-text);
    font-size: $font-size-page-title-large;
    font-weight: $font-weight-semibold;
  }

  p {
    margin: $spacing-1 0 0;
    color: var(--andi-text-light);
  }
}

.policy-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.policy-section {
  width: 100%;
}

.policy-card {
  width: 100%;
  border-color: var(--andi-border);
}

.policy-card :deep(.el-card__body) {
  padding: 14px 18px;
}

.app-checkbox-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.policy-detail {
  color: var(--andi-text-light);
  font-size: 12px;
  font-weight: $font-weight-medium;
  line-height: 1.38;
}

.blacklist-panel {
  margin: 12px 0 0;
  overflow: hidden;
  background: var(--andi-white);
  border: $border-width-base solid var(--andi-border);
  border-radius: 6px;
}

.blacklist-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 12px;
  color: var(--andi-text);
  font-weight: $font-weight-semibold;
  background: var(--andi-sidebar);
  border-bottom: $border-width-base solid var(--andi-border);
}

.blacklist-table :deep(.data-table-wrapper) {
  gap: 0;
}

.policy-bottom-note {
  display: flex;
  align-items: center;
  margin-top: 14px;
  color: var(--andi-text-light);
  font-size: 13px;
  font-weight: $font-weight-medium;
  gap: 6px;
}
</style>
