<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { ElMessage } from 'element-plus'
import ConnectionAlert from '@/components/ConnectionAlert.vue'
import ProgressDialog from '@/components/ProgressDialog.vue'
import { exportPolicy, importPolicy } from '@/services/policy-service'
import { ServiceError } from '@/services/send-command'
import { useConnectionStore } from '@/stores/connection'
import { useFilePolicyStore } from '@/stores/file-policy'
import { useSessionStore } from '@/stores/session'
import { useWhitelistStore } from '@/stores/whitelist'
import { confirmAction } from '@/utils/confirm-action'
import { showErrorDialog, showSuccessToast } from '@/utils/operation-feedback'

const connection = useConnectionStore()
const session = useSessionStore()
const filePolicyStore = useFilePolicyStore()
const whitelistStore = useWhitelistStore()

const isTransferring = ref(false)
const progressTitle = ref('')
const progressMessage = ref('')
let operationEpoch = 0
let operationOwner: symbol | null = null
let isComponentActive = true

interface PolicyOperation {
  owner: symbol
  epoch: number
  sessionToken: string
  hasWarnedConnection: boolean
}

interface PendingFailure {
  error: unknown
  fallback: string
  allowServiceMessage: boolean
}

onBeforeUnmount(() => {
  isComponentActive = false
  operationEpoch += 1
  operationOwner = null
})

function beginOperation(): PolicyOperation | null {
  if (operationOwner != null) {
    return null
  }
  const owner = Symbol('policy-transfer')
  operationOwner = owner
  isTransferring.value = true
  return {
    owner,
    epoch: operationEpoch,
    sessionToken: session.token,
    hasWarnedConnection: false,
  }
}

function ownsOperation(operation: PolicyOperation): boolean {
  return (
    isComponentActive &&
    operationOwner === operation.owner &&
    operationEpoch === operation.epoch
  )
}

function hasSameSession(operation: PolicyOperation): boolean {
  return ownsOperation(operation) && session.token === operation.sessionToken
}

function finishOperation(operation: PolicyOperation): void {
  if (operationOwner === operation.owner) {
    operationOwner = null
    isTransferring.value = false
    progressTitle.value = ''
    progressMessage.value = ''
  }
}

function canContinue(operation: PolicyOperation): boolean {
  if (!hasSameSession(operation)) {
    return false
  }
  if (connection.isConnected) {
    return true
  }
  warnDisconnected(operation)
  return false
}

function warnDisconnected(operation: PolicyOperation): void {
  if (!hasSameSession(operation) || connection.isConnected || operation.hasWarnedConnection) {
    return
  }
  operation.hasWarnedConnection = true
  ElMessage.warning('装置已断开连接，无法传输策略')
}

function formatPart(value: number): string {
  return String(value).padStart(2, '0')
}

function defaultPolicyFileName(now = new Date()): string {
  const date = `${now.getFullYear()}${formatPart(now.getMonth() + 1)}${formatPart(now.getDate())}`
  const time = `${formatPart(now.getHours())}${formatPart(now.getMinutes())}${formatPart(now.getSeconds())}`
  return `安全策略-${date}-${time}.bin`
}

async function reportOperationFailure(
  operation: PolicyOperation,
  failure: PendingFailure,
): Promise<void> {
  if (!hasSameSession(operation)) {
    return
  }
  if (!connection.isConnected) {
    warnDisconnected(operation)
    return
  }
  await showErrorDialog(
    failure.fallback,
    failure.allowServiceMessage && failure.error instanceof ServiceError
      ? failure.error.message
      : failure.fallback,
  )
}

async function revokeSelectedFileAccess(filePath: string): Promise<void> {
  try {
    await window.desktopApi.dialog.revokeFileAccess(filePath)
  } catch {
    // 主错误优先；撤销失败不暴露本地路径。
  }
}

async function handleExport(): Promise<void> {
  const operation = beginOperation()
  if (operation == null) {
    return
  }

  let selectedPath = ''
  let writeCompleted = false
  let accessRevoked = false
  try {
    if (!canContinue(operation)) {
      return
    }
    let result: Awaited<ReturnType<typeof window.desktopApi.dialog.saveFile>>
    try {
      result = await window.desktopApi.dialog.saveFile({
        title: '导出安全策略',
        defaultPath: defaultPolicyFileName(),
        filters: [{ name: '安全策略', extensions: ['bin'] }],
      })
    } catch (error: unknown) {
      await reportOperationFailure(operation, {
        error,
        fallback: '无法打开策略保存对话框',
        allowServiceMessage: false,
      })
      return
    }
    if (result.canceled || result.filePath == null) {
      return
    }
    selectedPath = result.filePath
    if (!hasSameSession(operation)) {
      await revokeSelectedFileAccess(selectedPath)
      accessRevoked = true
      return
    }
    if (!canContinue(operation)) {
      await revokeSelectedFileAccess(selectedPath)
      accessRevoked = true
      return
    }

    progressTitle.value = '导出策略'
    progressMessage.value = '正在从装置导出并保存策略...'
    let response: Awaited<ReturnType<typeof exportPolicy>>
    try {
      response = await exportPolicy(operation.sessionToken)
    } catch (error: unknown) {
      await revokeSelectedFileAccess(selectedPath)
      accessRevoked = true
      await reportOperationFailure(operation, {
        error,
        fallback: '策略导出失败',
        allowServiceMessage: true,
      })
      return
    }
    if (!canContinue(operation)) {
      return
    }
    try {
      await window.desktopApi.dialog.writeFile(selectedPath, response.policyData)
      writeCompleted = true
      if (canContinue(operation)) {
        showSuccessToast('策略导出成功')
      }
    } catch (error: unknown) {
      await revokeSelectedFileAccess(selectedPath)
      accessRevoked = true
      await reportOperationFailure(operation, {
        error,
        fallback: '策略导出失败',
        allowServiceMessage: false,
      })
    }
  } finally {
    if (selectedPath !== '' && !writeCompleted && !accessRevoked) {
      await revokeSelectedFileAccess(selectedPath)
    }
    finishOperation(operation)
  }
}

async function handleImport(): Promise<void> {
  const operation = beginOperation()
  if (operation == null) {
    return
  }

  let selectedPath = ''
  let pendingFailure: PendingFailure | null = null
  try {
    if (!canContinue(operation)) {
      return
    }
    let result: Awaited<ReturnType<typeof window.desktopApi.dialog.openFile>>
    try {
      result = await window.desktopApi.dialog.openFile({
        title: '导入安全策略',
        filters: [{ name: '加密策略文件', extensions: ['bin'] }],
      })
    } catch (error: unknown) {
      pendingFailure = {
        error,
        fallback: '无法打开策略文件选择对话框',
        allowServiceMessage: false,
      }
      return
    }
    if (result.canceled || result.filePaths.length === 0) {
      return
    }
    selectedPath = result.filePaths[0]
    if (!canContinue(operation)) {
      return
    }

    if (!selectedPath.toLowerCase().endsWith('.bin')) {
      await showErrorDialog('策略文件格式错误', '仅支持 .bin 策略文件')
      return
    }

    try {
      await confirmAction({
        message: '导入将整体覆盖当前装置的 U 盘白名单和文件访问策略，是否继续？',
        title: '导入策略确认',
        confirmButtonText: '导入',
        type: 'warning',
      })
    } catch {
      return
    }
    if (!canContinue(operation)) {
      return
    }

    progressTitle.value = '导入策略'
    progressMessage.value = '正在读取并导入策略...'
    let policyData: Uint8Array
    try {
      policyData = await window.desktopApi.dialog.readFile(selectedPath)
    } catch (error: unknown) {
      pendingFailure = {
        error,
        fallback: '策略文件读取失败',
        allowServiceMessage: false,
      }
      return
    }
    if (!canContinue(operation)) {
      return
    }

    try {
      await importPolicy(operation.sessionToken, policyData)
    } catch (error: unknown) {
      pendingFailure = {
        error,
        fallback: '策略导入失败',
        allowServiceMessage: true,
      }
      return
    }
    if (!hasSameSession(operation)) {
      return
    }
    if (!connection.isConnected) {
      await showErrorDialog('策略状态刷新失败', '策略已导入，但状态刷新失败，请稍后重试')
      return
    }

    progressMessage.value = '策略已导入，正在刷新当前状态...'
    const refreshResults = await Promise.allSettled([
      filePolicyStore.load(operation.sessionToken),
      whitelistStore.listWhitelist(operation.sessionToken),
    ])
    if (!hasSameSession(operation)) {
      return
    }
    if (
      !connection.isConnected ||
      refreshResults.some((refreshResult) => refreshResult.status === 'rejected')
    ) {
      await showErrorDialog('策略状态刷新失败', '策略已导入，但状态刷新失败，请稍后重试')
      return
    }
    showSuccessToast('导入成功，重新拔插或重新映射后生效')
  } finally {
    if (selectedPath !== '') {
      await revokeSelectedFileAccess(selectedPath)
    }
    if (pendingFailure != null) {
      await reportOperationFailure(operation, pendingFailure)
    }
    finishOperation(operation)
  }
}
</script>

<template>
  <div class="policies-page app-page">
    <header class="page-header app-page-header">
      <div>
        <h1 class="app-page-title">策略管理</h1>
        <p class="app-page-desc">导入和导出安全策略配置</p>
      </div>
    </header>
    <ConnectionAlert />

    <div class="policy-transfer-grid" data-testid="policy-transfer-grid">
      <section class="policy-transfer-card" data-testid="policy-export-card">
        <h3>导出策略</h3>
        <p>将当前装置策略备份为加密 .bin 文件。</p>
        <el-button
          type="primary"
          data-testid="export-policy"
          :loading="isTransferring"
          :disabled="isTransferring"
          @click="handleExport"
        >
          导出策略
        </el-button>
      </section>

      <section class="policy-transfer-card" data-testid="policy-import-card">
        <h3>导入策略</h3>
        <p>整体覆盖当前白名单和文件访问策略，仅支持 .bin 文件。</p>
        <el-button
          type="primary"
          data-testid="import-policy"
          :loading="isTransferring"
          :disabled="isTransferring"
          @click="handleImport"
        >
          导入策略
        </el-button>
      </section>
    </div>

    <ProgressDialog
      :visible="isTransferring && progressTitle !== ''"
      :title="progressTitle || '策略传输'"
      :message="progressMessage"
    />
  </div>
</template>

<style scoped lang="scss">
.policy-transfer-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 24px;
}

.policy-transfer-card {
  min-height: 220px;
  padding: 36px 42px;
  background: $bg-white;
  border: $border-width solid $border-color;
  border-radius: 6px;

  h3 {
    margin: 0 0 18px;
    color: $text-primary;
    font-size: 24px;
    font-weight: $font-weight-semibold;
  }

  p {
    min-height: 48px;
    margin: 0 0 48px;
    color: $text-secondary;
    font-size: 16px;
    line-height: 1.6;
  }
}

@media (max-width: 900px) {
  .policy-transfer-grid {
    grid-template-columns: 1fr;
  }
}
</style>
