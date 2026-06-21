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
</script>

<template>
  <div class="main-layout">
    <header class="main-header">
      <div class="header-left">
        <span class="brand-name">USB安全管理系统</span>
      </div>
      <div class="header-right">
        <span class="device-ip">{{ connection.deviceIp }}</span>
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
  background: linear-gradient(90deg, $brand-header-start, $brand-header-end);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
}

.brand-name {
  font-size: 16px;
  font-weight: 600;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-dropdown-trigger {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #fff;
  cursor: pointer;
  font-size: 14px;
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
  font-size: 14px;

  &:hover {
    background: darken($bg-sidebar, 5%);
  }

  &.active {
    color: $brand-primary;
    background: #e6f0fa;
    border-right: 3px solid $brand-primary;
    font-weight: 500;
  }
}

.main-content {
  flex: 1;
  overflow-y: auto;
  background: $bg-page;
  padding: 16px;
}

.main-footer {
  height: 28px;
  background: #fff;
  border-top: 1px solid $border-color;
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 6px;
  font-size: 12px;
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
