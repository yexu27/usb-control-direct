<script setup lang="ts">
import { computed, onUnmounted, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import ProgressDialog from '@/components/ProgressDialog.vue'
import { getMachineCode, uploadLicense } from '@/services/auth-service'
import {
  getSystemInfo,
  updateDeviceDescription,
  uploadSystemUpgrade,
  uploadVirusdbUpgrade,
} from '@/services/system-service'
import { useConnectionStore } from '@/stores/connection'
import { useSessionStore } from '@/stores/session'
import { formatUnixSeconds } from '@/utils/date-time'
import {
  calculateSha256Hex,
  parseSystemUpgradeVersion,
  parseVirusdbUpgradeVersion,
} from '@/utils/upgrade-package'
import { alertAction, confirmAction } from '@/utils/confirm-action'
import { errorMessage, showErrorDialog, showSuccessToast } from '@/utils/operation-feedback'
import type { usb_control } from '../../shared/proto/usb_control'

const session = useSessionStore()
const connection = useConnectionStore()

const systemInfo = ref<usb_control.RspSystemInfo | null>(null)
const isLoadingInfo = ref(false)
const systemUpgrading = ref(false)
const virusdbUpgrading = ref(false)
const licenseUploading = ref(false)
const machineCodeDialogVisible = ref(false)
const machineCodeLoading = ref(false)
const machineCode = ref('')
const qrcodePng = ref<Uint8Array>(new Uint8Array())
const qrcodeUrl = ref('')
const deviceDescription = ref('')
const deviceDescriptionDialogVisible = ref(false)
const deviceDescriptionSaving = ref(false)

const systemVersion = computed(() => systemInfo.value?.systemVersion ?? '-')
const virusDbVersion = computed(() => systemInfo.value?.virusDbVersion ?? '-')
const virusDbUpdatedAt = computed(() => formatUnixSeconds(systemInfo.value?.virusDbUpdatedAt ?? 0))
const authExpireTime = computed(() => formatUnixSeconds(systemInfo.value?.authExpireTime ?? 0))
const authStatusText = computed(() => {
  return formatAuthStatus(systemInfo.value?.authStatus ?? '', systemInfo.value?.authorized ?? false)
})
const currentDeviceDescription = computed(() => systemInfo.value?.deviceDescription ?? '-')
const transferVisible = computed(() => systemUpgrading.value || virusdbUpgrading.value || licenseUploading.value)
const transferTitle = computed(() => {
  if (systemUpgrading.value) {
    return '系统升级'
  }
  if (virusdbUpgrading.value) {
    return '病毒库升级'
  }
  return '上传授权文件'
})

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms)
  })
}

async function waitAtLeast(startedAt: number, minimumMs = 3_000): Promise<void> {
  const remainingMs = minimumMs - (Date.now() - startedAt)
  if (remainingMs > 0) {
    await delay(remainingMs)
  }
}

onMounted(() => {
  void loadSystemInfo()
})

onUnmounted(() => {
  if (qrcodeUrl.value !== '') {
    URL.revokeObjectURL(qrcodeUrl.value)
  }
})

function formatAuthStatus(status: string, authorized: boolean): string {
  if (status === 'expired') {
    return '已到期'
  }
  if (status === 'unauthorized') {
    return '未授权'
  }
  return authorized ? '已授权' : '未授权'
}

async function showError(error: unknown, fallback: string): Promise<void> {
  await showErrorDialog(fallback, errorMessage(error, fallback))
}

function blurActiveElement(): void {
  if (document.activeElement instanceof HTMLElement) {
    document.activeElement.blur()
  }
}

function releaseActiveElementFocus(): void {
  blurActiveElement()
  window.setTimeout(blurActiveElement, 0)
}

async function showUploadResult(title: string, message: string, type: 'success' | 'error' | 'warning'): Promise<void> {
  try {
    if (type === 'success') {
      showSuccessToast(message)
    } else {
      await showErrorDialog(title, message)
    }
  } finally {
    releaseActiveElementFocus()
  }
}

function canOperate(): boolean {
  if (connection.isConnected) {
    return true
  }
  ElMessage.warning('USB 管控装置已断开连接，操作失败。')
  return false
}

async function loadSystemInfo(): Promise<void> {
  if (!canOperate()) {
    return
  }
  isLoadingInfo.value = true
  try {
    const response = await getSystemInfo(session.token)
    systemInfo.value = response
    deviceDescription.value = response.deviceDescription
  } catch (error: unknown) {
    await showError(error, '系统信息加载失败')
  } finally {
    isLoadingInfo.value = false
  }
}

async function uploadSystemUpgradePackage(): Promise<void> {
  if (systemUpgrading.value || !canOperate()) {
    return
  }
  const result = await window.desktopApi.dialog.openFile({
    title: '选择系统升级包',
    filters: [{ name: '系统升级包', extensions: ['bin'] }],
  })
  if (result.canceled || result.filePaths.length === 0) {
    return
  }
  const filePath = result.filePaths[0]
  if (!filePath.toLowerCase().endsWith('.bin')) {
    await showUploadResult('系统升级包格式错误', '仅支持 .bin 系统升级包', 'error')
    return
  }
  let targetVersion = ''
  try {
    targetVersion = parseSystemUpgradeVersion(filePath)
  } catch (error: unknown) {
    await showUploadResult('系统升级包文件名错误', errorMessage(error, '系统升级包文件名格式错误'), 'error')
    return
  }

  systemUpgrading.value = true
  const startedAt = Date.now()
  let caughtError: unknown = null
  try {
    const fileData = await window.desktopApi.dialog.readFile(filePath)
    const checksum = await calculateSha256Hex(fileData)
    await uploadSystemUpgrade(session.token, fileData, targetVersion, checksum)
  } catch (error: unknown) {
    caughtError = error
  } finally {
    await waitAtLeast(startedAt)
    systemUpgrading.value = false
  }

  if (caughtError == null) {
    await showUploadResult('系统升级完成', '升级完成，请重新连接', 'success')
    await connection.disconnect(true).catch(() => {})
  } else {
    await showUploadResult('系统升级失败', errorMessage(caughtError, '系统升级失败'), 'error')
  }
}

async function uploadVirusdbUpgradePackage(): Promise<void> {
  if (virusdbUpgrading.value || !canOperate()) {
    return
  }
  const result = await window.desktopApi.dialog.openFile({
    title: '选择病毒库升级包',
    filters: [{ name: '病毒库升级包', extensions: ['zip'] }],
  })
  if (result.canceled || result.filePaths.length === 0) {
    return
  }
  const filePath = result.filePaths[0]
  if (!filePath.toLowerCase().endsWith('.zip')) {
    await showUploadResult('病毒库升级包格式错误', '仅支持 .zip 病毒库升级包', 'error')
    return
  }
  let targetVersion = ''
  try {
    targetVersion = parseVirusdbUpgradeVersion(filePath)
  } catch (error: unknown) {
    await showUploadResult('病毒库升级包文件名错误', errorMessage(error, '病毒库升级包文件名格式错误'), 'error')
    return
  }

  virusdbUpgrading.value = true
  const startedAt = Date.now()
  let caughtError: unknown = null
  try {
    const fileData = await window.desktopApi.dialog.readFile(filePath)
    const checksum = await calculateSha256Hex(fileData)
    await uploadVirusdbUpgrade(session.token, fileData, targetVersion, checksum)
  } catch (error: unknown) {
    caughtError = error
  } finally {
    await waitAtLeast(startedAt)
    virusdbUpgrading.value = false
  }

  if (caughtError == null) {
    await showUploadResult('病毒库升级完成', '病毒库升级成功', 'success')
    await loadSystemInfo()
  } else {
    await showUploadResult('病毒库升级失败', errorMessage(caughtError, '病毒库升级失败'), 'error')
  }
}

function replaceQrcodeUrl(bytes: Uint8Array): void {
  if (qrcodeUrl.value !== '') {
    URL.revokeObjectURL(qrcodeUrl.value)
  }
  const buffer = new ArrayBuffer(bytes.byteLength)
  new Uint8Array(buffer).set(bytes)
  qrcodeUrl.value = URL.createObjectURL(new Blob([buffer], { type: 'image/png' }))
}

async function openMachineCode(): Promise<void> {
  if (machineCodeLoading.value || !canOperate()) {
    return
  }
  machineCodeDialogVisible.value = true
  machineCodeLoading.value = true
  try {
    const response = await getMachineCode(session.token)
    machineCode.value = response.machineCode
    qrcodePng.value = new Uint8Array(response.qrcodePng)
    replaceQrcodeUrl(qrcodePng.value)
  } catch (error: unknown) {
    machineCodeDialogVisible.value = false
    await showError(error, '获取机器码失败')
  } finally {
    machineCodeLoading.value = false
  }
}

async function saveQrcode(): Promise<void> {
  if (qrcodePng.value.length === 0) {
    return
  }
  const result = await window.desktopApi.dialog.saveFile({
    title: '保存机器码二维码',
    defaultPath: 'machine-code-qrcode.png',
    filters: [{ name: 'PNG 图片', extensions: ['png'] }],
  })
  if (result.canceled || result.filePath == null) {
    return
  }
  try {
    await window.desktopApi.dialog.writeFile(result.filePath, qrcodePng.value)
    showSuccessToast('机器码二维码已保存')
  } catch (error: unknown) {
    await showError(error, '机器码二维码保存失败')
  }
}

async function uploadLicenseFile(): Promise<void> {
  if (licenseUploading.value || !canOperate()) {
    return
  }
  const result = await window.desktopApi.dialog.openFile({
    title: '选择授权文件',
    filters: [{ name: '授权文件', extensions: ['txt'] }],
  })
  if (result.canceled || result.filePaths.length === 0) {
    return
  }
  const filePath = result.filePaths[0]
  if (!filePath.toLowerCase().endsWith('.txt')) {
    await showUploadResult('授权文件格式错误', '仅支持 .txt 格式授权文件', 'error')
    return
  }
  licenseUploading.value = true
  const startedAt = Date.now()
  let caughtError: unknown = null
  try {
    const fileData = await window.desktopApi.dialog.readFile(filePath)
    await uploadLicense(session.token, fileData)
  } catch (error: unknown) {
    caughtError = error
  } finally {
    await waitAtLeast(startedAt)
    licenseUploading.value = false
  }

  if (caughtError == null) {
    await showUploadResult('授权文件上传完成', '授权文件上传成功', 'success')
    await loadSystemInfo()
  } else {
    await showUploadResult('授权文件上传失败', errorMessage(caughtError, '授权文件上传失败'), 'error')
  }
}

function validateDeviceDescription(value: string): string {
  if (value.trim() === '') {
    return '请输入设备描述'
  }
  if (value.length > 32) {
    return '长度超限'
  }
  if (!/^[A-Za-z0-9_]+$/.test(value)) {
    return '设备描述格式错误'
  }
  return ''
}

function openDeviceDescriptionDialog(): void {
  deviceDescription.value = systemInfo.value?.deviceDescription ?? ''
  deviceDescriptionDialogVisible.value = true
}

async function saveDeviceDescription(): Promise<void> {
  if (deviceDescriptionSaving.value || !canOperate()) {
    return
  }
  const nextDescription = deviceDescription.value.trim()
  const validationMessage = validateDeviceDescription(nextDescription)
  if (validationMessage !== '') {
    await showErrorDialog('设备描述格式错误', validationMessage)
    return
  }
  try {
    await confirmAction({
      message: '修改设备描述前，请确认当前装置未连接移动存储设备、键盘、鼠标等 USB 设备。修改完成后需重启 USB 管控装置才能生效。',
      title: '确认修改设备描述',
      confirmButtonText: '确认修改',
      type: 'warning',
    })
  } catch {
    return
  }
  deviceDescriptionSaving.value = true
  try {
    await updateDeviceDescription(session.token, nextDescription)
    showSuccessToast('修改成功，重启 USB 管控装置后生效')
    deviceDescriptionDialogVisible.value = false
    await loadSystemInfo()
  } catch (error: unknown) {
    await showError(error, '设备描述修改失败')
  } finally {
    deviceDescriptionSaving.value = false
  }
}
</script>

<template>
  <div class="system-page app-page">
    <header class="page-header app-page-header">
      <div>
        <h1 class="app-page-title">系统管理</h1>
        <p class="app-page-desc">系统升级、授权管理和设备配置</p>
      </div>
    </header>
    <ConnectionAlert />

    <div v-loading="isLoadingInfo" class="system-grid" data-testid="system-card-grid">
      <section class="system-card system-card-upgrade" data-testid="system-management-card">
        <h3>系统升级</h3>
        <p class="system-card-meta">当前版本: {{ systemVersion }}</p>
        <div class="system-card-actions">
          <el-button
            type="primary"
            data-testid="system-upgrade-upload"
            @click="uploadSystemUpgradePackage"
          >
            上传 .bin 升级包
          </el-button>
        </div>
        <p class="system-card-note">版本校验：升级包版本必须大于当前系统版本</p>
      </section>

      <section class="system-card system-card-virusdb" data-testid="system-management-card">
        <h3>病毒库升级</h3>
        <p class="system-card-meta">当前: {{ virusDbVersion }} | 更新时间: {{ virusDbUpdatedAt }}</p>
        <div class="system-card-actions">
          <el-button
            type="primary"
            data-testid="virusdb-upgrade-upload"
            @click="uploadVirusdbUpgradePackage"
          >
            上传 .zip 升级包
          </el-button>
        </div>
      </section>

      <section class="system-card system-card-license" data-testid="system-management-card">
        <h3>授权信息管理</h3>
        <p class="auth-line">状态: <span>{{ authStatusText }}</span></p>
        <p class="system-card-meta">授权截止时间: {{ authExpireTime }}</p>
        <div class="system-card-actions">
          <el-button data-testid="machine-code-open" @click="openMachineCode">
            下载机器码
          </el-button>
          <el-button type="primary" data-testid="license-upload" @click="uploadLicenseFile">
            上传授权文件
          </el-button>
        </div>
      </section>

      <section class="system-card system-card-device-desc" data-testid="system-management-card">
        <h3>自定义设备描述</h3>
        <p class="system-card-meta device-description-line">
          当前: <code>{{ currentDeviceDescription }}</code>
        </p>
        <p class="device-description-warning">
          修改完成后需重启设备，此时设备上请勿连接USB设备。描述长度最大32位，仅字母数字下划线。
        </p>
        <el-button data-testid="device-desc-edit" @click="openDeviceDescriptionDialog">
          修改
        </el-button>
      </section>
    </div>

    <el-dialog v-model="machineCodeDialogVisible" title="机器码" width="520px">
      <div v-loading="machineCodeLoading" class="machine-code-dialog">
        <code class="app-code" data-testid="machine-code-text">{{ machineCode }}</code>
        <img v-if="qrcodeUrl" :src="qrcodeUrl" alt="机器码二维码">
      </div>
      <template #footer>
        <el-button @click="machineCodeDialogVisible = false">关闭</el-button>
        <el-button type="primary" data-testid="machine-code-save" @click="saveQrcode">
          保存二维码
        </el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="deviceDescriptionDialogVisible" title="修改自定义设备描述" width="480px">
      <div class="device-description-dialog">
        <label for="device-desc-input">自定义设备描述</label>
        <el-input
          id="device-desc-input"
          v-model="deviceDescription"
          maxlength="32"
          data-testid="device-desc-input"
          placeholder="请输入设备描述"
        />
        <p class="dialog-hint">描述长度最大32位，仅可以使用字母数字下划线，不能存在空格</p>
      </div>
      <template #footer>
        <el-button @click="deviceDescriptionDialogVisible = false">取消</el-button>
        <el-button
          type="primary"
          data-testid="device-desc-save"
          :loading="deviceDescriptionSaving"
          @click="saveDeviceDescription"
        >
          确定
        </el-button>
      </template>
    </el-dialog>

    <ProgressDialog
      :visible="transferVisible"
      :title="transferTitle"
      message="正在传输文件..."
    />
  </div>
</template>

<style scoped lang="scss">
@use '@/styles/tokens' as *;

.system-page {
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

.system-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}

.system-card {
  min-height: 172px;
  padding: 28px 36px;
  background: var(--andi-white);
  border: 1px solid var(--andi-border);
  border-radius: 6px;
  box-shadow: 0 1px 2px rgba(16, 24, 40, 0.04);

  h3 {
    margin: 0 0 12px;
    color: var(--andi-text);
    font-size: 18px;
    font-weight: $font-weight-semibold;
  }
}

.system-card-meta {
  margin: 0 0 16px;
  color: var(--andi-text-light);
  font-size: 15px;
  font-weight: $font-weight-medium;
}

.system-card-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.system-card-note {
  margin: 14px 0 0;
  color: var(--andi-text-light);
  font-size: 13px;
  font-weight: $font-weight-medium;
}

.auth-line {
  margin: 0 0 10px;
  color: var(--andi-text);
  font-size: 15px;
  font-weight: $font-weight-medium;

  span {
    color: #2e7d32;
    font-weight: $font-weight-semibold;
  }
}

.device-description-line {
  margin-bottom: 8px;

  code {
    padding: 2px 6px;
    color: var(--andi-blue-dark);
    font-family: 'SF Mono', Consolas, monospace;
    font-size: 15px;
    background: #f0f2f5;
    border-radius: 4px;
  }
}

.device-description-warning {
  max-width: 620px;
  margin: 8px 0 18px;
  padding: 10px 14px;
  color: #e65100;
  font-size: 13px;
  font-weight: $font-weight-semibold;
  line-height: 1.6;
  background: #fff8e1;
  border-radius: 6px;
}

.device-description-dialog {
  label {
    display: block;
    margin-bottom: 4px;
    color: var(--andi-text-light);
    font-size: 12px;
  }
}

.dialog-hint {
  margin: 4px 0 0;
  color: var(--andi-text-light);
  font-size: 11px;
}

.system-card :deep(.el-button) {
  min-height: 38px;
  padding: 8px 18px;
  font-size: 15px;
  font-weight: $font-weight-semibold;
  border-radius: 4px;
}

.machine-code-dialog {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-5;

  code {
    padding: $spacing-2 $spacing-3;
    color: var(--andi-blue-dark);
    background: var(--andi-sidebar);
    border-radius: $radius-control;
  }

  img {
    width: 180px;
    height: 180px;
    object-fit: contain;
  }
}

@media (max-width: 1100px) {
  .system-grid {
    grid-template-columns: 1fr;
  }
}
</style>
