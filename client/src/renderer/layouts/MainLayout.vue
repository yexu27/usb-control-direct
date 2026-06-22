<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { User, Minus, FullScreen, Close } from '@element-plus/icons-vue'
import { useSessionStore } from '@/stores/session'
import { useConnectionStore } from '@/stores/connection'
import ChangePasswordDialog from '@/components/ChangePasswordDialog.vue'
import type { UserRole } from '../../shared/connection-state'

interface MenuItem {
  label: string
  path: string
  roles: UserRole[]
}

const ALL_MENUS: MenuItem[] = [
  { label: '文件访问控制', path: '/file-access', roles: ['operator'] },
  { label: 'U盘设备控制', path: '/usb-devices', roles: ['operator'] },
  { label: '策略管理', path: '/policies', roles: ['operator'] },
  { label: '日志管理', path: '/logs', roles: ['auditor'] },
  { label: '系统管理', path: '/system', roles: ['admin'] },
  { label: '用户管理', path: '/users', roles: ['admin'] },
]

const session = useSessionStore()
const connection = useConnectionStore()
const router = useRouter()
const route = useRoute()

const changePasswordVisible = ref(false)

const visibleMenus = computed(() => {
  const currentRole = session.role
  if (!currentRole) {
    return []
  }
  return ALL_MENUS.filter((menu) => menu.roles.includes(currentRole))
})

function navigateTo(path: string): void {
  router.push(path)
}

function handleUserCommand(command: string): void {
  if (command === 'change-password') {
    changePasswordVisible.value = true
  } else if (command === 'logout') {
    handleLogout()
  }
}

async function handleLogout(): Promise<void> {
  await session.logout()
  router.push('/login')
}

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
  <div class="main-layout">
    <header class="main-header">
      <div class="header-left">
        <div class="brand-logo">
          <div class="shield-icon" aria-hidden="true">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path
                class="shield-shape"
                d="M8 1L3 4v4c0 3.3 2.2 6.2 5 7 2.8-.8 5-3.7 5-7V4L8 1z"
                fill="currentColor"
              />
              <text
                class="shield-letter"
                x="8"
                y="10"
                text-anchor="middle"
                font-size="6"
                font-weight="700"
                fill="currentColor"
              >
                A
              </text>
            </svg>
          </div>
          <div class="brand-text">
            <span class="brand-cn" data-testid="brand-name">安帝科技</span>
            <span class="brand-en" data-testid="brand-en-name">ANDISEC</span>
          </div>
        </div>
        <span class="header-separator" aria-hidden="true" />
        <div class="system-title">
          <span class="sys-name" data-testid="product-name">USB安全管理系统</span>
          <span class="sys-sub">运维管理控制台 V1.0</span>
        </div>
      </div>

      <div class="header-right" data-testid="header-controls">
        <el-dropdown trigger="click" @command="handleUserCommand">
          <button
            type="button"
            class="user-menu-trigger"
            data-testid="user-menu-trigger"
            aria-label="用户菜单"
          >
            <el-icon aria-hidden="true"><User /></el-icon>
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="change-password">修改密码</el-dropdown-item>
              <el-dropdown-item command="logout" divided>登出</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <div class="win-controls" data-testid="window-controls">
          <button
            class="win-btn"
            title="最小化"
            data-testid="window-minimize"
            @click="handleMinimize"
          >
            <el-icon><Minus /></el-icon>
          </button>
          <button
            class="win-btn"
            title="最大化"
            data-testid="window-maximize"
            @click="handleMaximize"
          >
            <el-icon><FullScreen /></el-icon>
          </button>
          <button
            class="win-btn win-btn-close"
            title="关闭"
            data-testid="window-close"
            @click="handleClose"
          >
            <el-icon><Close /></el-icon>
          </button>
        </div>
      </div>
    </header>

    <div class="main-body">
      <aside class="main-sidebar">
        <nav class="sidebar-nav">
          <div
            v-for="menu in visibleMenus"
            :key="menu.path"
            class="nav-item"
            :class="{ active: route.path === menu.path }"
            data-testid="nav-item"
            @click="navigateTo(menu.path)"
          >
            {{ menu.label }}
          </div>
        </nav>
      </aside>

      <main class="main-content">
        <router-view />
      </main>
    </div>

    <footer class="main-footer" data-testid="connection-status">
      <span
        class="connection-dot"
        :class="connection.isConnected ? 'connected' : 'disconnected'"
        data-testid="connection-dot"
      />
      <span>{{ connection.isConnected ? '已连接' : '未连接' }}</span>
    </footer>

    <ChangePasswordDialog v-model:visible="changePasswordVisible" />
  </div>
</template>

<style scoped lang="scss">
.main-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  min-width: $window-min-width;
  min-height: $window-min-height;
  color: $text-primary;
  font-family: $font-family-base;
  font-size: $font-size-base;
  line-height: $line-height-base;
}

.main-header {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  height: $header-height;
  padding: 0 12px;
  color: $color-white;
  background: linear-gradient(135deg, $brand-header-start, $brand-header-end);
  user-select: none;
  -webkit-app-region: drag;
}

.header-left {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 12px;
}

.brand-logo {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shield-icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: $brand-logo-size;
  height: $brand-logo-size;
  color: $brand-primary;
  background: $brand-logo-shield-bg;
  border-radius: $brand-logo-radius;
}

.shield-shape {
  color: $brand-primary;
}

.shield-letter {
  color: $color-white;
}

.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-cn {
  color: $brand-text-cn;
  font-size: $font-base;
  font-weight: $font-weight-semibold;
  letter-spacing: $letter-spacing-brand;
}

.brand-en {
  color: $brand-text-en;
  font-size: $font-xs;
  letter-spacing: $letter-spacing-brand-en;
}

.header-separator {
  flex-shrink: 0;
  width: $border-width;
  height: $spacing-8;
  background: $brand-separator;
}

.system-title {
  display: flex;
  flex-direction: column;
  line-height: 1.3;
}

.sys-name {
  color: $brand-sys-name;
  font-size: $font-lg;
  font-weight: $font-weight-semibold;
  letter-spacing: $letter-spacing-product;
}

.sys-sub {
  color: $brand-sys-sub;
  font-size: $font-sm;
}

.header-right {
  display: flex;
  align-items: center;
  margin-right: 8px;
  margin-left: auto;
  gap: 12px;
  -webkit-app-region: no-drag;
}

.user-menu-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  padding: 0;
  color: $color-white;
  font-size: $font-size-lg;
  background: transparent;
  border: none;
  border-radius: $border-radius;
  cursor: pointer;

  &:hover,
  &:focus-visible {
    background: $win-btn-hover-bg;
  }
}

.win-controls {
  display: flex;
  -webkit-app-region: no-drag;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 28px;
  color: $win-btn-color;
  font-size: $font-base;
  background: transparent;
  border: none;
  border-radius: 0;
  cursor: pointer;

  &:hover {
    background: $win-btn-hover-bg;
  }
}

.win-btn-close:hover {
  color: $color-white;
  background: $win-btn-close-hover-bg;
}

.main-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.main-sidebar {
  flex-shrink: 0;
  width: $sidebar-width;
  overflow-y: auto;
  background: $bg-sidebar;
  border-right: $border-width solid $border-color;
}

.sidebar-nav {
  padding: $spacing-3 0;
}

.nav-item {
  padding: $menu-item-padding-block $menu-item-padding-inline;
  color: $text-primary;
  font-size: $font-lg;
  cursor: pointer;
  transition:
    color 0.15s,
    background-color 0.15s;

  &:hover {
    background: $hover-sidebar;
  }

  &.active {
    color: $brand-primary;
    font-weight: $font-weight-medium;
    background: $active-sidebar-bg;
    border-right: $menu-active-border-width solid $active-sidebar-border;
  }
}

.main-content {
  flex: 1;
  padding: $content-padding;
  overflow-y: auto;
  background: $bg-page;
}

.main-footer {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  height: $footer-height;
  padding: 0 $content-padding;
  color: $text-secondary;
  font-size: $font-base;
  background: $bg-white;
  border-top: $border-width solid $border-color;
  gap: $spacing-2;
}

.connection-dot {
  width: $connection-dot-size;
  height: $connection-dot-size;
  border-radius: 50%;

  &.connected {
    background: $color-success;
  }

  &.disconnected {
    background: $color-danger;
  }
}
</style>
