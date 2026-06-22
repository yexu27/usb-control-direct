<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import DataTable from '@/components/DataTable.vue'
import ProgressDialog from '@/components/ProgressDialog.vue'
import {
  LOG_TABS,
  OPERATION_LOG_CATEGORY_OPTIONS,
  USB_EVENT_TYPE_OPTIONS,
  type LogType,
  formatOperationLogCategory,
  getLogColumns,
} from '@/utils/log-display'
import {
  dateToUnixSeconds,
  formatUnixSeconds,
  getDefaultLogRange,
  isBeforeRetentionBoundary,
} from '@/utils/date-time'
import { deleteLogs, exportLogs, queryLogs, type LogQueryInput } from '@/services/log-service'
import { ServiceError } from '@/services/send-command'
import { useConnectionStore } from '@/stores/connection'
import { useSessionStore } from '@/stores/session'
import type { usb_control } from '../../shared/proto/usb_control'

const PAGE_SIZE = 20
const DISCONNECTED_MESSAGE = 'USB 管控装置已断开连接，操作失败。'

interface LogRow {
  id: string
  time: string
  deviceName?: string
  serialNumber?: string
  eventType?: string
  content: string
  virus?: string
  username?: string
  operationType?: string
}

const session = useSessionStore()
const connection = useConnectionStore()
const activeLogType = ref<LogType>('usb_audit')
const dateRange = ref<[Date, Date]>(createDefaultRange())
const keyword = ref('')
const selectedEventType = ref('')
const selectedOperationCategory = ref('')
const page = ref(1)
const pageSize = ref(PAGE_SIZE)
const rows = ref<LogRow[]>([])
const total = ref(0)
const isLoading = ref(false)
const errorMessage = ref('')
const isExporting = ref(false)
const clearDialogVisible = ref(false)
const clearRange = ref<[Date, Date]>(createDefaultRange())

const activeTabLabel = computed(() => {
  return LOG_TABS.find((tab) => tab.value === activeLogType.value)?.label ?? ''
})
const columns = computed(() => getLogColumns(activeLogType.value))
const showUsbEventFilter = computed(() => activeLogType.value === 'usb_audit')
const showOperationFilter = computed(() => activeLogType.value === 'operation')

onMounted(() => {
  void loadLogs()
})

function createDefaultRange(): [Date, Date] {
  const range = getDefaultLogRange()
  return [range.start, range.end]
}

function showError(error: unknown, fallback: string): void {
  ElMessage.error(error instanceof ServiceError || error instanceof Error ? error.message : fallback)
}

function canOperate(): boolean {
  if (connection.isConnected) {
    return true
  }
  ElMessage.warning(DISCONNECTED_MESSAGE)
  return false
}

function buildQueryInput(): LogQueryInput {
  return {
    logType: activeLogType.value,
    startTime: dateToUnixSeconds(dateRange.value[0]),
    endTime: dateToUnixSeconds(dateRange.value[1]),
    keyword: keyword.value.trim(),
    eventType: activeLogType.value === 'usb_audit' ? selectedEventType.value : '',
    page: page.value,
    pageSize: pageSize.value,
    logCategory: activeLogType.value === 'operation' ? selectedOperationCategory.value : '',
    actionType: '',
  }
}

async function loadLogs(): Promise<void> {
  if (!canOperate()) {
    return
  }
  isLoading.value = true
  errorMessage.value = ''
  try {
    const response = await queryLogs(session.token, buildQueryInput())
    total.value = Number(response.total ?? 0)
    rows.value = mapResponseRows(response)
  } catch (error: unknown) {
    rows.value = []
    total.value = 0
    errorMessage.value = error instanceof Error ? error.message : '日志查询失败'
  } finally {
    isLoading.value = false
  }
}

function mapResponseRows(response: usb_control.RspQueryLogs): LogRow[] {
  if (activeLogType.value === 'usb_audit') {
    return response.usbAuditEntries.map(mapUsbAuditRow)
  }
  if (activeLogType.value === 'malware') {
    return response.malwareEntries.map(mapMalwareRow)
  }
  return response.operationEntries.map(mapOperationRow)
}

function mapUsbAuditRow(entry: usb_control.IUsbAuditLogEntry): LogRow {
  return {
    id: String(entry.id ?? ''),
    time: formatUnixSeconds(entry.eventTime ?? 0),
    deviceName: entry.deviceName ?? '',
    serialNumber: entry.deviceSn ?? '',
    eventType: formatUsbEventType(entry.eventType ?? ''),
    content: buildContent(entry.detail, entry.result, entry.failReason),
  }
}

function mapMalwareRow(entry: usb_control.IMalwareLogEntry): LogRow {
  return {
    id: String(entry.id ?? ''),
    time: formatUnixSeconds(entry.scanTime ?? 0),
    deviceName: entry.deviceName ?? '',
    serialNumber: entry.deviceSn ?? '',
    content: buildContent(entry.detail, entry.processResult, entry.failReason),
    virus: entry.virusName ?? '',
  }
}

function mapOperationRow(entry: usb_control.IOperationLogEntry): LogRow {
  return {
    id: String(entry.id ?? ''),
    time: formatUnixSeconds(entry.opTime ?? 0),
    username: entry.username ?? '',
    operationType: formatOperationLogCategory(entry.logCategory ?? ''),
    content: buildContent(entry.detail, entry.result, entry.failReason),
  }
}

function buildContent(
  detail: string | null | undefined,
  result: string | number | null | undefined,
  failReason: string | null | undefined,
): string {
  if (detail != null && detail.trim() !== '') {
    return detail
  }
  const parts = [`结果：${String(result ?? '-')}`]
  if (failReason != null && failReason.trim() !== '') {
    parts.push(`失败原因：${failReason}`)
  }
  return parts.join('，')
}

function formatUsbEventType(value: string): string {
  return USB_EVENT_TYPE_OPTIONS.find((item) => item.value === value)?.label ?? value
}

function handleTabChange(tabName: string | number): void {
  activeLogType.value = tabName as LogType
  page.value = 1
  selectedEventType.value = ''
  selectedOperationCategory.value = ''
  void loadLogs()
}

function handleSearch(): void {
  page.value = 1
  void loadLogs()
}

function handlePageChange(nextPage: number): void {
  page.value = nextPage
  void loadLogs()
}

function handlePageSizeChange(nextPageSize: number): void {
  pageSize.value = nextPageSize
  page.value = 1
  void loadLogs()
}

async function handleExport(): Promise<void> {
  if (isExporting.value || !canOperate()) {
    return
  }
  isExporting.value = true
  let selectedPath = ''
  let writeCompleted = false
  try {
    const response = await exportLogs(session.token, buildExportInput())
    const result = await window.desktopApi.dialog.saveFile({
      title: '导出日志',
      defaultPath: response.suggestedFilename,
      filters: [{ name: '日志压缩包', extensions: ['zip'] }],
    })
    if (result.canceled || result.filePath == null) {
      return
    }
    selectedPath = result.filePath
    await window.desktopApi.dialog.writeFile(selectedPath, response.zipData)
    writeCompleted = true
    ElMessage.success('日志导出成功')
  } catch (error: unknown) {
    if (selectedPath !== '' && !writeCompleted) {
      await revokeFileAccess(selectedPath)
    }
    showError(error, '日志导出失败')
  } finally {
    isExporting.value = false
  }
}

function buildExportInput(): Omit<LogQueryInput, 'page' | 'pageSize'> {
  const input = buildQueryInput()
  return {
    logType: input.logType,
    startTime: input.startTime,
    endTime: input.endTime,
    keyword: input.keyword,
    eventType: input.eventType,
    logCategory: input.logCategory,
    actionType: input.actionType,
  }
}

async function revokeFileAccess(filePath: string): Promise<void> {
  try {
    await window.desktopApi.dialog.revokeFileAccess(filePath)
  } catch {
    // 主错误优先展示，撤销失败不暴露本地路径。
  }
}

async function handleClear(): Promise<void> {
  if (!canOperate()) {
    return
  }
  clearRange.value = [new Date(dateRange.value[0]), new Date(dateRange.value[1])]
  clearDialogVisible.value = true
}

async function confirmClear(): Promise<void> {
  if (!canOperate()) {
    return
  }
  const startSeconds = dateToUnixSeconds(clearRange.value[0])
  const endSeconds = dateToUnixSeconds(clearRange.value[1])
  const startText = formatUnixSeconds(startSeconds)
  const endText = formatUnixSeconds(endSeconds)
  try {
    await ElMessageBox.confirm(
      `请确认是否清除 ${startText} - ${endText} 的${activeTabLabel.value}？`,
      '清理日志',
      {
        type: 'warning',
        confirmButtonText: '确认清理',
        cancelButtonText: '取消',
      },
    )
  } catch {
    return
  }
  try {
    await deleteLogs(session.token, activeLogType.value, startSeconds, endSeconds)
    clearDialogVisible.value = false
    ElMessage.success('日志已清理')
    await loadLogs()
  } catch (error: unknown) {
    showError(error, '日志清理失败')
  }
}

function disabledClearDate(date: Date): boolean {
  return !isBeforeRetentionBoundary(date)
}
</script>

<template>
  <div class="logs-page">
    <header class="page-header">
      <h1>日志管理</h1>
      <p>USB操作和安全事件的审计追踪</p>
    </header>
    <ConnectionAlert />

    <el-card shadow="never" class="logs-card">
      <el-tabs
        :model-value="activeLogType"
        data-testid="log-tabs"
        @tab-change="handleTabChange"
      >
        <el-tab-pane
          v-for="tab in LOG_TABS"
          :key="tab.value"
          :label="tab.label"
          :name="tab.value"
        />
      </el-tabs>

      <DataTable
        :columns="columns"
        :data="rows"
        :loading="isLoading"
        :error="errorMessage"
        :total="total"
        :page="page"
        :page-size="pageSize"
        empty-text="暂无日志"
        @page-change="handlePageChange"
        @page-size-change="handlePageSizeChange"
      >
        <template #filters>
          <div class="log-filter-bar">
            <el-input
              v-model="keyword"
              class="filter-keyword"
              placeholder="关键字"
              clearable
              data-testid="log-keyword"
              @keyup.enter="handleSearch"
            />
            <el-date-picker
              v-model="dateRange"
              class="filter-range"
              type="datetimerange"
              start-placeholder="开始时间"
              end-placeholder="结束时间"
              :clearable="false"
              data-testid="log-date-range"
            />
            <el-select
              v-if="showUsbEventFilter"
              v-model="selectedEventType"
              class="filter-select"
              data-testid="log-event-type"
            >
              <el-option
                v-for="option in USB_EVENT_TYPE_OPTIONS"
                :key="option.value"
                :label="option.label"
                :value="option.value"
              />
            </el-select>
            <el-select
              v-if="showOperationFilter"
              v-model="selectedOperationCategory"
              class="filter-select"
              data-testid="log-category"
            >
              <el-option
                v-for="option in OPERATION_LOG_CATEGORY_OPTIONS"
                :key="option.value"
                :label="option.label"
                :value="option.value"
              />
            </el-select>
            <div class="filter-actions">
              <el-button type="primary" data-testid="log-search" @click="handleSearch">
                搜索
              </el-button>
              <el-button data-testid="log-export" :loading="isExporting" @click="handleExport">
                导出 .zip
              </el-button>
              <el-button type="danger" data-testid="log-clear" @click="handleClear">
                清理
              </el-button>
            </div>
          </div>
        </template>
        <template #eventType="{ row }">
          <el-tag size="small">{{ row.eventType }}</el-tag>
        </template>
        <template #operationType="{ row }">
          <el-tag size="small" type="info">{{ row.operationType }}</el-tag>
        </template>
      </DataTable>
    </el-card>

    <el-dialog v-model="clearDialogVisible" title="清理日志" width="520px">
      <div class="clear-dialog-body">
        <p>当前日志类型：{{ activeTabLabel }}</p>
        <el-date-picker
          v-model="clearRange"
          class="clear-range"
          type="datetimerange"
          start-placeholder="开始时间"
          end-placeholder="结束时间"
          :clearable="false"
          :disabled-date="disabledClearDate"
          data-testid="log-clear-range"
        />
      </div>
      <template #footer>
        <el-button @click="clearDialogVisible = false">取消</el-button>
        <el-button type="danger" data-testid="log-clear-confirm" @click="confirmClear">
          确认清理
        </el-button>
      </template>
    </el-dialog>

    <ProgressDialog
      :visible="isExporting"
      title="导出日志"
      message="正在从装置导出日志..."
    />
  </div>
</template>

<style scoped lang="scss">
.logs-page {
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

.logs-card {
  border-color: $border-color;
}

.log-filter-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  width: 100%;
  gap: $spacing-3;
}

.filter-keyword {
  flex: 1 1 180px;
  min-width: 180px;
}

.filter-range {
  flex: 0 1 360px;
}

.filter-select {
  flex: 0 1 160px;
}

.filter-actions {
  display: flex;
  flex: 0 0 auto;
  margin-left: auto;
  gap: $spacing-3;
}

:deep(.table-pagination) {
  align-self: flex-end;
}

.clear-dialog-body {
  display: flex;
  flex-direction: column;
  gap: $spacing-4;

  p {
    margin: 0;
    color: $text-regular;
  }
}

.clear-range {
  width: 100%;
}

@media (max-width: 1120px) {
  .filter-actions {
    flex: 1 1 100%;
    justify-content: flex-end;
  }
}
</style>
