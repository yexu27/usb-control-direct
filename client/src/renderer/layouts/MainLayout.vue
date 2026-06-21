<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ArrowDown, Minus, FullScreen, Close } from '@element-plus/icons-vue'
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
  const currentRole = session.role as UserRole
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
          <div class="shield-icon">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M8 1L3 4v4c0 3.3 2.2 6.2 5 7 2.8-.8 5-3.7 5-7V4L8 1z" fill="#0056b3"/>
              <text x="8" y="10" text-anchor="middle" font-size="6" font-weight="700" fill="#fff">A</text>
            </svg>
          </div>
          <div class="brand-text">
            <span class="brand-cn">安帝科技</span>
            <span class="brand-en">ANDISEC</span>
          </div>
        </div>
        <span class="header-separator" />
        <div class="system-title">
          <span class="sys-name">USB安全管理系统</span>
          <span class="sys-sub">运维管理控制台 V1.0</span>
        </div>
      </div>

      <div class="header-right">
        <span v-if="connection.deviceIp" class="device-ip">{{ connection.deviceIp }}</span>
        <el-dropdown trigger="click" @command="handleUserCommand">
          <span class="user-dropdown-trigger">
            {{ session.username }}
            <el-icon><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="change-password">修改密码</el-dropdown-item>
              <el-dropdown-item command="logout" divided>登出</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>

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
    </header>

    <div class="main-body">
      <aside class="main-sidebar">
        <nav class="sidebar-nav">
          <div
            v-for="menu in visibleMenus"
            :key="menu.path"
            class="nav-item"
            :class="{ active: route.path === menu.path }"
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

    <footer class="main-footer">
      <span
        class="connection-dot"
        :class="connection.isConnected ? 'connected' : 'disconnected'"
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
}

.main-header {
  height: $header-height;
  background: linear-gradient(135deg, $brand-header-start, $brand-header-end);
  color: $color-white;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  flex-shrink: 0;
  -webkit-app-region: drag;
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.brand-logo {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shield-icon {
  width: 28px;
  height: 28px;
  background: $brand-logo-shield-bg;
  border-radius: $border-radius-lg;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-cn {
  font-size: $font-base;
  font-weight: 600;
  color: $brand-text-cn;
  letter-spacing: 1px;
}

.brand-en {
  font-size: $font-xs;
  color: $brand-text-en;
  letter-spacing: 1.5px;
}

.header-separator {
  width: 1px;
  height: 24px;
  background: $brand-separator;
  flex-shrink: 0;
}

.system-title {
  display: flex;
  flex-direction: column;
  line-height: 1.3;
}

.sys-name {
  font-size: $font-lg;
  font-weight: 600;
  color: $brand-sys-name;
  letter-spacing: 0.5px;
}

.sys-sub {
  font-size: $font-sm;
  color: $brand-sys-sub;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-left: auto;
  margin-right: 8px;
  -webkit-app-region: no-drag;
}

.device-ip {
  font-size: $font-base;
  color: $device-ip-color;
  background: $device-ip-bg;
  border-radius: $border-radius;
  padding: 2px 8px;
}

.user-dropdown-trigger {
  display: flex;
  align-items: center;
  gap: 4px;
  color: $color-white;
  cursor: pointer;
  font-size: $font-md;
}

.win-controls {
  display: flex;
  -webkit-app-region: no-drag;
}

.win-btn {
  width: 36px;
  height: 28px;
  border: none;
  background: transparent;
  color: $win-btn-color;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: $font-base;
  border-radius: 0;

  &:hover {
    background: $win-btn-hover-bg;
  }
}

.win-btn-close:hover {
  background: $win-btn-close-hover-bg;
  color: $color-white;
}

.main-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.main-sidebar {
  width: $sidebar-width;
  background: $bg-sidebar;
  border-right: 1px solid $border-color;
  flex-shrink: 0;
  overflow-y: auto;
}

.sidebar-nav {
  padding: 8px 0;
}

.nav-item {
  padding: 12px 20px;
  cursor: pointer;
  color: $text-primary;
  font-size: $font-lg;

  &:hover {
    background: $hover-sidebar;
  }

  &.active {
    color: $brand-primary;
    background: $active-sidebar-bg;
    border-right: 3px solid $active-sidebar-border;
    font-weight: 500;
  }
}

.main-content {
  flex: 1;
  overflow-y: auto;
  background: $bg-page;
  padding: $content-padding;
}

.main-footer {
  height: $footer-height;
  background: $bg-white;
  border-top: 1px solid $border-color;
  display: flex;
  align-items: center;
  padding: 0 $content-padding;
  gap: 6px;
  font-size: $font-base;
  color: $text-secondary;
  flex-shrink: 0;
}

.connection-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;

  &.connected {
    background: $color-success;
  }

  &.disconnected {
    background: $color-danger;
  }
}
</style>
