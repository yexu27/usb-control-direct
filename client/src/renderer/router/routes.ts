import type { RouteRecordRaw } from 'vue-router'
import type { UserRole } from '../../shared/connection-state'

declare module 'vue-router' {
  interface RouteMeta {
    guest?: boolean
    licenseFlow?: boolean
    requiresAuth?: boolean
    roles?: UserRole[]
    rootEntry?: boolean
  }
}

export const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/pages/LoginPage.vue'),
    meta: { guest: true },
  },
  {
    path: '/license',
    name: 'License',
    component: () => import('@/pages/LicensePage.vue'),
    meta: { licenseFlow: true },
  },
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      {
        path: 'file-access',
        name: 'FileAccess',
        component: () => import('@/pages/FileAccessPage.vue'),
        meta: { requiresAuth: true, roles: ['operator'] },
      },
      {
        path: 'usb-devices',
        name: 'UsbDevices',
        component: () => import('@/pages/UsbDevicesPage.vue'),
        meta: { requiresAuth: true, roles: ['operator'] },
      },
      {
        path: 'policies',
        name: 'Policies',
        component: () => import('@/pages/PoliciesPage.vue'),
        meta: { requiresAuth: true, roles: ['operator'] },
      },
      {
        path: 'logs',
        name: 'Logs',
        component: () => import('@/pages/LogsPage.vue'),
        meta: { requiresAuth: true, roles: ['auditor'] },
      },
      {
        path: 'system',
        name: 'System',
        component: () => import('@/pages/SystemPage.vue'),
        meta: { requiresAuth: true, roles: ['admin'] },
      },
      {
        path: 'users',
        name: 'Users',
        component: () => import('@/pages/UsersPage.vue'),
        meta: { requiresAuth: true, roles: ['admin'] },
      },
      {
        path: '',
        name: 'RootEntry',
        component: { render: () => null },
        meta: { rootEntry: true },
      },
    ],
  },
]

export const ROLE_DEFAULT_ROUTES: Record<UserRole, string> = {
  admin: '/users',
  operator: '/file-access',
  auditor: '/logs',
}
