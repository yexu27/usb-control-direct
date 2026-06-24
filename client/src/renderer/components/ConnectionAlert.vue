<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { useRouter } from 'vue-router'
import { useConnectionStore } from '@/stores/connection'
import { useSessionStore } from '@/stores/session'

const connection = useConnectionStore()
const session = useSessionStore()
const router = useRouter()
const isReconnecting = ref(false)

async function handleReconnect(): Promise<void> {
  if (isReconnecting.value) {
    return
  }
  isReconnecting.value = true
  try {
    const isValidSession = await session.reconnectAndValidate()
    if (!isValidSession) {
      await router.push('/login')
      return
    }
    ElMessage.success('USB 管控装置重新连接成功')
  } catch (error: unknown) {
    ElMessage.error(error instanceof Error ? error.message : 'USB 管控装置重新连接失败')
  } finally {
    isReconnecting.value = false
  }
}
</script>

<template>
  <el-alert
    v-if="connection.wasConnected && connection.isDisconnected"
    title="USB 管控装置已断开连接，请检查网络或设备连接。"
    type="error"
    :closable="false"
    show-icon
    class="connection-alert"
  >
    <div class="connection-alert-actions">
      <span>恢复连接后会重新校验当前会话并刷新页面数据。</span>
      <el-button
        size="small"
        type="primary"
        :loading="isReconnecting"
        :disabled="isReconnecting"
        data-testid="connection-reconnect"
        @click="handleReconnect"
      >
        重新连接
      </el-button>
    </div>
  </el-alert>
</template>

<style scoped lang="scss">
@use '@/styles/tokens' as *;

.connection-alert {
  margin-bottom: $spacing-5;
}

.connection-alert-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $spacing-4;
}
</style>
