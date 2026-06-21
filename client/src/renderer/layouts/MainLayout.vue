<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ArrowDown } from '@element-plus/icons-vue'
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
</script>

<template>
  <div class="main-layout">
    <header class="main-header">
      <div class="header-left">
        <div class="brand-logo" aria-hidden="true">
          <svg viewBox="0 0 16 16" role="img">
            <path d="M8 1 3 4v4c0 3.3 2.2 6.2 5 7 2.8-.8 5-3.7 5-7V4L8 1Z" />
            <path class="brand-logo-inner" d="M8 3 5 5v3.5c0 2.2 1.3 4 3 4.6 1.7-.6 3-2.4 3-4.6V5L8 3Z" />
          </svg>
        </div>
        <div class="brand-copy">
          <span class="brand-name" data-testid="brand-name">安帝科技</span>
          <span class="brand-name-en" data-testid="brand-en-name">ANDISEC</span>
        </div>
        <span class="header-divider" aria-hidden="true" />
        <div class="product-copy">
          <span class="product-name" data-testid="product-name">USB安全管理系统</span>
          <span class="product-subtitle">运维管理控制台</span>
        </div>
      </div>
      <div class="header-right">
        <span class="device-ip" data-testid="device-ip">
          装置：{{ connection.deviceIp || '--' }}
        </span>
        <el-dropdown trigger="click" @command="handleUserCommand">
          <span class="user-dropdown-trigger" data-testid="current-user">
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
  padding: 0 $spacing-7;
  background: linear-gradient(90deg, $brand-header-start, $brand-header-end);
  color: $text-on-brand;
}

.header-left {
  display: flex;
  align-items: center;
  gap: $spacing-4;
}

.brand-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: $brand-logo-size;
  height: $brand-logo-size;
  color: $brand-primary;
  background: $color-white;
  border-radius: $brand-logo-radius;

  svg {
    width: $font-size-lg;
    height: $font-size-lg;
    fill: currentcolor;
  }

  .brand-logo-inner {
    fill: $color-white;
  }
}

.brand-copy,
.product-copy {
  display: flex;
  flex-direction: column;
}

.brand-name {
  font-size: $font-size-sm;
  font-weight: $font-weight-semibold;
  letter-spacing: $letter-spacing-brand;
}

.brand-name-en {
  color: $text-on-brand-muted;
  font-size: $font-size-xs;
  letter-spacing: $letter-spacing-brand-en;
}

.header-divider {
  width: $border-width;
  height: $spacing-8;
  margin: 0 $spacing-2;
  background: rgba($color-white, 0.25);
}

.product-name {
  font-size: $font-size-md;
  font-weight: $font-weight-semibold;
  letter-spacing: $letter-spacing-product;
}

.product-subtitle {
  color: $text-on-brand-muted;
  font-size: $font-size-xs;
}

.header-right {
  display: flex;
  align-items: center;
  gap: $spacing-5;
}

.device-ip {
  color: $text-on-brand-muted;
  font-size: $font-size-sm;
}

.user-dropdown-trigger {
  display: flex;
  align-items: center;
  gap: $spacing-1;
  color: $text-on-brand;
  cursor: pointer;
  font-size: $font-size-md;
}

.main-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.main-sidebar {
  width: $sidebar-width;
  background: $bg-sidebar;
  border-right: $border-width solid $border-color;
  flex-shrink: 0;
  overflow-y: auto;
}

.sidebar-nav {
  padding: $spacing-2 0;
}

.nav-item {
  padding: $menu-item-padding-block $menu-item-padding-inline;
  color: $text-primary;
  font-size: $font-size-base;
  cursor: pointer;
  border-left: $menu-active-border-width solid transparent;
  transition:
    color 0.15s,
    background-color 0.15s;

  &:hover {
    color: $brand-primary;
    background: $menu-item-hover-bg;
  }

  &.active {
    color: $brand-primary;
    font-weight: $font-weight-semibold;
    background: $menu-item-active-bg;
    border-left-color: $brand-primary;
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
  padding: 0 $spacing-6;
  gap: $spacing-2;
  color: $text-secondary;
  font-size: $font-size-sm;
  background: $bg-sidebar-footer;
  border-top: $border-width solid $border-color;
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
