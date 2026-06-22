<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessage } from 'element-plus'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import DataTable from '@/components/DataTable.vue'
import AddBlacklistDialog from '@/components/file-policy/AddBlacklistDialog.vue'
import type { DataTableColumn } from '@/components/data-table'
import { useConnectionStore } from '@/stores/connection'
import { useFilePolicyStore, type FilePolicyKey } from '@/stores/file-policy'
import { useSessionStore } from '@/stores/session'
import { confirmAction } from '@/utils/confirm-action'

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

function showError(error: unknown, fallback: string): void {
  ElMessage.error(error instanceof Error ? error.message : fallback)
}

function canWrite(): boolean {
  if (connection.isConnected) {
    return true
  }
  ElMessage.warning('装置已断开连接，无法修改策略')
  return false
}

async function changeSwitch(key: FilePolicyKey, enabled: boolean): Promise<boolean> {
  if (!canWrite()) {
    return false
  }
  try {
    await filePolicy.setSwitch(session.token, key, enabled)
    ElMessage.success(SUCCESS_MESSAGE)
    return true
  } catch (error: unknown) {
    showError(error, '开关修改失败')
    return false
  }
}

async function handleAdd(value: BlacklistFormValue): Promise<void> {
  if (isAdding.value || !canWrite()) {
    return
  }
  isAdding.value = true
  try {
    await filePolicy.addExtension(session.token, value.extension, value.description)
    addDialogVisible.value = false
    ElMessage.success(SUCCESS_MESSAGE)
  } catch (error: unknown) {
    showError(error, '黑名单添加失败')
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
    if (!canWrite()) {
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
    if (!canWrite()) {
      return
    }
    try {
      await filePolicy.removeExtension(session.token, normalizedExtension)
      ElMessage.success(SUCCESS_MESSAGE)
    } catch (error: unknown) {
      showError(error, '黑名单删除失败')
    }
  } finally {
    removingExtensions.value.delete(normalizedExtension)
  }
}

function changeExecControl(): Promise<boolean> {
  return changeSwitch('exec_control', !(filePolicy.policy?.execControlEnabled ?? false))
}

function changeAutoReadControl(): Promise<boolean> {
  return changeSwitch('auto_read_control', !(filePolicy.policy?.autoReadControlEnabled ?? false))
}

function changeBlacklistControl(): Promise<boolean> {
  return changeSwitch(
    'file_type_blacklist_control',
    !(filePolicy.policy?.fileTypeBlacklistEnabled ?? false),
  )
}

function openAddDialog(): void {
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
        <p class="app-page-desc">管理 USB 介质的可执行程序、自动读取与文件类型访问策略。</p>
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
          <div class="card-heading">
            <div>
              <h2>可执行程序访问控制</h2>
              <p>禁止 USB 介质中的可执行程序被读取或运行。</p>
            </div>
            <el-switch
              data-testid="exec-control-switch"
              aria-label="可执行程序访问控制"
              :model-value="filePolicy.policy?.execControlEnabled ?? false"
              :disabled="isPending('exec_control')"
              :loading="isPending('exec_control')"
              :before-change="changeExecControl"
            />
          </div>
          <div class="type-list" aria-label="可执行文件类型">
            <span
              v-for="type in ['dll', 'exe', 'PE', 'ELF']"
              :key="type"
              data-testid="executable-type"
            >
              {{ type }}
            </span>
          </div>
        </el-card>
      </section>

      <section
        class="policy-section"
        data-testid="file-policy-section"
        data-policy="auto_read_control"
      >
        <el-card shadow="never" class="policy-card app-card" data-testid="file-policy-card">
          <div class="card-heading">
            <div>
              <h2>介质自动读取</h2>
              <p>控制 USB 介质插入后是否允许系统自动读取。</p>
            </div>
            <el-switch
              data-testid="auto-read-control-switch"
              aria-label="介质自动读取控制"
              :model-value="filePolicy.policy?.autoReadControlEnabled ?? false"
              :disabled="isPending('auto_read_control')"
              :loading="isPending('auto_read_control')"
              :before-change="changeAutoReadControl"
            />
          </div>
        </el-card>
      </section>

      <section
        class="policy-section"
        data-testid="file-policy-section"
        data-policy="file_type_blacklist_control"
      >
        <el-card shadow="never" class="policy-card app-card" data-testid="file-policy-card">
          <div class="card-heading">
            <div>
              <h2>文件类型黑名单</h2>
              <p>禁止访问指定后缀名的文件。</p>
            </div>
            <el-switch
              data-testid="blacklist-control-switch"
              aria-label="文件类型黑名单控制"
              :model-value="filePolicy.policy?.fileTypeBlacklistEnabled ?? false"
              :disabled="isPending('file_type_blacklist_control')"
              :loading="isPending('file_type_blacklist_control')"
              :before-change="changeBlacklistControl"
            />
          </div>
          <DataTable
            :columns="columns"
            :data="pageRows"
            :loading="filePolicy.isLoading"
            :error="filePolicy.errorMessage"
            :total="blacklist.length"
            :page="page"
            :page-size="pageSize"
            empty-text="暂无黑名单条目"
            @page-change="changePage"
            @page-size-change="changePageSize"
          >
            <template #filters>
              <el-button
                type="primary"
                data-testid="add-blacklist-trigger"
                :disabled="isAdding"
                @click="openAddDialog"
              >
                添加黑名单
              </el-button>
            </template>
            <template #actions="{ row }">
              <el-button
                link
                type="danger"
                :data-extension="row.extension"
                :disabled="isRemoving(row.extension)"
                :loading="isRemoving(row.extension)"
                @click="handleRemove(row.extension)"
              >
                删除
              </el-button>
            </template>
          </DataTable>
        </el-card>
      </section>
    </div>

    <AddBlacklistDialog
      v-model:visible="addDialogVisible"
      :submitting="isAdding"
      @submit="handleAdd"
    />
  </div>
</template>

<style scoped lang="scss">
.file-access-page {
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

.policy-list {
  display: flex;
  flex-direction: column;
  gap: $spacing-5;
}

.policy-section {
  width: 100%;
}

.policy-card {
  width: 100%;
  border-color: $border-color;
}

.card-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: $spacing-6;

  h2 {
    margin: 0;
    color: $text-primary;
    font-size: $font-size-md;
    font-weight: $font-weight-semibold;
  }

  p {
    margin: $spacing-1 0 0;
    color: $text-secondary;
  }
}

.type-list {
  display: flex;
  margin-top: $spacing-5;
  gap: $spacing-3;

  span {
    padding: $spacing-1 $spacing-3;
    color: $text-regular;
    background: $bg-sidebar;
    border: $border-width solid $border-color;
    border-radius: $border-radius;
  }
}

.data-table-wrapper {
  margin-top: $spacing-6;
}
</style>
