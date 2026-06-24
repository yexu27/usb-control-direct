<script setup lang="ts">
import { Check } from '@element-plus/icons-vue'
import { useAppToastStore } from '@/stores/app-toast'

const toast = useAppToastStore()
</script>

<template>
  <transition name="app-toast">
    <div
      v-if="toast.visible"
      class="app-toast-host app-toast-success"
      data-testid="app-toast"
      role="status"
      aria-live="polite"
    >
      <el-icon class="app-toast-icon" aria-hidden="true">
        <Check />
      </el-icon>
      <span class="app-toast-message">{{ toast.message }}</span>
    </div>
  </transition>
</template>

<style scoped lang="scss">
.app-toast-host {
  position: fixed;
  top: 72px;
  left: 50%;
  z-index: 3000;
  display: inline-flex;
  align-items: center;
  max-width: min(560px, calc(100vw - 96px));
  min-height: 40px;
  padding: 9px 18px 9px 14px;
  border: 1px solid #9bd7ad;
  border-radius: 6px;
  box-shadow: 0 10px 28px rgba(18, 56, 30, 0.14);
  transform: translateX(-50%);
}

.app-toast-success {
  color: #1b6b35;
  background: #edf8f0;
}

.app-toast-icon {
  flex: 0 0 auto;
  width: 18px;
  height: 18px;
  margin-right: 8px;
  color: #1f8d46;
}

.app-toast-message {
  overflow: hidden;
  color: #1b6b35;
  font-size: 14px;
  font-weight: 500;
  line-height: 20px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-toast-enter-active,
.app-toast-leave-active {
  transition: opacity 160ms ease, transform 160ms ease;
}

.app-toast-enter-from,
.app-toast-leave-to {
  opacity: 0;
  transform: translate(-50%, -8px);
}
</style>
