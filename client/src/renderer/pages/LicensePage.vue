<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Close, FullScreen, Minus } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { getMachineCode, uploadLicense } from '@/services/auth-service'
import { useConnectionStore } from '@/stores/connection'
import { useSessionStore } from '@/stores/session'

const session = useSessionStore()
const connection = useConnectionStore()
const router = useRouter()

const machineCodeDialogVisible = ref(false)
const loadingMachineCode = ref(false)
const machineCode = ref('')
const qrcodePng = ref<Uint8Array>(new Uint8Array())
const qrcodeUrl = ref('')

const uploadDialogVisible = ref(false)
const uploadingLicense = ref(false)
const selectedLicensePath = ref('')
const uploadError = ref('')

const isExpired = computed(() => connection.status === 'LICENSE_EXPIRED')
const selectedLicenseName = computed(() => {
  return selectedLicensePath.value.split(/[\\/]/).pop() ?? ''
})

function handleMinimize(): void {
  void window.desktopApi.window.minimize()
}

function handleMaximize(): void {
  void window.desktopApi.window.maximize()
}

function handleClose(): void {
  void window.desktopApi.window.close()
}

function replaceQrcodeUrl(bytes: Uint8Array): void {
  if (qrcodeUrl.value !== '') {
    URL.revokeObjectURL(qrcodeUrl.value)
  }
  const blob = new Blob([bytes.slice().buffer], { type: 'image/png' })
  qrcodeUrl.value = URL.createObjectURL(blob)
}

async function handleOpenMachineCode(): Promise<void> {
  if (loadingMachineCode.value) {
    return
  }

  machineCodeDialogVisible.value = true
  loadingMachineCode.value = true
  machineCode.value = ''
  qrcodePng.value = new Uint8Array()

  try {
    const response = await getMachineCode(session.token)
    machineCode.value = response.machineCode
    qrcodePng.value = new Uint8Array(response.qrcodePng)
    replaceQrcodeUrl(qrcodePng.value)
  } catch (error: unknown) {
    machineCodeDialogVisible.value = false
    ElMessage.error(error instanceof Error ? error.message : '获取机器码失败')
  } finally {
    loadingMachineCode.value = false
  }
}

async function handleCopyMachineCode(): Promise<void> {
  if (machineCode.value === '') {
    return
  }
  try {
    await navigator.clipboard.writeText(machineCode.value)
    ElMessage.success('机器码已复制')
  } catch {
    ElMessage.warning('复制失败，请手动复制')
  }
}

async function handleDownloadQrcode(): Promise<void> {
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
    ElMessage.success('二维码图片已下载')
  } catch (error: unknown) {
    ElMessage.error(error instanceof Error ? error.message : '二维码图片保存失败')
  }
}

function handleOpenUploadDialog(): void {
  selectedLicensePath.value = ''
  uploadError.value = ''
  uploadDialogVisible.value = true
}

async function handleSelectLicenseFile(): Promise<void> {
  const result = await window.desktopApi.dialog.openFile({
    title: '选择授权文件',
    filters: [{ name: '授权文件', extensions: ['txt'] }],
  })
  if (result.canceled || result.filePaths.length === 0) {
    return
  }

  selectedLicensePath.value = result.filePaths[0]
  uploadError.value = ''
}

async function finishLicenseFlow(): Promise<void> {
  try {
    await connection.applyStateEvent('LICENSE_UPLOAD_SUCCESS')
  } catch {
    // 网络事件可能已先将状态推进到 DISCONNECTED，仍需完成本地清理。
  }
  session.clearSession()
  await connection.disconnect(true).catch(() => {})
  await router.push('/login')
}

async function handleConfirmUpload(): Promise<void> {
  if (uploadingLicense.value) {
    return
  }
  if (selectedLicensePath.value === '') {
    uploadError.value = '请先选择授权文件'
    return
  }
  if (!selectedLicensePath.value.toLowerCase().endsWith('.txt')) {
    uploadError.value = '仅支持 .txt 格式授权文件'
    return
  }

  uploadingLicense.value = true
  uploadError.value = ''
  const selectedPath = selectedLicensePath.value

  try {
    const fileData = await window.desktopApi.dialog.readFile(selectedPath)
    selectedLicensePath.value = ''
    await uploadLicense(session.token, fileData)
    uploadDialogVisible.value = false
    ElMessage.success('授权成功！自动返回登录页')
    await finishLicenseFlow()
  } catch (error: unknown) {
    selectedLicensePath.value = ''
    uploadError.value = error instanceof Error ? error.message : '授权失败'
  } finally {
    uploadingLicense.value = false
  }
}

onUnmounted(() => {
  if (qrcodeUrl.value !== '') {
    URL.revokeObjectURL(qrcodeUrl.value)
  }
})
</script>

<template>
  <div class="license-page auth-page">
    <div class="win-controls">
      <button class="win-btn" title="最小化" @click="handleMinimize">
        <el-icon><Minus /></el-icon>
      </button>
      <button class="win-btn" title="最大化" @click="handleMaximize">
        <el-icon><FullScreen /></el-icon>
      </button>
      <button class="win-btn win-btn-close" title="关闭" @click="handleClose">
        <el-icon><Close /></el-icon>
      </button>
    </div>

    <section class="license-card auth-card app-card" aria-labelledby="license-title">
      <div class="license-logo" aria-hidden="true">
        <svg viewBox="0 0 16 16" role="img">
          <path d="M8 1 3 4v4c0 3.3 2.2 6.2 5 7 2.8-.8 5-3.7 5-7V4L8 1Z" />
        </svg>
      </div>
      <h1 id="license-title" class="auth-title">{{ isExpired ? '授权已到期' : '设备授权' }}</h1>
      <p class="license-description">
        {{
          isExpired
            ? '当前装置授权已到期，请上传新的授权文件后重新登录。'
            : '请先完成设备授权后再登录。'
        }}
      </p>

      <div class="license-actions">
        <el-button
          type="primary"
          data-testid="machine-code-open"
          :loading="loadingMachineCode"
          @click="handleOpenMachineCode"
        >
          下载机器码
        </el-button>
        <el-button
          data-testid="license-upload-open"
          :disabled="uploadingLicense"
          @click="handleOpenUploadDialog"
        >
          上传授权文件
        </el-button>
      </div>
      <p class="license-hint">授权成功后自动返回登录页</p>
    </section>

    <el-dialog
      v-model="machineCodeDialogVisible"
      title="机器码"
      width="480px"
      :close-on-click-modal="false"
    >
      <div v-loading="loadingMachineCode" class="machine-code-content">
        <p>机器码（可复制文本）</p>
        <el-input :model-value="machineCode" type="textarea" :rows="3" readonly />
        <img v-if="qrcodeUrl" class="qrcode" :src="qrcodeUrl" alt="机器码二维码" />
      </div>
      <template #footer>
        <el-button :disabled="machineCode === ''" @click="handleCopyMachineCode">
          复制机器码
        </el-button>
        <el-button
          type="primary"
          :disabled="qrcodePng.length === 0"
          @click="handleDownloadQrcode"
        >
          下载二维码
        </el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="uploadDialogVisible"
      title="上传授权文件"
      width="440px"
      :close-on-click-modal="false"
      :close-on-press-escape="!uploadingLicense"
      :show-close="!uploadingLicense"
    >
      <button
        class="file-selector"
        type="button"
        :disabled="uploadingLicense"
        @click="handleSelectLicenseFile"
      >
        <span>点击选择授权文件</span>
        <small>支持 .txt 格式授权文件</small>
      </button>
      <el-input
        :model-value="selectedLicenseName"
        placeholder="已选择文件：无"
        readonly
      />
      <el-alert
        v-if="uploadError"
        class="upload-error"
        :title="uploadError"
        type="error"
        show-icon
        :closable="false"
      />
      <template #footer>
        <el-button :disabled="uploadingLicense" @click="uploadDialogVisible = false">
          取消
        </el-button>
        <el-button
          type="primary"
          data-testid="license-upload-confirm"
          :loading="uploadingLicense"
          :disabled="uploadingLicense"
          @click="handleConfirmUpload"
        >
          确定上传
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
@use '@/styles/tokens' as *;

.license-page {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  overflow: hidden;
  color: var(--andi-text);
  font-family: var(--andi-font-family);
  background: linear-gradient(135deg, var(--andi-page-bg), #f5f7fa);
  user-select: none;
  -webkit-app-region: drag;
}

.win-controls {
  position: absolute;
  z-index: 10;
  top: 0;
  right: 0;
  display: flex;
  -webkit-app-region: no-drag;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 28px;
  color: var(--andi-window-control-light-color);
  font-size: $font-size-body;
  background: transparent;
  border: none;
  cursor: pointer;

  &:hover {
    background: var(--andi-window-control-light-hover-bg);
  }
}

.win-btn-close:hover {
  color: var(--andi-white);
  background: var(--andi-window-control-close-hover-bg);
}

.license-card {
  width: 400px;
  padding: 40px 32px;
  text-align: center;
  background: var(--andi-white);
  border-radius: var(--andi-radius-window);
  box-shadow: var(--andi-shadow-window);
  -webkit-app-region: no-drag;

  h1 {
    margin: 0 0 $spacing-1;
    color: var(--andi-text);
    font-size: 17px;
    font-weight: 700;
  }
}

.license-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 52px;
  height: 52px;
  margin: 0 auto $spacing-6;
  color: var(--andi-blue);
  background: var(--andi-blue-light);
  border-radius: 10px;

  svg {
    width: 24px;
    height: 24px;
    fill: currentcolor;
  }
}

.license-description {
  margin: 0 0 $spacing-8;
  color: var(--andi-text-secondary);
  font-size: $font-size-body;
  line-height: $line-height-base;
}

.license-actions {
  display: flex;
  flex-direction: column;
  gap: $spacing-5;

  :deep(.el-button) {
    width: 100%;
    margin-left: 0;
    padding: $spacing-5;
  }
}

.license-hint {
  margin: $spacing-7 0 0;
  color: var(--andi-text-secondary);
  font-size: $font-size-sm;
}

.machine-code-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-5;

  p {
    align-self: flex-start;
    margin: 0;
    color: var(--andi-text-light);
    font-size: $font-size-body;
  }
}

.qrcode {
  width: 200px;
  height: 200px;
  object-fit: contain;
  border: $border-width-base solid var(--andi-border);
  border-radius: $radius-card;
}

.file-selector {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  margin-bottom: $spacing-5;
  padding: $spacing-8;
  color: var(--andi-text-secondary);
  background: var(--andi-white);
  border: 1px dashed var(--andi-border);
  border-radius: $radius-card;
  cursor: pointer;
  gap: $spacing-2;

  &:hover:not(:disabled) {
    color: var(--andi-blue);
    border-color: var(--andi-blue);
  }

  small {
    color: var(--andi-text-light);
  }
}

.upload-error {
  margin-top: $spacing-5;
}
</style>
