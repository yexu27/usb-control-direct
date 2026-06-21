<script setup lang="ts">
import { useConnectionStore } from '@/stores/connection'

const connection = useConnectionStore()
</script>

<template>
  <el-alert
    v-if="connection.wasConnected && connection.isDisconnected"
    title="USB 管控装置已断开连接，请检查网络或设备连接。"
    type="error"
    :closable="false"
    show-icon
    class="connection-alert"
  />
</template>

<style scoped lang="scss">
.connection-alert {
  margin-bottom: $spacing-5;
}
</style>
