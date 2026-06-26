<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessage } from 'element-plus'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import DataTable from '@/components/DataTable.vue'
import ProgressDialog from '@/components/ProgressDialog.vue'
import { useConnectedOperationGuard } from '@/composables/use-connected-operation-guard'
import { useDeviceBackedPageRefresh } from '@/composables/use-device-backed-page-refresh'
import {
  LOG_TABS,
  USB_EVENT_TYPE_OPTIONS,
  formatOperationLogCategory,
  type LogType,
  getLogColumns,
} from '@/utils/log-display'
import {
  dateToUnixSeconds,
  formatUnixSeconds,
  getDefaultLogRange,
  isBeforeRetentionBoundary,
} from '@/utils/date-time'
import { deleteLogs, exportLogs, queryLogs, type LogExportInput, type LogQueryInput } from '@/services/log-service'
import { ServiceError } from '@/services/send-command'
import { useSessionStore } from '@/stores/session'
import { confirmAction } from '@/utils/confirm-action'
import { showErrorDialog } from '@/utils/operation-feedback'
import type { usb_control } from '../../shared/proto/usb_control'

const PAGE_SIZE = 20
const DISCONNECTED_MESSAGE = 'USB 管控装置已断开连接，操作失败。'

interface LogRow {
  id: string
  time: string
  deviceName?: string
  serialNumber?: string
  eventType?: string
  logCategory?: string
  content: string
  virus?: string
  username?: string
}

const session = useSessionStore()
const connectedOperationGuard = useConnectedOperationGuard()
const { isBusinessActionDisabled, canReadFromDevice } = connectedOperationGuard
const activeLogType = ref<LogType>('usb_audit')
const dateRange = ref<[Date, Date]>(createDefaultRange())
const keyword = ref('')
const selectedEventType = ref('')
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
const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize.value)))
const startTime = computed({
  get: () => dateRange.value[0],
  set: (value: Date | null) => {
    if (!(value instanceof Date)) {
      return
    }
    dateRange.value = [value, dateRange.value[1]]
  },
})
const endTime = computed({
  get: () => dateRange.value[1],
  set: (value: Date | null) => {
    if (!(value instanceof Date)) {
      return
    }
    dateRange.value = [dateRange.value[0], value]
  },
})
const rangeStart = computed(() => {
  if (total.value === 0) {
    return 0
  }
  return (page.value - 1) * pageSize.value + 1
})
const rangeEnd = computed(() => Math.min(total.value, page.value * pageSize.value))
const visiblePages = computed(() => {
  if (totalPages.value <= 3) {
    return Array.from({ length: totalPages.value }, (_, index) => index + 1)
  }
  const start = Math.min(Math.max(1, page.value - 1), totalPages.value - 2)
  return Array.from({ length: 3 }, (_, index) => start + index)
})
const showPaginationEllipsis = computed(() => visiblePages.value.at(-1) !== totalPages.value)

useDeviceBackedPageRefresh(loadLogs)

function createDefaultRange(): [Date, Date] {
  const range = getDefaultLogRange()
  return [range.start, range.end]
}

function showError(error: unknown, fallback: string): void {
  ElMessage.error(error instanceof ServiceError || error instanceof Error ? error.message : fallback)
}

async function canWriteToDevice(): Promise<boolean> {
  if (canReadFromDevice()) {
    return true
  }
  await showErrorDialog('操作失败', DISCONNECTED_MESSAGE)
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
  }
}

async function loadLogs(): Promise<void> {
  if (!canReadFromDevice()) {
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
    content: buildUsbAuditContent(entry),
  }
}

type UsbAuditDeviceKind = 'storage' | 'keyboard' | 'mouse' | 'unsupported'

interface LongLike {
  toNumber?: () => number
}

function buildUsbAuditContent(entry: usb_control.IUsbAuditLogEntry): string {
  const deviceKind = getUsbAuditDeviceKind(entry)
  const action = getUsbAuditActionText(entry, deviceKind)

  if (deviceKind === 'storage') {
    const parts = [getStorageDeviceCategory(entry)]
    const permission = formatUsbAuditPermission(entry.permission)
    const capacity = formatUsbAuditCapacity(entry.capacityBytes)

    if (entry.eventType === 'insert_success') {
      if (permission !== '') {
        parts.push(permission)
      }
      if (capacity !== '') {
        parts.push(capacity)
      }
      parts.push(action)
      return parts.join(', ')
    }

    if (entry.eventType === 'device_remove') {
      if (permission !== '') {
        parts.push(permission)
      }
      return parts.join(', ')
    }

    parts.push(action)
    return parts.join(', ')
  }

  if (deviceKind === 'keyboard') {
    return ['键盘', action].filter((item) => item !== '').join(', ')
  }

  if (deviceKind === 'mouse') {
    return ['鼠标', action].filter((item) => item !== '').join(', ')
  }

  return ['不支持的 USB 设备类型', action].filter((item) => item !== '').join(', ')
}

function getUsbAuditDeviceKind(entry: usb_control.IUsbAuditLogEntry): UsbAuditDeviceKind {
  const deviceType = String(entry.deviceType ?? '').toLowerCase()
  const interfaceType = String(entry.interfaceType ?? '').toLowerCase()

  if (deviceType === 'storage' || interfaceType === 'mass_storage') {
    return 'storage'
  }
  if (deviceType === 'keyboard' || interfaceType === 'hid_keyboard') {
    return 'keyboard'
  }
  if (deviceType === 'mouse' || interfaceType === 'hid_mouse') {
    return 'mouse'
  }
  return 'unsupported'
}

function getStorageDeviceCategory(entry: usb_control.IUsbAuditLogEntry): string {
  const result = String(entry.result ?? '').toLowerCase()
  const failReason = `${entry.failReason ?? ''}${entry.detail ?? ''}`

  if (
    result === 'denied' ||
    result === 'blocked' ||
    failReason.includes('未授权') ||
    failReason.includes('不在白名单')
  ) {
    return '未授权设备'
  }

  return '授权设备'
}

function formatUsbAuditPermission(permission: string | null | undefined): string {
  const value = String(permission ?? '').toLowerCase()
  if (value === 'readwrite' || value === 'rw' || value === '1') {
    return '读写'
  }
  if (value === 'readonly' || value === 'read_only' || value === 'ro' || value === '0') {
    return '只读'
  }
  return ''
}

function formatUsbAuditCapacity(value: number | LongLike | null | undefined): string {
  const bytes = toSafeNumber(value)
  if (bytes <= 0) {
    return ''
  }
  const gb = bytes / 1_000_000_000
  if (gb >= 1) {
    return `${Number.isInteger(gb) ? gb : Number(gb.toFixed(1))}GB`
  }
  const mb = bytes / 1_000_000
  if (mb >= 1) {
    return `${Number.isInteger(mb) ? mb : Number(mb.toFixed(1))}MB`
  }
  return `${bytes}B`
}

function toSafeNumber(value: number | LongLike | null | undefined): number {
  if (typeof value === 'number') {
    return Number.isFinite(value) ? value : 0
  }
  if (value != null && typeof value.toNumber === 'function') {
    const numeric = value.toNumber()
    return Number.isFinite(numeric) ? numeric : 0
  }
  return 0
}

function getUsbAuditActionText(
  entry: usb_control.IUsbAuditLogEntry,
  deviceKind: UsbAuditDeviceKind,
): string {
  const eventType = String(entry.eventType ?? '')
  const result = String(entry.result ?? '').toLowerCase()
  const failReason = String(entry.failReason ?? '')
  const detail = String(entry.detail ?? '')
  const source = `${failReason} ${detail}`.trim()

  if (eventType === 'device_remove') {
    return ''
  }

  if (deviceKind === 'unsupported') {
    return '禁止使用'
  }

  if (eventType === 'insert_success') {
    if (deviceKind === 'keyboard') {
      return source.includes('验证') ? `${normalizeUsbAuditReason(source)}, 映射完成` : '验证通过, 映射完成'
    }
    if (deviceKind === 'mouse') {
      return '映射完成'
    }
    return '映射完成'
  }

  if (source.includes('扫描中断')) {
    return source.includes('拔出') ? '扫描中断, U 盘拔出' : '扫描中断'
  }
  if (source.includes('挂载失败')) {
    return '挂载失败'
  }
  if (source.includes('验证未通过') || source.includes('验证失败')) {
    return '验证未通过'
  }
  if (source.includes('映射失败') || result === 'failed') {
    return '映射失败'
  }
  if (source.includes('未授权') || source.includes('不在白名单') || result === 'denied' || result === 'blocked') {
    return '禁止使用'
  }

  return normalizeUsbAuditReason(source) || '禁止使用'
}

function normalizeUsbAuditReason(value: string): string {
  const text = value.trim()
  if (text === '') {
    return ''
  }
  if (text.includes('不支持的设备类型')) {
    return '禁止使用'
  }
  return text
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
    logCategory: formatOperationLogCategory(entry.logCategory ?? ''),
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

function eventChipClass(eventType: string | undefined): string {
  if (eventType === 'USB插入成功') {
    return 'success'
  }
  if (eventType === 'USB拔出') {
    return 'info'
  }
  if (eventType === 'USB插入失败') {
    return 'danger'
  }
  return 'info'
}

function selectLogType(logType: LogType): void {
  if (activeLogType.value === logType) {
    return
  }
  handleTabChange(logType)
}

function resetSearchState(): void {
  dateRange.value = createDefaultRange()
  keyword.value = ''
  selectedEventType.value = ''
  page.value = 1
  pageSize.value = PAGE_SIZE
}

function handleTabChange(tabName: string | number): void {
  activeLogType.value = tabName as LogType
  resetSearchState()
  void loadLogs()
}

async function handleSearch(): Promise<void> {
  if (!(await canWriteToDevice())) {
    return
  }
  page.value = 1
  void loadLogs()
}

function handlePageChange(nextPage: number): void {
  page.value = clampPage(nextPage)
  void loadLogs()
}

function clampPage(nextPage: number): number {
  if (!Number.isFinite(nextPage)) {
    return 1
  }
  return Math.min(Math.max(1, Math.trunc(nextPage)), totalPages.value)
}

function handlePageSizeChange(nextPageSize: number): void {
  pageSize.value = nextPageSize
  page.value = 1
  void loadLogs()
}

async function handleExport(): Promise<void> {
  if (isExporting.value || !(await canWriteToDevice())) {
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

function buildExportInput(): LogExportInput {
  const input = buildQueryInput()
  return {
    logType: input.logType,
    startTime: input.startTime,
    endTime: input.endTime,
    keyword: input.keyword,
    eventType: input.eventType,
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
  if (!(await canWriteToDevice())) {
    return
  }
  clearRange.value = [new Date(dateRange.value[0]), new Date(dateRange.value[1])]
  clearDialogVisible.value = true
}

async function confirmClear(): Promise<void> {
  if (!(await canWriteToDevice())) {
    return
  }
  const startSeconds = dateToUnixSeconds(clearRange.value[0])
  const endSeconds = dateToUnixSeconds(clearRange.value[1])
  const startText = formatUnixSeconds(startSeconds)
  const endText = formatUnixSeconds(endSeconds)
  try {
    await confirmAction({
      message: `请确认是否清除 ${startText} - ${endText} 的${activeTabLabel.value}？`,
      title: '清理日志',
      confirmButtonText: '确认清理',
      type: 'warning',
    })
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
  <div class="logs-page app-page">
    <header class="page-header app-page-header">
      <div>
        <h1 class="app-page-title">日志管理</h1>
        <p class="app-page-desc">USB操作和安全事件的审计追踪</p>
      </div>
    </header>
    <ConnectionAlert />

    <section class="logs-prototype-shell" data-testid="logs-prototype-shell">
      <nav class="logs-segmented-tabs" aria-label="日志类型">
        <button
          v-for="tab in LOG_TABS"
          :key="tab.value"
          class="logs-segmented-tab"
          :class="{ active: activeLogType === tab.value }"
          type="button"
          :data-testid="`logs-tab-${tab.value}`"
          @click="selectLogType(tab.value)"
        >
          {{ tab.label }}
        </button>
      </nav>

      <DataTable
        class="logs-table prototype-table"
        :columns="columns"
        :data="rows"
        :loading="isLoading"
        :error="errorMessage"
        :total="total"
        :page="page"
        :page-size="pageSize"
        empty-text="暂无日志"
        :show-default-pagination="false"
        data-testid="logs-table-shell"
        @page-change="handlePageChange"
        @page-size-change="handlePageSizeChange"
      >
        <template #filters>
          <div
            class="log-filter-bar"
            :class="{ 'without-type-filter': !showUsbEventFilter }"
          >
            <el-input
              v-model="keyword"
              class="filter-keyword"
              placeholder="搜索内容..."
              clearable
              data-testid="log-keyword"
              @keyup.enter="handleSearch"
            />
            <el-date-picker
              v-model="startTime"
              class="filter-start"
              type="datetime"
              placeholder="开始时间"
              :clearable="false"
              data-testid="log-start-time"
            />
            <el-date-picker
              v-model="endTime"
              class="filter-end"
              type="datetime"
              placeholder="结束时间"
              :clearable="false"
              data-testid="log-end-time"
            />
            <el-select
              v-if="showUsbEventFilter"
              v-model="selectedEventType"
              class="filter-select"
              placeholder="全部类型"
              data-testid="log-event-type"
            >
              <el-option
                v-for="option in USB_EVENT_TYPE_OPTIONS"
                :key="option.value"
                :label="option.value === '' ? '全部类型' : option.label"
                :value="option.value"
              />
            </el-select>
            <div class="filter-actions app-filter-actions">
              <el-button
                type="primary"
                data-testid="log-search"
                :disabled="isBusinessActionDisabled || isLoading"
                @click="handleSearch"
              >
                搜索
              </el-button>
              <el-button
                data-testid="log-export"
                :loading="isExporting"
                :disabled="isBusinessActionDisabled || isExporting"
                @click="handleExport"
              >
                导出 .zip
              </el-button>
              <el-button
                class="logs-clear-button"
                data-testid="log-clear"
                :disabled="isBusinessActionDisabled"
                @click="handleClear"
              >
                清理
              </el-button>
            </div>
          </div>
        </template>
        <template #serialNumber="{ row }">
          <span class="log-serial-chip" data-testid="log-serial-chip">{{ row.serialNumber }}</span>
        </template>
        <template #eventType="{ row }">
          <span
            class="log-event-chip"
            :class="eventChipClass(row.eventType)"
            data-testid="log-event-chip"
          >
            {{ row.eventType }}
          </span>
        </template>
      </DataTable>

      <nav
        class="logs-prototype-pagination"
        data-testid="logs-prototype-pagination"
        aria-label="日志分页"
      >
        <span>显示 {{ rangeStart }}-{{ rangeEnd }}，共 {{ total }} 条</span>
        <div class="logs-page-buttons">
          <button
            v-for="pageNumber in visiblePages"
            :key="pageNumber"
            type="button"
            class="logs-page-button"
            :class="{ active: page === pageNumber }"
            @click="handlePageChange(pageNumber)"
          >
            {{ pageNumber }}
          </button>
          <button
            v-if="showPaginationEllipsis"
            type="button"
            class="logs-page-button"
            @click="handlePageChange(Math.min(totalPages, page + 1))"
          >
            ...
          </button>
        </div>
      </nav>
    </section>

    <el-dialog v-model="clearDialogVisible" title="清理日志" width="520px">
      <div class="clear-dialog-body">
        <div class="app-danger-block">
          <p>当前日志类型：{{ activeTabLabel }}</p>
          <p>清理后该时间范围内的日志将不可恢复，请确认后继续。</p>
        </div>
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
        <el-button
          type="danger"
          data-testid="log-clear-confirm"
          :disabled="isBusinessActionDisabled"
          @click="confirmClear"
        >
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
@use '@/styles/tokens' as *;

.logs-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.page-header {
  margin-bottom: 4px;

  h1 {
    margin: 0;
    color: var(--andi-text);
    font-size: 26px;
    font-weight: $font-weight-semibold;
    line-height: 1.2;
  }

  p {
    margin: 6px 0 0;
    color: var(--andi-text-light);
    font-size: 16px;
    font-weight: $font-weight-medium;
  }
}

.logs-prototype-shell {
  display: flex;
  flex-direction: column;
  gap: 18px;
  overflow: visible;
  border: 0;
  border-radius: 0;
  background: transparent;
}

.logs-segmented-tabs {
  display: flex;
  align-self: flex-start;
  overflow: hidden;
  border: 1px solid #d8dee8;
  border-radius: 5px;
  background: #f3f6fa;
  gap: 0;
}

.logs-segmented-tab {
  min-width: 134px;
  height: 40px;
  padding: 0 20px;
  border: 0;
  border-right: 1px solid #d8dee8;
  background: transparent;
  color: var(--andi-text-light);
  font-family: var(--andi-font-family);
  font-size: 16px;
  font-weight: $font-weight-semibold;
  cursor: pointer;
  transition:
    background-color 120ms ease,
    color 120ms ease;

  &.active {
    border-right-color: var(--andi-blue);
    background: var(--andi-blue);
    color: var(--andi-white);
  }

  &:last-child {
    border-right: 0;
  }

  &:focus-visible {
    outline: 2px solid rgba(0, 86, 179, 0.26);
    outline-offset: -2px;
  }

  &:not(.active):hover {
    background: #f9fbff;
    color: var(--andi-blue);
  }
}

.logs-table {
  overflow: visible;
  border-radius: 8px;
}

.logs-table :deep(.data-table-wrapper) {
  gap: 18px;
}

.logs-table :deep(.app-filter-bar) {
  display: block;
}

.logs-table :deep(.el-table) {
  overflow: hidden;
  border: 1px solid #d8dee8;
  border-radius: 8px;
  font-size: 16px;
}

.logs-table :deep(.el-table__border-left-patch),
.logs-table :deep(.el-table__inner-wrapper::before),
.logs-table :deep(.el-table__inner-wrapper::after) {
  display: none;
}

.logs-table :deep(th.el-table__cell) {
  height: 42px;
  padding: 0 16px;
  background: #eef2f7;
  color: var(--andi-text-secondary);
  font-size: 15px;
  font-weight: $font-weight-semibold;
}

.logs-table :deep(td.el-table__cell) {
  height: 48px;
  padding: 0 16px;
  background: var(--andi-white);
  color: var(--andi-text);
  font-size: 16px;
}

.logs-table :deep(.el-table__body tr.el-table__row--striped td.el-table__cell),
.logs-table :deep(.el-table__body tr:hover > td.el-table__cell) {
  background: var(--andi-white);
}

.log-filter-bar {
  display: grid;
  grid-template-columns: 112px 168px 168px 108px minmax(0, 1fr);
  align-items: center;
  width: 100%;
  gap: 10px;
}

.log-filter-bar.without-type-filter {
  grid-template-columns: 208px 176px 176px minmax(0, 1fr);
}

.filter-keyword,
.filter-start,
.filter-end,
.filter-select {
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  min-width: 0;
}

.filter-start.filter-start,
.filter-end.filter-end,
.filter-select.filter-select {
  width: 100%;
  max-width: 100%;
  min-width: 0;
}

.logs-page :deep(.filter-start.el-date-editor.el-input),
.logs-page :deep(.filter-end.el-date-editor.el-input),
.logs-page :deep(.filter-select.el-select) {
  width: 100%;
  max-width: 100%;
  min-width: 0;
}

.filter-keyword :deep(.el-input__wrapper),
.filter-start :deep(.el-input__wrapper),
.filter-end :deep(.el-input__wrapper),
.filter-select :deep(.el-select__wrapper) {
  min-height: 34px;
  border-radius: 4px;
  box-shadow: 0 0 0 1px #d8dee8 inset;
  font-size: 13px;
}

.filter-keyword :deep(.el-input__inner),
.filter-start :deep(.el-input__inner),
.filter-end :deep(.el-input__inner),
.filter-select :deep(.el-select__placeholder),
.filter-select :deep(.el-select__selected-item) {
  font-size: 13px;
}

.filter-actions {
  display: flex;
  justify-content: flex-end;
  min-width: 0;
  margin-left: auto;
  gap: 6px;
}

.filter-actions :deep(.el-button) {
  min-width: 58px;
  height: 34px;
  margin-left: 0;
  padding: 0 12px;
  border-radius: 4px;
  font-size: 13px;
  font-weight: $font-weight-semibold;
}

.filter-actions :deep([data-testid='log-export']) {
  min-width: 76px;
}

.filter-actions :deep(.el-button--primary) {
  background: var(--andi-blue);
  border-color: var(--andi-blue);
}

.logs-clear-button {
  color: #e53935;
  background: var(--andi-white);
  border-color: #ff8f8f;

  &:hover,
  &:focus {
    color: var(--andi-white);
    background: #e53935;
    border-color: #e53935;
  }
}

.log-serial-chip,
.log-event-chip {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 0 8px;
  border-radius: $radius-control;
  font-size: 15px;
  font-weight: $font-weight-semibold;
  line-height: 1;
}

.log-serial-chip {
  background: #f0f2f5;
  color: var(--andi-blue-dark);
}

.log-event-chip {
  &.success {
    background: color-mix(in srgb, var(--andi-success) 12%, transparent);
    color: var(--andi-success);
  }

  &.danger {
    background: color-mix(in srgb, var(--andi-danger) 12%, transparent);
    color: var(--andi-danger);
  }

  &.warning {
    background: color-mix(in srgb, var(--andi-warning) 12%, transparent);
    color: var(--andi-warning);
  }

  &.info {
    background: color-mix(in srgb, var(--andi-info) 12%, transparent);
    color: var(--andi-info);
  }
}

.logs-prototype-pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0;
  border-top: 0;
  color: var(--andi-text-light);
  font-size: 15px;
}

.logs-page-buttons {
  display: flex;
  gap: 8px;
}

.logs-page-button {
  min-width: 30px;
  height: 30px;
  border: 1px solid #d8dee8;
  border-radius: $radius-control;
  background: var(--andi-white);
  color: var(--andi-text);
  font-size: 15px;
  cursor: pointer;

  &.active {
    border-color: var(--andi-blue);
    background: var(--andi-blue);
    color: var(--andi-white);
  }

  &:not(.active):hover,
  &:focus-visible {
    border-color: var(--andi-blue);
    color: var(--andi-blue);
  }
}

.clear-dialog-body {
  display: flex;
  flex-direction: column;
  gap: $spacing-4;

  p {
    margin: 0;
    color: var(--andi-text-secondary);
  }
}

.clear-range {
  width: 100%;
}

@media (max-width: 720px) {
  .logs-prototype-pagination {
    align-items: flex-start;
    flex-direction: column;
    gap: $spacing-3;
  }
}
</style>
