<script setup lang="ts">
import { computed, onUnmounted, ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
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
import type { usb_control } from '../../shared/proto/usb_control'

const session = useSessionStore()
const connection = useConnectionStore()

const systemInfo = ref<usb_control.RspSystemInfo | null>(null)
const isLoadingInfo = ref(false)
const systemPackagePath = ref('')
const systemTargetVersion = ref('')
const virusdbPackagePath = ref('')
const virusdbTargetVersion = ref('')
const systemUpgrading = ref(false)
const virusdbUpgrading = ref(false)
const licenseUploading = ref(false)
const machineCodeDialogVisible = ref(false)
const machineCodeLoading = ref(false)
const machineCode = ref('')
const qrcodePng = ref<Uint8Array>(new Uint8Array())
const qrcodeUrl = ref('')
const deviceDescription = ref('')
const deviceDescriptionSaving = ref(false)

const systemVersion = computed(() => systemInfo.value?.systemVersion ?? '-')
const virusDbVersion = computed(() => systemInfo.value?.virusDbVersion ?? '-')
const virusDbUpdatedAt = computed(() => formatUnixSeconds(systemInfo.value?.virusDbUpdatedAt ?? 0))
const authExpireTime = computed(() => formatUnixSeconds(systemInfo.value?.authExpireTime ?? 0))
const authStatusText = computed(() => {
  return formatAuthStatus(systemInfo.value?.authStatus ?? '', systemInfo.value?.authorized ?? false)
})
const currentDeviceDescription = computed(() => systemInfo.value?.deviceDescription ?? '-')
const selectedSystemPackageName = computed(() => basename(systemPackagePath.value))
const selectedVirusdbPackageName = computed(() => basename(virusdbPackagePath.value))
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

onMounted(() => {
  void loadSystemInfo()
})

onUnmounted(() => {
  if (qrcodeUrl.value !== '') {
    URL.revokeObjectURL(qrcodeUrl.value)
  }
})

function basename(filePath: string): string {
  return filePath.split(/[\\/]/).pop() ?? filePath
}

function formatAuthStatus(status: string, authorized: boolean): string {
  if (status === 'expired') {
    return '已到期'
  }
  if (status === 'unauthorized') {
    return '未授权'
  }
  return authorized ? '已授权' : '未授权'
}

function showError(error: unknown, fallback: string): void {
  ElMessage.error(error instanceof Error ? error.message : fallback)
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
    showError(error, '系统信息加载失败')
  } finally {
    isLoadingInfo.value = false
  }
}

async function selectSystemPackage(): Promise<void> {
  const result = await window.desktopApi.dialog.openFile({
    title: '选择系统升级包',
    filters: [{ name: '系统升级包', extensions: ['bin'] }],
  })
  if (result.canceled || result.filePaths.length === 0) {
    return
  }
  const filePath = result.filePaths[0]
  if (!filePath.toLowerCase().endsWith('.bin')) {
    ElMessage.error('仅支持 .bin 系统升级包')
    return
  }
  try {
    systemTargetVersion.value = parseSystemUpgradeVersion(filePath)
    systemPackagePath.value = filePath
  } catch (error: unknown) {
    showError(error, '系统升级包文件名格式错误')
  }
}

async function runSystemUpgrade(): Promise<void> {
  if (systemUpgrading.value || systemPackagePath.value === '' || !canOperate()) {
    return
  }
  systemUpgrading.value = true
  try {
    const fileData = await window.desktopApi.dialog.readFile(systemPackagePath.value)
    const checksum = await calculateSha256Hex(fileData)
    await uploadSystemUpgrade(session.token, fileData, systemTargetVersion.value, checksum)
    ElMessage.success('升级完成，请重新连接')
    await connection.disconnect(true).catch(() => {})
  } catch (error: unknown) {
    showError(error, '系统升级失败')
  } finally {
    systemUpgrading.value = false
  }
}

async function selectVirusdbPackage(): Promise<void> {
  const result = await window.desktopApi.dialog.openFile({
    title: '选择病毒库升级包',
    filters: [{ name: '病毒库升级包', extensions: ['zip'] }],
  })
  if (result.canceled || result.filePaths.length === 0) {
    return
  }
  const filePath = result.filePaths[0]
  if (!filePath.toLowerCase().endsWith('.zip')) {
    ElMessage.error('仅支持 .zip 病毒库升级包')
    return
  }
  try {
    virusdbTargetVersion.value = parseVirusdbUpgradeVersion(filePath)
    virusdbPackagePath.value = filePath
  } catch (error: unknown) {
    showError(error, '病毒库升级包文件名格式错误')
  }
}

async function runVirusdbUpgrade(): Promise<void> {
  if (virusdbUpgrading.value || virusdbPackagePath.value === '' || !canOperate()) {
    return
  }
  virusdbUpgrading.value = true
  try {
    const fileData = await window.desktopApi.dialog.readFile(virusdbPackagePath.value)
    const checksum = await calculateSha256Hex(fileData)
    await uploadVirusdbUpgrade(session.token, fileData, virusdbTargetVersion.value, checksum)
    ElMessage.success('病毒库升级成功')
    await loadSystemInfo()
  } catch (error: unknown) {
    showError(error, '病毒库升级失败')
  } finally {
    virusdbUpgrading.value = false
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
    showError(error, '获取机器码失败')
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
    ElMessage.success('机器码二维码已保存')
  } catch (error: unknown) {
    showError(error, '机器码二维码保存失败')
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
    ElMessage.error('仅支持 .txt 格式授权文件')
    return
  }
  licenseUploading.value = true
  try {
    const fileData = await window.desktopApi.dialog.readFile(filePath)
    await uploadLicense(session.token, fileData)
    ElMessage.success('授权文件上传成功')
    await loadSystemInfo()
  } catch (error: unknown) {
    showError(error, '授权文件上传失败')
  } finally {
    licenseUploading.value = false
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

async function saveDeviceDescription(): Promise<void> {
  if (deviceDescriptionSaving.value || !canOperate()) {
    return
  }
  const nextDescription = deviceDescription.value.trim()
  const validationMessage = validateDeviceDescription(nextDescription)
  if (validationMessage !== '') {
    ElMessage.error(validationMessage)
    return
  }
  try {
    await ElMessageBox.confirm(
      '修改设备描述前，请确认当前装置未连接移动存储设备、键盘、鼠标等 USB 设备。修改完成后需重启 USB 管控装置才能生效。',
      '确认修改设备描述',
      { confirmButtonText: '确认修改', cancelButtonText: '取消', type: 'warning' },
    )
  } catch {
    return
  }
  deviceDescriptionSaving.value = true
  try {
    await updateDeviceDescription(session.token, nextDescription)
    ElMessage.success('修改成功，重启设备后生效')
    await loadSystemInfo()
  } catch (error: unknown) {
    showError(error, '设备描述修改失败')
  } finally {
    deviceDescriptionSaving.value = false
  }
}
</script>

<template>
  <div class="system-page">
    <header class="page-header">
      <h1>系统管理</h1>
      <p>系统升级、授权管理和设备配置</p>
    </header>
    <ConnectionAlert />

    <div v-loading="isLoadingInfo" class="system-grid">
      <el-card shadow="never" class="system-card">
        <template #header>系统信息</template>
        <dl class="info-list">
          <dt>当前系统版本</dt><dd>{{ systemVersion }}</dd>
          <dt>病毒库版本</dt><dd>{{ virusDbVersion }}</dd>
          <dt>病毒库更新时间</dt><dd>{{ virusDbUpdatedAt }}</dd>
          <dt>授权状态</dt><dd>{{ authStatusText }}</dd>
          <dt>授权截止时间</dt><dd>{{ authExpireTime }}</dd>
          <dt>当前设备描述</dt><dd>{{ currentDeviceDescription }}</dd>
        </dl>
      </el-card>

      <el-card shadow="never" class="system-card">
        <template #header>系统升级</template>
        <p>当前版本：{{ systemVersion }}</p>
        <div class="action-row">
          <el-button data-testid="system-upgrade-select" @click="selectSystemPackage">
            选择系统升级包
          </el-button>
          <el-button
            type="primary"
            data-testid="system-upgrade-submit"
            :loading="systemUpgrading"
            :disabled="systemPackagePath === ''"
            @click="runSystemUpgrade"
          >
            升级
          </el-button>
        </div>
        <p v-if="systemPackagePath" class="selected-file">
          {{ selectedSystemPackageName }}，目标版本 {{ systemTargetVersion }}
        </p>
      </el-card>

      <el-card shadow="never" class="system-card">
        <template #header>病毒库升级</template>
        <p>当前版本：{{ virusDbVersion }}，更新时间：{{ virusDbUpdatedAt }}</p>
        <div class="action-row">
          <el-button data-testid="virusdb-upgrade-select" @click="selectVirusdbPackage">
            选择病毒库升级包
          </el-button>
          <el-button
            type="primary"
            data-testid="virusdb-upgrade-submit"
            :loading="virusdbUpgrading"
            :disabled="virusdbPackagePath === ''"
            @click="runVirusdbUpgrade"
          >
            升级
          </el-button>
        </div>
        <p v-if="virusdbPackagePath" class="selected-file">
          {{ selectedVirusdbPackageName }}，目标版本 {{ virusdbTargetVersion }}
        </p>
      </el-card>

      <el-card shadow="never" class="system-card">
        <template #header>授权信息管理</template>
        <p>状态：{{ authStatusText }}，截止时间：{{ authExpireTime }}</p>
        <div class="action-row">
          <el-button data-testid="machine-code-open" @click="openMachineCode">
            下载机器码
          </el-button>
          <el-button type="primary" data-testid="license-upload" @click="uploadLicenseFile">
            上传授权文件
          </el-button>
        </div>
      </el-card>

      <el-card shadow="never" class="system-card">
        <template #header>自定义设备描述</template>
        <p>当前描述：{{ currentDeviceDescription }}</p>
        <div class="device-desc-row">
          <el-input
            v-model="deviceDescription"
            maxlength="32"
            data-testid="device-desc-input"
            placeholder="请输入设备描述"
          />
          <el-button
            type="primary"
            data-testid="device-desc-save"
            :loading="deviceDescriptionSaving"
            @click="saveDeviceDescription"
          >
            保存
          </el-button>
        </div>
        <p class="hint">仅支持 1-32 位字母、数字或下划线，修改成功后重启设备生效。</p>
      </el-card>
    </div>

    <el-dialog v-model="machineCodeDialogVisible" title="机器码" width="520px">
      <div v-loading="machineCodeLoading" class="machine-code-dialog">
        <code data-testid="machine-code-text">{{ machineCode }}</code>
        <img v-if="qrcodeUrl" :src="qrcodeUrl" alt="机器码二维码">
      </div>
      <template #footer>
        <el-button @click="machineCodeDialogVisible = false">关闭</el-button>
        <el-button type="primary" data-testid="machine-code-save" @click="saveQrcode">
          保存二维码
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
.system-page {
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

.system-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $spacing-5;
}

.system-card {
  border-color: $border-color;
}

.info-list {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr);
  gap: $spacing-3 $spacing-5;
  margin: 0;

  dt {
    color: $text-secondary;
  }

  dd {
    min-width: 0;
    margin: 0;
    color: $text-primary;
    word-break: break-all;
  }
}

.action-row,
.device-desc-row {
  display: flex;
  align-items: center;
  gap: $spacing-3;
}

.device-desc-row {
  margin-top: $spacing-3;
}

.selected-file,
.hint {
  margin: $spacing-3 0 0;
  color: $text-secondary;
}

.machine-code-dialog {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-5;

  code {
    padding: $spacing-2 $spacing-3;
    color: $brand-primary-dark;
    background: $bg-sidebar;
    border-radius: $border-radius;
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
