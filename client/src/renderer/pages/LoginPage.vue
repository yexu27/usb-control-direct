<script setup lang="ts">
import { Minus, FullScreen, Close } from '@element-plus/icons-vue'

function handleMinimize(): void {
  window.desktopApi.window.minimize()
}

function handleMaximize(): void {
  window.desktopApi.window.maximize()
}

function handleClose(): void {
  window.desktopApi.window.close()
}
</script>

<template>
  <div class="login-page">
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

    登录页（占位）
  </div>
</template>

<style scoped lang="scss">
.login-page {
  height: 100vh;
  overflow: hidden;
  position: relative;
}

.win-controls {
  position: absolute;
  top: 0;
  right: 0;
  display: flex;
  -webkit-app-region: no-drag;
  z-index: 10;
}

.win-btn {
  width: 36px;
  height: 28px;
  border: none;
  background: transparent;
  color: $text-secondary;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;

  &:hover {
    background: rgba(0, 0, 0, 0.06);
  }
}

.win-btn-close:hover {
  background: #e81123;
  color: #fff;
}
</style>
