<script setup lang="ts">
import { computed, ref, type Component } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  Close,
  Connection,
  DocumentChecked,
  Folder,
  FullScreen,
  Minus,
  Setting,
  Tickets,
  User,
} from '@element-plus/icons-vue'
import { useSessionStore } from '@/stores/session'
import { useConnectionStore } from '@/stores/connection'
import ChangePasswordDialog from '@/components/ChangePasswordDialog.vue'
import type { UserRole } from '../../shared/connection-state'

interface MenuItem {
  label: string
  path: string
  roles: UserRole[]
  icon: Component
}

const ALL_MENUS: MenuItem[] = [
  { label: '文件访问控制', path: '/file-access', roles: ['operator'], icon: Folder },
  { label: 'U盘设备控制', path: '/usb-devices', roles: ['operator'], icon: Connection },
  { label: '策略管理', path: '/policies', roles: ['operator'], icon: DocumentChecked },
  { label: '日志管理', path: '/logs', roles: ['auditor'], icon: Tickets },
  { label: '系统管理', path: '/system', roles: ['admin'], icon: Setting },
  { label: '用户管理', path: '/users', roles: ['admin'], icon: User },
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
  <div class="app-desktop">
    <div class="main-layout app-window-shell">
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
            <el-icon class="nav-icon" aria-hidden="true">
              <component :is="menu.icon" />
            </el-icon>
            <span>{{ menu.label }}</span>
          </div>
        </nav>
        <div class="sidebar-footer">
          <span class="sidebar-footer-brand">安帝科技 · 系统状态</span>
          <span class="connection-status" data-testid="connection-status">
            <span
              class="connection-dot"
              :class="connection.isConnected ? 'connected' : 'disconnected'"
              data-testid="connection-dot"
            />
            <span>{{ connection.isConnected ? '已连接' : '未连接' }}</span>
          </span>
        </div>
      </aside>

      <main class="main-content app-content-scroll">
        <router-view />
      </main>
    </div>

      <ChangePasswordDialog v-model:visible="changePasswordVisible" />
    </div>
  </div>
</template>
