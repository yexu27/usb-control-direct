import { createRouter, createWebHashHistory } from 'vue-router'
import { routes, ROLE_DEFAULT_ROUTES } from './routes'
import { useSessionStore } from '@/stores/session'
import { useConnectionStore } from '@/stores/connection'
import type { UserRole } from '../../shared/connection-state'

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

router.beforeEach((to, _from) => {
  if (to.meta.guest === true) {
    return true
  }

  const session = useSessionStore()
  if (!session.isLoggedIn) {
    return '/login'
  }

  const connection = useConnectionStore()
  if (
    connection.status === 'AUTH_REQUIRED' ||
    connection.status === 'LICENSE_EXPIRED'
  ) {
    return '/license'
  }

  if (to.meta.requiresAuth === true && to.meta.roles != null) {
    const currentRole = session.role as UserRole
    if (!to.meta.roles.includes(currentRole)) {
      const defaultRoute = ROLE_DEFAULT_ROUTES[currentRole]
      return defaultRoute ?? '/login'
    }
  }

  return true
})

export default router
