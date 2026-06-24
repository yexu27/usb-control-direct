<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import AddWhitelistDialog from '@/components/whitelist/AddWhitelistDialog.vue'
import EditWhitelistDialog from '@/components/whitelist/EditWhitelistDialog.vue'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import DataTable from '@/components/DataTable.vue'
import type { DataTableColumn } from '@/components/data-table'
import { getConnectedDevices } from '@/services/device-service'
import { listManagementUsbStorageDevices } from '@/services/management-usb-service'
import { useConnectionStore } from '@/stores/connection'
import { useSessionStore } from '@/stores/session'
import { useWhitelistStore } from '@/stores/whitelist'
import { confirmAction } from '@/utils/confirm-action'
import { errorMessage, showErrorDialog, showSuccessToast } from '@/utils/operation-feedback'

const PAGE_SIZE = 20
const DISCONNECTED_MESSAGE = '装置已断开连接，无法修改白名单'
const REMOVED_MESSAGE = '设备已移除，请重新插入后再添加'
const MANAGEMENT_ENUM_ERROR = '管理端 USB 设备读取失败，请重试'
const WHITELIST_LOAD_ERROR = 'U盘白名单加载失败，请重试'

type AddSource = 'device' | 'management'
type WhitelistPermission = 'readonly' | 'readwrite'

interface CandidateDevice {
  serialNumber: string
  vid: string
  pid: string
  deviceName: string
  capacityBytes: number
  deviceType: 'storage'
  addable: boolean
  unavailableReason: string
}

interface AddWhitelistFormValue {
  candidate: CandidateDevice
  description: string
  permission: WhitelistPermission
}

interface EditWhitelistFormValue {
  description: string
  permission: WhitelistPermission
}

interface WhitelistRow {
  serialNumber: string
  description: string
  permission: string
  permissionLabel: string
  addMethodLabel: string
  createdAtLabel: string
}

const session = useSessionStore()
const connection = useConnectionStore()
const whitelist = useWhitelistStore()
const page = ref(1)
const pageSize = ref(PAGE_SIZE)
const addVisible = ref(false)
const addSource = ref<AddSource>('device')
const candidates = ref<CandidateDevice[]>([])
const candidatesLoading = ref(false)
const addSubmitting = ref(false)
const addErrorMessage = ref('')
const addEpoch = ref(0)
const editVisible = ref(false)
const editTarget = ref<WhitelistRow | null>(null)
const editSubmitting = ref(false)
const editErrorMessage = ref('')
const editEpoch = ref(0)
const addOwnership = new Set<string>()
const editOwnership = new Set<string>()
const removeOwnership = ref(new Set<string>())
const loadEpochs: Record<AddSource, number> = { device: 0, management: 0 }

const columns: DataTableColumn[] = [
  { prop: 'serialNumber', label: '序列号', minWidth: 180, slot: 'serialNumber' },
  { prop: 'description', label: '描述', minWidth: 200 },
  { prop: 'permissionLabel', label: '权限', width: 100, slot: 'permissionLabel' },
  { prop: 'addMethodLabel', label: '添加方式', width: 130 },
  { prop: 'createdAtLabel', label: '添加时间', width: 180 },
  { prop: 'actions', label: '操作', width: 120, slot: 'actions' },
]

function pad(value: number): string {
  return String(value).padStart(2, '0')
}
function formatUnixSeconds(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return '--'
  }
  const milliseconds = seconds * 1000
  if (!Number.isFinite(milliseconds)) {
    return '--'
  }
  const date = new Date(milliseconds)
  if (!Number.isFinite(date.getTime())) {
    return '--'
  }
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ` +
    `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}
function permissionLabel(permission: string): string {
  return permission === 'readonly' ? '只读' : permission === 'readwrite' ? '读写' : permission
}
function addMethodLabel(addMethod: string): string {
  return addMethod === 'device' ? '装置端添加' : addMethod === 'management' ? '管理端添加' : addMethod
}

const rows = computed<WhitelistRow[]>(() => whitelist.devices.map((device) => ({
  serialNumber: device.serialNumber,
  description: device.description,
  permission: device.permission,
  permissionLabel: permissionLabel(device.permission),
  addMethodLabel: addMethodLabel(device.addMethod),
  createdAtLabel: formatUnixSeconds(Number(device.createdAt)),
})))
const pageRows = computed(() => {
  const start = (page.value - 1) * pageSize.value
  return rows.value.slice(start, start + pageSize.value)
})
const editPermission = computed<WhitelistPermission>(() => {
  const permission = editTarget.value?.permission ?? ''
  return validPermission(permission) ? permission : 'readonly'
})
const tableError = computed(() => whitelist.errorMessage === '' ? '' : WHITELIST_LOAD_ERROR)
watch([() => rows.value.length, pageSize], ([total, size]) => {
  page.value = Math.min(page.value, Math.max(1, Math.ceil(total / size)))
})

async function showError(error: unknown, fallback: string): Promise<void> {
  await showErrorDialog(fallback, errorMessage(error, fallback))
}
function canWrite(): boolean {
  if (connection.isConnected) {
    return true
  }
  ElMessage.warning(DISCONNECTED_MESSAGE)
  return false
}
function validPermission(value: string): value is WhitelistPermission {
  return value === 'readonly' || value === 'readwrite'
}
function mapManagementCandidates(devices: Awaited<ReturnType<typeof listManagementUsbStorageDevices>>): CandidateDevice[] {
  return devices.map((device) => {
    const missingSerial = device.serialNumber.trim() === ''
    return {
      ...device,
      addable: device.addable && !missingSerial,
      unavailableReason: missingSerial ? '设备标识异常' : device.unavailableReason,
    }
  })
}
function invalidateCandidateLoad(source: AddSource): void {
  loadEpochs[source] += 1
  if (addSource.value === source) {
    candidatesLoading.value = false
  }
}

async function openAddDialog(source: AddSource): Promise<void> {
  if (
    addSubmitting.value ||
    (addVisible.value && addSource.value === source) ||
    !canWrite()
  ) {
    return
  }
  invalidateCandidateLoad(addSource.value)
  addEpoch.value += 1
  const dialogEpoch = addEpoch.value
  addSource.value = source
  addVisible.value = true
  addErrorMessage.value = ''
  candidates.value = []
  const epoch = ++loadEpochs[source]
  candidatesLoading.value = true
  try {
    const loaded = source === 'device'
      ? (await getConnectedDevices(session.token)).devices
        .filter((device) => device.deviceType === 'storage')
        .map<CandidateDevice>((device) => {
          const serialNumber = device.serialNumber ?? ''
          const capacityBytes = Number(device.capacityBytes ?? 0)
          return {
            serialNumber,
            vid: device.vid ?? '',
            pid: device.pid ?? '',
            deviceName: device.deviceName ?? '',
            capacityBytes: Number.isFinite(capacityBytes) && capacityBytes >= 0 ? capacityBytes : 0,
            deviceType: 'storage',
            addable: device.admissionStatus === 'addable' && serialNumber.trim() !== '',
            unavailableReason: serialNumber.trim() === ''
              ? '设备标识异常' : (device.failReason ?? ''),
          }
        })
      : mapManagementCandidates(await listManagementUsbStorageDevices())
    if (
      addVisible.value && addSource.value === source && addEpoch.value === dialogEpoch &&
      loadEpochs[source] === epoch
    ) {
      candidates.value = loaded
      if (loaded.filter((candidate) => candidate.addable).length === 0) {
        ElMessage.warning('未检测到可添加的 U 盘设备')
      }
    }
  } catch (error: unknown) {
    if (
      addVisible.value && addSource.value === source && addEpoch.value === dialogEpoch &&
      loadEpochs[source] === epoch
    ) {
      addErrorMessage.value = source === 'management'
        ? MANAGEMENT_ENUM_ERROR
        : errorMessage(error, '装置端 USB 设备读取失败，请重试')
    }
  } finally {
    if (addSource.value === source && loadEpochs[source] === epoch) {
      candidatesLoading.value = false
    }
  }
}

function changeAddVisible(visible: boolean): void {
  if (!visible) {
    addEpoch.value += 1
    invalidateCandidateLoad(addSource.value)
    candidates.value = []
    addErrorMessage.value = ''
  }
  addVisible.value = visible
}

async function handleAdd(value: AddWhitelistFormValue): Promise<void> {
  const source = addSource.value
  const dialogEpoch = addEpoch.value
  const serialNumber = value.candidate.serialNumber.trim()
  const dialogCandidate = candidates.value.find(
    (candidate) => candidate.serialNumber.trim() === serialNumber && candidate.addable,
  )
  const ownershipKey = `${source}:${serialNumber}`
  if (addOwnership.has(ownershipKey) || addSubmitting.value || !canWrite()) {
    return
  }
  if (serialNumber === '' || dialogCandidate == null || !validPermission(value.permission)) {
    return
  }
  if (whitelist.pendingSerialNumbers.has(serialNumber)) {
    return
  }
  addOwnership.add(ownershipKey)
  addSubmitting.value = true
  addErrorMessage.value = ''
  try {
    let candidate = dialogCandidate
    if (source === 'management') {
      let currentDevices: CandidateDevice[]
      try {
        currentDevices = mapManagementCandidates(await listManagementUsbStorageDevices())
      } catch {
        if (addEpoch.value === dialogEpoch && addVisible.value && addSource.value === source) {
          addErrorMessage.value = MANAGEMENT_ENUM_ERROR
        }
        return
      }
      if (addEpoch.value !== dialogEpoch || !addVisible.value || addSource.value !== source) {
        return
      }
      const matches = currentDevices.filter(
        (device) => device.serialNumber.trim() === serialNumber,
      )
      const current = matches.length === 1 ? matches[0] : undefined
      if (current == null || !current.addable || current.serialNumber.trim() === '') {
        addErrorMessage.value = REMOVED_MESSAGE
        return
      }
      candidate = current
    }
    if (!canWrite()) {
      return
    }
    if (whitelist.pendingSerialNumbers.has(serialNumber)) {
      return
    }
    await whitelist.addWhitelist(session.token, {
      serialNumber,
      vid: candidate.vid,
      pid: candidate.pid,
      deviceName: candidate.deviceName,
      capacityBytes: candidate.capacityBytes,
      permission: value.permission,
      description: value.description,
      addMethod: source,
      deviceType: 'storage',
    })
    if (
      addEpoch.value !== dialogEpoch || !addVisible.value || addSource.value !== source ||
      !canWrite()
    ) {
      return
    }
    changeAddVisible(false)
    showSuccessToast('添加成功，重新拔插后生效')
  } catch (error: unknown) {
    if (addEpoch.value === dialogEpoch && addVisible.value && addSource.value === source) {
      addErrorMessage.value = errorMessage(error, '白名单添加失败')
    }
  } finally {
    addSubmitting.value = false
    addOwnership.delete(ownershipKey)
  }
}

function openEditDialog(row: WhitelistRow): void {
  if (editSubmitting.value || !validPermission(row.permission)) {
    return
  }
  editEpoch.value += 1
  editErrorMessage.value = ''
  editTarget.value = row
  editVisible.value = true
}
function changeEditVisible(visible: boolean): void {
  if (!visible) {
    editEpoch.value += 1
    editVisible.value = false
    editTarget.value = null
    editErrorMessage.value = ''
    return
  }
  if (editTarget.value == null) {
    return
  }
  editVisible.value = true
}
async function handleEdit(value: EditWhitelistFormValue): Promise<void> {
  const target = editTarget.value
  if (!editVisible.value || target == null) {
    return
  }
  const serialNumber = target.serialNumber
  const dialogEpoch = editEpoch.value
  if (editOwnership.has(serialNumber) || editSubmitting.value || !canWrite()) {
    return
  }
  if (!validPermission(value.permission) || whitelist.pendingSerialNumbers.has(serialNumber)) {
    return
  }
  editOwnership.add(serialNumber)
  editSubmitting.value = true
  editErrorMessage.value = ''
  try {
    await whitelist.updateWhitelist(session.token, serialNumber, value.permission, value.description)
    if (
      editEpoch.value !== dialogEpoch || !editVisible.value ||
      editTarget.value?.serialNumber !== serialNumber || !canWrite()
    ) {
      return
    }
    changeEditVisible(false)
    showSuccessToast('修改成功，重新拔插后生效')
  } catch (error: unknown) {
    if (
      editEpoch.value === dialogEpoch && editVisible.value &&
      editTarget.value?.serialNumber === serialNumber
    ) {
      editErrorMessage.value = errorMessage(error, '白名单修改失败')
    }
  } finally {
    editSubmitting.value = false
    editOwnership.delete(serialNumber)
  }
}

async function handleRemove(serialNumber: string): Promise<void> {
  if (removeOwnership.value.has(serialNumber) || !canWrite()) {
    return
  }
  removeOwnership.value.add(serialNumber)
  try {
    try {
      await confirmAction({
        message: `确定删除白名单设备 ${serialNumber} 吗？`,
        title: '删除确认',
        confirmButtonText: '删除',
        type: 'warning',
      })
    } catch {
      return
    }
    if (!canWrite() || whitelist.pendingSerialNumbers.has(serialNumber)) {
      return
    }
    await whitelist.removeWhitelist(session.token, serialNumber)
    if (!canWrite()) {
      return
    }
    showSuccessToast('删除成功')
  } catch (error: unknown) {
    await showError(error, '白名单删除失败')
  } finally {
    removeOwnership.value.delete(serialNumber)
  }
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
  <div class="usb-devices-page app-page">
    <header class="page-header app-page-header">
      <div>
        <h1 class="app-page-title">U盘设备控制</h1>
        <p class="app-page-desc">管理受信任移动存储设备白名单</p>
      </div>
    </header>
    <ConnectionAlert />
    <section class="usb-section" aria-label="受信任普通移动存储设备白名单">
      <div class="usb-panel" data-testid="usb-table-panel">
        <div class="usb-panel-header">
          <h3>受信任普通移动存储设备白名单</h3>
          <div class="usb-panel-actions">
            <el-button
              data-testid="add-device-trigger"
              :disabled="addSubmitting"
              :loading="addSubmitting"
              @click="openAddDialog('device')"
            >
              装置端添加
            </el-button>
            <el-button
              type="primary"
              data-testid="add-management-trigger"
              :disabled="addSubmitting"
              :loading="addSubmitting"
              @click="openAddDialog('management')"
            >
              管理端添加
            </el-button>
          </div>
        </div>
        <DataTable
          class="usb-table prototype-table"
          :columns="columns" :data="pageRows" :loading="whitelist.isLoading"
          :error="tableError" :total="rows.length" :page="page" :page-size="pageSize"
          empty-text="暂无数据" @page-change="changePage" @page-size-change="changePageSize"
        >
          <template #serialNumber="{ row }">
            <span class="serial-chip">{{ row.serialNumber }}</span>
          </template>
          <template #permissionLabel="{ row }">
            <span
              :class="[
                'permission-chip',
                row.permission === 'readwrite' ? 'permission-rw' : 'permission-ro',
              ]"
            >
              {{ row.permissionLabel }}
            </span>
          </template>
          <template #actions="{ row }">
            <el-button
              class="prototype-outline-action prototype-outline-danger"
              :data-testid="`remove-${row.serialNumber}`"
              :disabled="removeOwnership.has(row.serialNumber)"
              :loading="removeOwnership.has(row.serialNumber)"
              @click="handleRemove(row.serialNumber)"
            >
              删除
            </el-button>
          </template>
        </DataTable>
      </div>
      <div class="usb-bottom-note">
        白名单设备重新插入后经过扫描审计方可使用。未授权设备显示黄色指示灯常亮。添加完成后需重新拔插U盘生效。
      </div>
    </section>
    <AddWhitelistDialog
      :key="`${addSource}-${addEpoch}`"
      :visible="addVisible" :source="addSource" :candidates="candidates"
      :loading="candidatesLoading" :submitting="addSubmitting"
      :error-message="addErrorMessage"
      @update:visible="changeAddVisible" @submit="handleAdd"
    />
    <EditWhitelistDialog
      :key="editEpoch" :visible="editVisible" :serial-number="editTarget?.serialNumber ?? ''"
      :current-description="editTarget?.description ?? ''"
      :current-permission="editPermission"
      :submitting="editSubmitting" :error-message="editErrorMessage"
      @update:visible="changeEditVisible" @submit="handleEdit"
    />
  </div>
</template>

<style scoped lang="scss">
.usb-devices-page {
  display: flex;
  flex-direction: column;
  gap: $spacing-5;
}
.page-header h1 {
  margin: 0;
}
.page-header p {
  margin: $spacing-2 0 0;
  color: $text-secondary;
}

.usb-section {
  margin-top: 28px;
}

.usb-panel {
  overflow: hidden;
  background: $bg-white;
  border: 1px solid #dce2ea;
  border-radius: 8px;
  box-shadow: 0 1px 4px rgba(16, 24, 40, 0.06);
}

.usb-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 64px;
  padding: 0 18px 0 20px;
  background: $bg-sidebar;
  border-bottom: 1px solid #dce2ea;
  gap: 16px;
}

.usb-panel-header h3 {
  margin: 0;
  color: $text-primary;
  font-size: 18px;
  font-weight: $font-weight-semibold;
}

.usb-panel-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.usb-table :deep(.data-table-wrapper) {
  gap: 0;
}

.serial-chip {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 0 7px;
  color: $brand-primary-dark;
  font-family: Consolas, 'SF Mono', monospace;
  font-weight: $font-weight-semibold;
  background: #f0f2f5;
  border-radius: 4px;
}

.permission-chip {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 0 10px;
  border-radius: 5px;
  font-size: 13px;
  font-weight: $font-weight-semibold;
}

.permission-rw {
  color: #2e7d32;
  background: #e8f5e9;
}

.permission-ro {
  color: #e65100;
  background: #fff8e1;
}

.usb-bottom-note {
  margin-top: 12px;
  color: var(--andi-text-light);
  font-size: 13px;
  font-weight: $font-weight-medium;
}
</style>
