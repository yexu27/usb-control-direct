import type { RouteMeta } from 'vue-router'
import { useSessionStore } from '@/stores/session'
import { useConnectionStore } from '@/stores/connection'
import { ROLE_DEFAULT_ROUTES } from './routes'

export function resolveRouteAccess(meta: RouteMeta): true | string {
  if (meta.guest === true) {
    return true
  }

  const session = useSessionStore()
  if (!session.isLoggedIn) {
    return '/login'
  }

  const connection = useConnectionStore()
  if (connection.status === 'AUTH_REQUIRED' || connection.status === 'LICENSE_EXPIRED') {
    return '/license'
  }

  if (meta.requiresAuth === true && meta.roles != null) {
    const currentRole = session.role
    if (!currentRole || !meta.roles.includes(currentRole)) {
      return currentRole ? ROLE_DEFAULT_ROUTES[currentRole] : '/login'
    }
  }

  return true
}
